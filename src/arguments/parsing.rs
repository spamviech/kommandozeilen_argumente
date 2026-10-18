//! Staged parsing implementation and intermediate recognition state.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    env,
    ffi::OsString,
    fmt::Display,
    num::NonZeroI32,
    process, ptr,
};

use itertools::Itertools as _;
use nonempty::NonEmpty;

use crate::{
    arguments::Arguments,
    description::ArgumentInput,
    language::Language,
    outcome::{self, AlternativeErrors, Error, Result as OutcomeResult},
};

/// An early-exit argument recognized while parsing an input argument.
#[derive(Debug, Clone)]
pub struct ParsedEarlyExit<'s> {
    /// The matched argument name, without its prefix.
    pub name: Cow<'s, str>,
    /// The message to display for the early exit.
    pub message: Cow<'s, str>,
    /// The original input argument that contained the matched name.
    pub input: Cow<'s, str>,
}

/// A short flag recognized while parsing an input argument.
#[derive(Debug, Clone)]
pub struct ParsedFlag<'s> {
    /// The matched argument name, without its prefix.
    pub name: Cow<'s, str>,
    /// The boolean value parsed from the flag, including long-form inversion.
    pub value: bool,
    /// The original input argument that contained the matched name.
    pub input: Cow<'s, str>,
}

/// The name of a value argument recognized during parsing.
///
/// This is the key for the parsed-stage `values` maps.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParsedValueName<'s> {
    /// The matched argument name, without its prefix.
    pub name: Cow<'s, str>,
}

/// The raw value associated with a recognized value argument.
#[derive(Debug, Clone)]
pub struct ParsedRawValue<'s> {
    /// The unparsed operating-system string.
    pub value: OsString,
    /// The original input argument that supplied the value.
    pub input: Cow<'s, str>,
}

/// An alternative branch selected during name recognition.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlternativeBranch {
    /// Identity of the `Alternatives` allocation that owns this branch.
    alternatives: *const (),
    /// Index of the selected branch within that allocation.
    index: usize,
}

/// Returns whether two recognition candidates chose compatible alternative branches.
fn alternative_id<T>(alternatives: &T) -> *const () {
    ptr::from_ref(alternatives).cast()
}

/// Returns whether two recognition candidates chose compatible alternative branches.
fn compatible_alternative_branches(
    first: &[AlternativeBranch],
    second: &[AlternativeBranch],
) -> bool {
    first.iter().all(|first_branch| {
        second.iter().all(|second_branch| {
            first_branch.alternatives != second_branch.alternatives
                || first_branch.index == second_branch.index
        })
    })
}

/// Common data produced by a name-recognition stage.
///
/// The three public stage names below are aliases because their output is structurally identical.
/// Keeping one representation prevents their construction and merging semantics from diverging.
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct ParsedForms<'argument> {
    /// Early-exit arguments recognized in this stage.
    pub early_exits: Vec<ParsedEarlyExit<'argument>>,
    /// Flags recognized in this stage.
    pub flags: Vec<ParsedFlag<'argument>>,
    /// Value names recognized without an eligible raw value.
    pub missing_values: HashSet<ParsedValueName<'argument>>,
    /// Raw values recognized in this stage, keyed by their value name.
    pub values: HashMap<ParsedValueName<'argument>, ParsedRawValue<'argument>>,
    /// Input not consumed by this stage.
    pub remaining: Vec<ArgumentInput>,
    /// Branches selected by nested alternatives during this stage.
    pub alternative_branches: Vec<AlternativeBranch>,
}

/// Output of stage 1, merged short-name recognition.
pub type ParsedMergedShortForms<'argument> = ParsedForms<'argument>;

/// Output of stage 2, standalone short-name recognition.
pub type ParsedShortForms<'argument> = ParsedForms<'argument>;

/// Output of stage 3, long-name recognition.
pub type ParsedLongForms<'argument> = ParsedForms<'argument>;

/// Recognition state accumulated from the three name-recognition stages.
#[derive(Debug, Clone)]
pub struct RecognizedArguments<'argument> {
    /// All recognized early exits.
    pub early_exits: Vec<ParsedEarlyExit<'argument>>,
    /// All recognized flags.
    pub flags: Vec<ParsedFlag<'argument>>,
    /// Recognized value names for which no raw value was eligible.
    pub missing_values: HashSet<ParsedValueName<'argument>>,
    /// Recognized raw values. Repeated names retain the last value.
    pub values: HashMap<ParsedValueName<'argument>, ParsedRawValue<'argument>>,
    /// Input unconsumed after stage 3.
    pub remaining: Vec<ArgumentInput>,
    /// Branches selected during name recognition.
    pub alternative_branches: Vec<AlternativeBranch>,
}

impl<'argument> RecognizedArguments<'argument> {
    /// Combines the results of the three name-recognition stages.
    fn from_stages(
        merged: ParsedMergedShortForms<'argument>,
        short: ParsedShortForms<'argument>,
        long: ParsedLongForms<'argument>,
    ) -> Self {
        let mut result = Self {
            early_exits: merged.early_exits,
            flags: merged.flags,
            missing_values: merged.missing_values,
            values: merged.values,
            remaining: long.remaining,
            alternative_branches: merged.alternative_branches,
        };
        result.alternative_branches.extend(short.alternative_branches);
        result.alternative_branches.extend(long.alternative_branches);
        result.merge_stage(short.early_exits, short.flags, short.missing_values, short.values);
        result.merge_stage(long.early_exits, long.flags, long.missing_values, long.values);
        result
    }

    /// Merges one recognition stage into this accumulated state.
    fn merge_stage(
        &mut self,
        mut early_exits: Vec<ParsedEarlyExit<'argument>>,
        mut flags: Vec<ParsedFlag<'argument>>,
        missing_values: HashSet<ParsedValueName<'argument>>,
        values: HashMap<ParsedValueName<'argument>, ParsedRawValue<'argument>>,
    ) {
        self.early_exits.append(&mut early_exits);
        self.flags.append(&mut flags);
        for name in missing_values {
            if !self.values.contains_key(&name) {
                let _ = self.missing_values.insert(name);
            }
        }
        for (name, value) in values {
            let _ = self.missing_values.remove(&name);
            let _ = self.values.insert(name, value);
        }
    }
}

impl ParsedForms<'_> {
    /// Creates an empty result for a name-recognition stage.
    pub(crate) fn empty() -> Self {
        Self {
            early_exits: Vec::new(),
            flags: Vec::new(),
            missing_values: HashSet::new(),
            values: HashMap::new(),
            remaining: Vec::new(),
            alternative_branches: Vec::new(),
        }
    }

    /// Returns whether this candidate and `next` selected compatible branches.
    pub(crate) fn compatible_with(&self, alternative_branches: &[AlternativeBranch]) -> bool {
        compatible_alternative_branches(&self.alternative_branches, alternative_branches)
    }

    /// Appends the next result from the same recognition stage.
    pub(crate) fn merge(mut self, mut next: Self) -> Self {
        self.alternative_branches.append(&mut next.alternative_branches);
        self.early_exits.append(&mut next.early_exits);
        self.flags.append(&mut next.flags);
        for name in next.missing_values {
            if !self.values.contains_key(&name) {
                let _ = self.missing_values.insert(name);
            }
        }
        for (name, value) in next.values {
            let _ = self.missing_values.remove(&name);
            let _ = self.values.insert(name, value);
        }
        self.remaining = next.remaining;
        self
    }
}

impl<'t, T, F> Arguments<'t, T, F> {
    /// Parses merged short-form arguments.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> NonEmpty<ParsedMergedShortForms<'t>> {
        self.parse_merged_short_inputs(args.map(ArgumentInput::Unchanged))
    }

    /// Parses merged short-form arguments from staged input.
    #[inline]
    pub(crate) fn parse_merged_short_inputs(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> NonEmpty<ParsedMergedShortForms<'t>> {
        match self {
            Self::Single(argument) => argument.parse_merged_short_input_candidates(args),
            Self::Combined(combine) => combine.parse_merged_short_inputs(Box::new(args)),
            Self::Alternatives(alternatives) => {
                let args = args.collect_vec();
                let alternatives_id = alternative_id(alternatives.as_ref());
                NonEmpty::collect(alternatives.iter().enumerate().flat_map(|(index, argument)| {
                    argument.parse_merged_short_inputs(args.iter().cloned()).into_iter().map(
                        move |mut candidate| {
                            candidate
                                .alternative_branches
                                .push(AlternativeBranch { alternatives: alternatives_id, index });
                            candidate
                        },
                    )
                }))
                .expect("Iterator of NonEmpty<NonEmpty<_>>.")
            },
        }
    }

    /// Parses standalone short-form arguments after merged short forms.
    #[inline]
    #[expect(
        clippy::missing_panics_doc,
        reason = "Collecting an iterator of an NonEmpty to a NonEmpty."
    )]
    pub fn parse_short_forms(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> NonEmpty<ParsedShortForms<'t>> {
        match self {
            Self::Single(argument) => NonEmpty::singleton(argument.parse_short_form(args)),
            Self::Combined(combine) => combine.parse_short_forms(Box::new(args)),
            Self::Alternatives(alternatives) => {
                let args = args.collect_vec();
                let alternatives_id = alternative_id(alternatives.as_ref());
                NonEmpty::collect(alternatives.iter().enumerate().flat_map(|(index, argument)| {
                    argument.parse_short_forms(args.iter().cloned()).into_iter().map(
                        move |mut candidate| {
                            candidate
                                .alternative_branches
                                .push(AlternativeBranch { alternatives: alternatives_id, index });
                            candidate
                        },
                    )
                }))
                .expect("Iterator of NonEmpty<NonEmpty<_>>.")
            },
        }
    }

    /// Parses long-form arguments after standalone short forms.
    #[inline]
    #[expect(
        clippy::missing_panics_doc,
        reason = "Collecting an iterator of an NonEmpty to a NonEmpty."
    )]
    pub fn parse_long_forms(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> NonEmpty<ParsedLongForms<'t>> {
        match self {
            Self::Single(argument) => NonEmpty::singleton(argument.parse_long_form(args)),
            Self::Combined(combine) => combine.parse_long_forms(Box::new(args)),
            Self::Alternatives(alternatives) => {
                let args = args.collect_vec();
                let alternatives_id = alternative_id(alternatives.as_ref());
                NonEmpty::collect(alternatives.iter().enumerate().flat_map(|(index, argument)| {
                    argument.parse_long_forms(args.iter().cloned()).into_iter().map(
                        move |mut candidate| {
                            candidate
                                .alternative_branches
                                .push(AlternativeBranch { alternatives: alternatives_id, index });
                            candidate
                        },
                    )
                }))
                .expect("Iterator of NonEmpty<NonEmpty<_>>.")
            },
        }
    }
}

#[expect(
    clippy::multiple_inherent_impl,
    reason = "Impl for parsing after methods for intermediate results."
)]
impl<'t, T, F> Arguments<'t, T, F> {
    /// Parses the supplied arguments through the six staged parsing steps.
    #[inline]
    #[expect(
        clippy::missing_panics_doc,
        reason = "Length is always checked before collecting to a NonEmpty."
    )]
    pub fn parse(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> (OutcomeResult<'t, T, F>, Vec<ArgumentInput>) {
        match self {
            Self::Alternatives(alternatives) => {
                let args = args.collect_vec();
                let mut errors = Vec::new();
                let mut remaining = Vec::new();
                for alternative in *alternatives {
                    let (result, candidate_remaining) = alternative.parse(args.iter().cloned());
                    match result {
                        OutcomeResult::Value(value) => {
                            return (OutcomeResult::Value(value), candidate_remaining);
                        },
                        OutcomeResult::EarlyExit(messages) => {
                            return (OutcomeResult::EarlyExit(messages), candidate_remaining);
                        },
                        OutcomeResult::Error(candidate_errors) => {
                            remaining = candidate_remaining;
                            errors.push(candidate_errors);
                        },
                    }
                }
                (
                    OutcomeResult::Error(NonEmpty::singleton(Error::Alternatives(Box::new(
                        AlternativeErrors(
                            NonEmpty::from_vec(errors).expect("Alternatives are non-empty."),
                        ),
                    )))),
                    remaining,
                )
            },
            arguments @ (Self::Single(_) | Self::Combined(_)) => {
                let input = args.collect_vec();
                let mut errors = Vec::new();
                let mut selected = None;
                for merged in arguments.parse_merged_short_forms(input.iter().cloned()) {
                    for short in arguments.parse_short_forms(merged.remaining.clone().into_iter()) {
                        if !merged.compatible_with(&short.alternative_branches) {
                            continue;
                        }
                        for long in arguments.parse_long_forms(short.remaining.clone().into_iter())
                        {
                            if !merged.compatible_with(&long.alternative_branches)
                                || !short.compatible_with(&long.alternative_branches)
                            {
                                continue;
                            }
                            let candidate = RecognizedArguments::from_stages(
                                merged.clone(),
                                short.clone(),
                                long,
                            );
                            match arguments.validate(candidate.clone()) {
                                OutcomeResult::Value(()) | OutcomeResult::EarlyExit(_) => {
                                    selected = Some(candidate);
                                    break;
                                },
                                OutcomeResult::Error(candidate_errors) => {
                                    errors.push(candidate_errors);
                                },
                            }
                        }
                        if selected.is_some() {
                            break;
                        }
                    }
                    if selected.is_some() {
                        break;
                    }
                }
                if let Some(candidate) = selected {
                    let remaining = candidate.remaining.clone();
                    (arguments.accumulate(candidate), remaining)
                } else {
                    let errors = NonEmpty::from_vec(errors)
                        .expect("Parsing has at least one failed candidate.");
                    let errors = if errors.len() == 1 {
                        errors.head
                    } else {
                        NonEmpty::singleton(Error::Alternatives(Box::new(AlternativeErrors(
                            errors,
                        ))))
                    };
                    (OutcomeResult::Error(errors), Vec::new())
                }
            },
        }
    }

    /// Validates a candidate's typed parsing before consuming the parser branch.
    pub(crate) fn validate(&self, recognized: RecognizedArguments<'t>) -> OutcomeResult<'t, (), F> {
        match self {
            Self::Single(argument) => argument.validate(recognized),
            Self::Combined(combine) => combine.validate(recognized),
            Self::Alternatives(alternatives) => {
                let alternatives_id = alternative_id(alternatives.as_ref());
                let selected_branch = recognized
                    .alternative_branches
                    .iter()
                    .find(|branch| branch.alternatives == alternatives_id)
                    .map(|branch| branch.index);
                let mut errors = Vec::new();
                for (index, alternative) in alternatives.iter().enumerate() {
                    if selected_branch.is_some_and(|selected_branch| selected_branch != index) {
                        continue;
                    }
                    match alternative.validate(recognized.clone()) {
                        OutcomeResult::Value(()) => return OutcomeResult::Value(()),
                        OutcomeResult::EarlyExit(messages) => {
                            return OutcomeResult::EarlyExit(messages);
                        },
                        OutcomeResult::Error(candidate_errors) => errors.push(candidate_errors),
                    }
                }
                OutcomeResult::Error(NonEmpty::singleton(Error::Alternatives(Box::new(
                    AlternativeErrors(
                        NonEmpty::from_vec(errors).expect("Alternatives are non-empty."),
                    ),
                ))))
            },
        }
    }

    /// Accumulates this argument's recognized inputs into its typed result.
    pub(crate) fn accumulate(self, recognized: RecognizedArguments<'t>) -> OutcomeResult<'t, T, F> {
        match self {
            Self::Single(argument) => argument.accumulate(recognized),
            Self::Combined(combine) => combine.accumulate(recognized),
            Self::Alternatives(alternatives) => {
                let alternatives_id = alternative_id(alternatives.as_ref());
                let selected_branch = recognized
                    .alternative_branches
                    .iter()
                    .find(|branch| branch.alternatives == alternatives_id)
                    .map(|branch| branch.index);
                let mut errors = Vec::new();
                for (index, alternative) in alternatives.into_iter().enumerate() {
                    if selected_branch.is_some_and(|selected_branch| selected_branch != index) {
                        continue;
                    }
                    match alternative.validate(recognized.clone()) {
                        OutcomeResult::Value(()) | OutcomeResult::EarlyExit(_) => {
                            return alternative.accumulate(recognized);
                        },
                        OutcomeResult::Error(candidate_errors) => errors.push(candidate_errors),
                    }
                }
                OutcomeResult::Error(NonEmpty::singleton(Error::Alternatives(Box::new(
                    AlternativeErrors(
                        NonEmpty::from_vec(errors).expect("Alternatives are non-empty."),
                    ),
                ))))
            },
        }
    }

    /// Parses arguments from the environment.
    #[inline]
    pub fn parse_from_env(self) -> (outcome::Result<'t, T, F>, Vec<ArgumentInput>)
    where
        Self: 't,
        F: 't,
    {
        self.parse(env::args_os().skip(1))
    }

    /// Parses arguments and exits successfully after writing an early-exit message.
    #[inline]
    pub fn parse_with_early_exit(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> (Result<T, NonEmpty<Error<'t, F>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        F: 't,
    {
        let (result, remaining) = self.parse(args);
        let result = match result {
            outcome::Result::Value(value) => Ok(value),
            outcome::Result::EarlyExit(messages) => {
                #[expect(
                    clippy::print_stdout,
                    reason = "This convenience API is specifically responsible for printing \
                              parser output."
                )]
                for message in messages {
                    println!("{message}");
                }
                process::exit(0);
            },
            outcome::Result::Error(errors) => Err(errors),
        };
        (result, remaining)
    }

    /// Parses environment arguments and exits successfully after writing an early-exit message.
    #[inline]
    pub fn parse_from_env_with_early_exit(
        self,
    ) -> (Result<T, NonEmpty<Error<'t, F>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        F: 't,
    {
        self.parse_with_early_exit(env::args_os().skip(1))
    }

    /// Parses arguments completely, reporting an error and exiting on failure.
    #[inline]
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "This public localization API accepts each independently configurable message \
                  explicitly."
    )]
    pub fn parse_complete(
        self,
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> T
    where
        F: Display,
    {
        let (result, remaining) = self.parse(args);
        #[expect(
            clippy::print_stderr,
            reason = "This convenience API is specifically responsible for printing parser errors."
        )]
        if !remaining.is_empty() {
            eprintln!("{unused_arg}");
            process::exit(error_code.get());
        }
        match result {
            outcome::Result::Value(value) => value,
            outcome::Result::EarlyExit(messages) => {
                #[expect(
                    clippy::print_stdout,
                    reason = "This convenience API is specifically responsible for printing \
                              parser output."
                )]
                for message in messages {
                    println!("{message}");
                }
                process::exit(0);
            },
            outcome::Result::Error(errors) => {
                #[expect(
                    clippy::print_stderr,
                    reason = "This convenience API is specifically responsible for printing \
                              parser errors."
                )]
                for error in errors {
                    eprintln!(
                        "{}",
                        error.create_error_message(
                            missing_flag,
                            missing_value,
                            parse_error,
                            invalid_string,
                        )
                    );
                }
                process::exit(error_code.get());
            },
        }
    }

    /// Parses arguments completely using localized messages.
    #[inline]
    #[must_use]
    pub fn parse_complete_with_language(
        self,
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        language: Language,
    ) -> T
    where
        F: Display,
    {
        self.parse_complete(
            args,
            error_code,
            language.missing_flag,
            language.missing_value,
            language.parse_error,
            language.invalid_string,
            language.unused_argument,
        )
    }

    /// Parses arguments completely using English messages.
    #[inline]
    #[must_use]
    pub fn parse_with_error_message(
        self,
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
    ) -> T
    where
        F: Display,
    {
        self.parse_complete_with_language(args, error_code, Language::ENGLISH)
    }

    /// Parses environment arguments completely.
    #[inline]
    #[must_use]
    pub fn parse_complete_from_env(
        self,
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> T
    where
        F: Display,
    {
        self.parse_complete(
            env::args_os().skip(1),
            error_code,
            missing_flag,
            missing_value,
            parse_error,
            invalid_string,
            unused_arg,
        )
    }

    /// Parses environment arguments completely using localized messages.
    #[inline]
    #[must_use]
    pub fn parse_complete_with_language_from_env(
        self,
        error_code: NonZeroI32,
        language: Language,
    ) -> T
    where
        F: Display,
    {
        self.parse_complete_with_language(env::args_os().skip(1), error_code, language)
    }

    /// Parses environment arguments completely using English messages.
    #[inline]
    #[must_use]
    pub fn parse_with_error_message_from_env(self, error_code: NonZeroI32) -> T
    where
        F: Display,
    {
        self.parse_complete_with_language_from_env(error_code, Language::ENGLISH)
    }
}
