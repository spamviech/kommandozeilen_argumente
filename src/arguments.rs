//! Definition von akzeptierten Kommandozeilen-Argumenten.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    env,
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
    num::NonZeroI32,
    path::Path,
    process,
};

use itertools::Itertools as _;
use nonempty::{NonEmpty, nonempty};
use void::Void;

use crate::{
    Description,
    arguments::{
        combine::Combine, flag::Flag, help::CreateHelpText, single_argument::SingleArgument,
        value::Value,
    },
    description::ArgumentInput,
    language::Language,
    outcome::Error,
};

pub mod argumente;
pub mod combine;
pub mod early_exit;
pub mod flag;
pub mod help;
pub mod single_argument;
pub mod value;

#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
// TODO Name/Version für Hilfetext angeben, als alternative für macros (derive-Feature)
// TODO Unterbefehle/subcommands
// TODO Positions-basierte Argumente

/// Helper to use [`Combine::debug_fmt`] with [`fmt::Formatter::debug_tuple`].
struct CombineDebug<'s, 't, T, Error>(&'s dyn Combine<'t, T, Error>);

impl<T, Error> Debug for CombineDebug<'_, '_, T, Error> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.debug_fmt(formatter)
    }
}

/// Configuration of command-line arguments.
///
/// ## Deutsches Synonym
/// [`Argumente`](argumente::Argumente)
#[allow(clippy::large_enum_variant, clippy::module_name_repetitions)]
#[must_use]
pub enum Arguments<'t, T, Error> {
    /// A single argument.
    Single(SingleArgument<'t, T, Error>),
    /// A combination of multiple arguments.
    Combined(Box<dyn 't + Combine<'t, T, Error>>),
    /// Alternative command-line arguments.
    Alternatives(Box<NonEmpty<Self>>),
}

impl<T: Debug, Error: Debug> Debug for Arguments<'_, T, Error> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(argument) => formatter.debug_tuple("Single").field(argument).finish(),
            Self::Combined(combine) => {
                formatter.debug_tuple("Combined").field(&CombineDebug(&**combine)).finish()
            },
            Self::Alternatives(alternatives) => {
                formatter.debug_tuple("Alternatives").field(alternatives).finish()
            },
        }
    }
}

impl<'t, T, Error> From<SingleArgument<'t, T, Error>> for Arguments<'t, T, Error> {
    fn from(argument: SingleArgument<'t, T, Error>) -> Self {
        Self::Single(argument)
    }
}

impl<'t, T, Error> From<Flag<'t, T>> for Arguments<'t, T, Error> {
    fn from(flag: Flag<'t, T>) -> Self {
        Self::Single(flag.into())
    }
}

impl<'t, T, Error> From<Value<'t, T, Error>> for Arguments<'t, T, Error> {
    fn from(value: Value<'t, T, Error>) -> Self {
        Self::Single(value.into())
    }
}

impl<T, Error> From<NonEmpty<Self>> for Arguments<'_, T, Error> {
    fn from(alternatives: NonEmpty<Self>) -> Self {
        Self::Alternatives(Box::new(alternatives))
    }
}

impl<T, Error> From<Box<NonEmpty<Self>>> for Arguments<'_, T, Error> {
    fn from(alternatives: Box<NonEmpty<Self>>) -> Self {
        Self::Alternatives(alternatives)
    }
}

impl<'t, T, Error> Arguments<'t, T, Error> {
    /// Creates a single-argument variant with suitable type parameters.
    pub fn single_argument(argument: SingleArgument<'t, T, Error>) -> Self {
        Self::Single(argument)
    }

    /// Creates a combined-arguments variant with suitable type parameters.
    pub fn combine(combine: impl 't + Combine<'t, T, Error>) -> Self {
        Self::Combined(Box::new(combine))
    }

    /// Creates an alternatives variant with suitable type parameters.
    pub fn alternatives(alternatives: NonEmpty<Self>) -> Self {
        Self::Alternatives(Box::new(alternatives))
    }

    /// Creates a boxed alternatives variant with suitable type parameters.
    pub fn alternatives_boxed(alternatives: Box<NonEmpty<Self>>) -> Self {
        Self::Alternatives(alternatives)
    }
}

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

/// Output of stage 1, merged short-name recognition.
#[derive(Debug, Clone)]
pub struct ParsedMergedShortForms<'argument> {
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
}

/// Output of stage 2, standalone short-name recognition.
#[derive(Debug, Clone)]
pub struct ParsedShortForms<'argument> {
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
}

/// Output of stage 3, long-name recognition.
#[derive(Debug, Clone)]
pub struct ParsedLongForms<'argument> {
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
}

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
}

impl<'argument> ParsedMergedShortForms<'argument> {
    pub(crate) fn empty() -> Self {
        Self {
            early_exits: Vec::new(), flags: Vec::new(), missing_values: HashSet::new(),
            values: HashMap::new(), remaining: Vec::new(),
        }
    }

    pub(crate) fn merge(mut self, mut next: Self) -> Self {
        self.early_exits.append(&mut next.early_exits);
        self.flags.append(&mut next.flags);
        for name in next.missing_values { if !self.values.contains_key(&name) { let _ = self.missing_values.insert(name); } }
        for (name, value) in next.values { let _ = self.missing_values.remove(&name); let _ = self.values.insert(name, value); }
        self.remaining = next.remaining;
        self
    }
}

impl<'argument> ParsedShortForms<'argument> {
    pub(crate) fn empty() -> Self { Self { early_exits: Vec::new(), flags: Vec::new(), missing_values: HashSet::new(), values: HashMap::new(), remaining: Vec::new() } }
    pub(crate) fn merge(mut self, mut next: Self) -> Self {
        self.early_exits.append(&mut next.early_exits); self.flags.append(&mut next.flags);
        for name in next.missing_values { if !self.values.contains_key(&name) { let _ = self.missing_values.insert(name); } }
        for (name, value) in next.values { let _ = self.missing_values.remove(&name); let _ = self.values.insert(name, value); }
        self.remaining = next.remaining; self
    }
}

impl<'argument> ParsedLongForms<'argument> {
    pub(crate) fn empty() -> Self { Self { early_exits: Vec::new(), flags: Vec::new(), missing_values: HashSet::new(), values: HashMap::new(), remaining: Vec::new() } }
    pub(crate) fn merge(mut self, mut next: Self) -> Self {
        self.early_exits.append(&mut next.early_exits); self.flags.append(&mut next.flags);
        for name in next.missing_values { if !self.values.contains_key(&name) { let _ = self.missing_values.insert(name); } }
        for (name, value) in next.values { let _ = self.missing_values.remove(&name); let _ = self.values.insert(name, value); }
        self.remaining = next.remaining; self
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
            Self::Single(argument) => {
                let args = args.collect_vec();
                let mut result = argument.parse_merged_short_forms(args.iter().filter_map(|argument| match argument {
                    ArgumentInput::Unchanged(argument) => Some(argument.clone()),
                    ArgumentInput::AdjustedMergedShortNames(_) => None,
                }));
                result.remaining.extend(args.into_iter().filter(|argument| {
                    matches!(argument, ArgumentInput::AdjustedMergedShortNames(_))
                }));
                NonEmpty::singleton(result)
            },
            Self::Combined(combine) => combine.parse_merged_short_inputs(Box::new(args)),
            Self::Alternatives(alternatives) => {
                let args = args.collect_vec();
                NonEmpty::collect(
                    alternatives.iter().flat_map(|argument| {
                        argument.parse_merged_short_inputs(args.iter().cloned())
                    }),
                )
                .expect("Iterator of NonEmpty<NonEmpty<_>>.")
            },
        }
    }

    /// Parses standalone short-form arguments after merged short forms.
    #[inline]
    pub fn parse_short_forms(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> NonEmpty<ParsedShortForms<'t>> {
        match self {
            Self::Single(argument) => NonEmpty::singleton(argument.parse_short_form(args)),
            Self::Combined(combine) => combine.parse_short_forms(Box::new(args)),
            Self::Alternatives(alternatives) => {
                let args = args.collect_vec();
                NonEmpty::collect(alternatives.iter().flat_map(|argument| {
                    argument.parse_short_forms(args.iter().cloned())
                })).expect("Iterator of NonEmpty<NonEmpty<_>>.")
            },
        }
    }

    /// Parses long-form arguments after standalone short forms.
    #[inline]
    pub fn parse_long_forms(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> NonEmpty<ParsedLongForms<'t>> {
        match self {
            Self::Single(argument) => NonEmpty::singleton(argument.parse_long_form(args)),
            Self::Combined(combine) => combine.parse_long_forms(Box::new(args)),
            Self::Alternatives(alternatives) => {
                let args = args.collect_vec();
                NonEmpty::collect(alternatives.iter().flat_map(|argument| {
                    argument.parse_long_forms(args.iter().cloned())
                })).expect("Iterator of NonEmpty<NonEmpty<_>>.")
            },
        }
    }
}

impl<'t, T, F> Arguments<'t, T, F> {
    /// Parses the supplied arguments and produces the corresponding value.
    #[inline]
    pub fn parse(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> (crate::outcome::Result<'t, T, F>, Vec<ArgumentInput>) {
        todo!("parse({:?})", args.collect::<Vec<_>>());
    }

    /// Parses arguments from the environment.
    #[inline]
    pub fn parse_from_env(self) -> (crate::outcome::Result<'t, T, F>, Vec<ArgumentInput>)
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
            crate::outcome::Result::Value(value) => Ok(value),
            crate::outcome::Result::EarlyExit(messages) => {
                #[allow(clippy::print_stdout)]
                for message in messages {
                    println!("{message}");
                }
                process::exit(0);
            },
            crate::outcome::Result::Error(errors) => Err(errors),
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
    #[allow(clippy::too_many_arguments)]
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
        #[allow(clippy::print_stderr)]
        if !remaining.is_empty() {
            eprintln!("{unused_arg}");
            process::exit(error_code.get());
        }
        match result {
            crate::outcome::Result::Value(value) => value,
            crate::outcome::Result::EarlyExit(messages) => {
                #[allow(clippy::print_stdout)]
                for message in messages {
                    println!("{message}");
                }
                process::exit(0);
            },
            crate::outcome::Result::Error(errors) => {
                #[allow(clippy::print_stderr)]
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
    #[allow(clippy::too_many_arguments)]
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

impl<'t, T, Error> Arguments<'t, T, Error> {
    /// Converts errors with the supplied functions.
    #[inline]
    pub fn convert_error<NewError>(
        self,
        mapper: impl 't + Fn(Error) -> NewError + Clone,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> Arguments<'t, T, NewError> {
        todo!()
    }

    /// Converts errors using [`From::from`].
    #[inline]
    pub fn error_from<NewError: From<Error>>(
        self,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> Arguments<'t, T, NewError> {
        self.convert_error(NewError::from, display_new_error)
    }
}

impl<'t, T> Arguments<'t, T, Void> {
    /// Converts the infallible error type using [`void::unreachable`].
    #[inline]
    pub fn error_from_void<NewError>(
        self,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> Arguments<'t, T, NewError> {
        self.convert_error(|value| void::unreachable(value), display_new_error)
    }
}

impl<T, Error> Arguments<'_, T, Error> {
    /// Creates syntax and help text for this argument.
    #[inline]
    pub fn create_help_text(
        &self,
        variant: &dyn CreateHelpText,
        meta_default: &str,
        meta_possible_values: &str,
    ) -> NonEmpty<help::Alternatives> {
        match self {
            Self::Single(argument) => nonempty![help::Alternatives::Single(
                variant
                    .create_help_text(
                        argument.as_string_value().into(),
                        meta_default,
                        meta_possible_values,
                    )
                    .into()
            )],
            Self::Combined(combine) => combine
                .create_help_text(variant, meta_default, meta_possible_values)
                .map(Into::into),
            Self::Alternatives(alternatives) => {
                // TODO use alternatives.as_ref().flat_map(...), coming in nonempty > 0.10.0
                NonEmpty::collect(alternatives.iter().map(|argument| {
                    help::Alternatives::Alternatives(Box::new(argument.create_help_text(
                        variant,
                        meta_default,
                        meta_possible_values,
                    )))
                }))
                .expect("NonEmpty::map(...) has at least one argument!")
            },
        }
    }
}

impl<'t, T: Debug, Error: Debug> Arguments<'t, T, Error> {
    /// Adds an [`EarlyExit`](early_exit::EarlyExit)-flag that shows the program version.
    #[inline]
    pub fn with_version_early_exit(
        self,
        arg_description: Description<'t, Void>,
        program_name: &str,
        program_version: &str,
    ) -> Self {
        let message = format!("{program_name} {program_version}");
        let early_exit =
            early_exit::EarlyExit { description: arg_description, message: Cow::Owned(message) };
        let early_exit_argument: Arguments<'t, (), Error> =
            Arguments::from(SingleArgument::from(early_exit));
        Self::combine((|value: T, ()| value, self, early_exit_argument))
    }

    /// Adds a version early-exit flag using the supplied language.
    #[inline]
    pub fn with_version_early_exit_with_language(
        self,
        program_name: &str,
        program_version: &str,
        language: Language,
    ) -> Self {
        self.with_version_early_exit(
            Description::new_with_language(
                language.version_long,
                language.version_short,
                Some(language.version_description),
                None,
                language,
            ),
            program_name,
            program_version,
        )
    }

    /// Add an [`EarlyExit`](early_exit::EarlyExit)-flag showing the help text for all arguments.
    ///
    /// ### Panics
    /// If the syntax-description (including normal + alternativ prefixes) for an argument exceeds
    /// [`usize::MAX`].
    ///
    /// ## Deutsches Synonym
    /// [`mit_hilfe_frühes_beenden`](argumente::Argumente::mit_hilfe_frühes_beenden)
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn with_help_early_exit(
        self,
        variant: &dyn CreateHelpText,
        arg_description: Description<'t, Void>,
        program_name: &str,
        program_description: Option<&str>,
        program_version: Option<&str>,
        meta_standard: &str,
        meta_possible_values: &str,
        meta_options: &str,
        meta_syntax_prefix: &str,
        meta_syntax_padding: char,
        meta_alternative_prefix: &str,
        meta_alternative_separator: char,
    ) -> Self {
        let mut helps = self.create_help_text(variant, meta_standard, meta_possible_values);
        let mut early_exit =
            early_exit::EarlyExit { description: arg_description, message: Cow::Borrowed("") };
        helps.push(help::Alternatives::Single(early_exit.create_help_text()));
        let max_syntax_width = max_syntax_width(&helps, meta_alternative_prefix);
        let current_exe = env::current_exe().ok();
        let exe_name = current_exe
            .as_deref()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .unwrap_or(program_name);
        let mut name = String::from(program_name);
        if let Some(version) = program_version {
            name.push(' ');
            name.push_str(version);
        }
        let program_description =
            program_description.map(|description| format!("\n{description}")).unwrap_or_default();
        let mut help_text = format!(
            "{name}{program_description}\n\n{exe_name} [{meta_options}]\n\n{meta_options}:\n"
        );
        for help in helps {
            write_argument_or_alternatives(
                &mut help_text,
                Cow::Borrowed(meta_syntax_prefix),
                meta_syntax_prefix.len() + max_syntax_width + 1,
                meta_syntax_padding,
                &help,
                meta_alternative_prefix,
                meta_alternative_separator,
            );
        }
        early_exit.message = Cow::Owned(help_text);
        let early_exit_argument: Arguments<'t, (), Error> =
            Arguments::from(SingleArgument::from(early_exit));
        Self::combine((|value: T, ()| value, self, early_exit_argument))
    }

    /// Add [`EarlyExit`](crate::arguments::early_exit::EarlyExit)-flags, showing the program
    /// version, or the help text for all arguments.
    ///
    /// ### Panics
    /// If the syntax-description (including normal + alternativ prefixes) for an argument exceeds
    /// [`usize::MAX`].
    ///
    /// ## Deutsches Synonym
    /// [`mit_hilfe_und_version_frühes_beenden`](argumente::Argumente::mit_hilfe_und_version_frühes_beenden)
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn with_help_and_version_early_exit(
        self,
        variant: &dyn CreateHelpText,
        version_description: Description<'t, Void>,
        help_description: Description<'t, Void>,
        program_name: &str,
        program_description: Option<&str>,
        program_version: &str,
        meta_standard: &str,
        meta_possible_values: &str,
        meta_options: &str,
        meta_syntax_prefix: &str,
        meta_syntax_padding: char,
        meta_alternative_prefix: &str,
        meta_alternative_separator: char,
    ) -> Self {
        self.with_version_early_exit(version_description, program_name, program_version)
            .with_help_early_exit(
                variant,
                help_description,
                program_name,
                program_description,
                Some(program_version),
                meta_standard,
                meta_possible_values,
                meta_options,
                meta_syntax_prefix,
                meta_syntax_padding,
                meta_alternative_prefix,
                meta_alternative_separator,
            )
    }

    /// Variant of [`with_help_early_exit`](Self::with_help_early_exit)
    /// based on a [`Language`].
    ///
    /// ## Deutsches Synonym
    /// [`mit_hilfe_frühes_beenden_mit_sprache`](argumente::Argumente::mit_hilfe_frühes_beenden_mit_sprache).
    #[inline]
    pub fn with_help_early_exit_with_language(
        self,
        variant: &dyn CreateHelpText,
        program_name: &str,
        program_beschreibung: Option<&str>,
        program_version: Option<&str>,
        language: Language,
    ) -> Self {
        self.with_help_early_exit(
            variant,
            Description::new_with_language(
                language.help_long,
                language.help_short,
                Some(language.help_description),
                None,
                language,
            ),
            program_name,
            program_beschreibung,
            program_version,
            language.default,
            language.allowed_values,
            language.options,
            language.syntax_prefix,
            language.syntax_padding,
            language.alternative_prefix,
            language.alternative_separator,
        )
    }

    /// Variant of [`with_help_early_exit`](Self::with_help_early_exit)
    /// and [`with_version_early_exit`](Self::with_version_early_exit),
    /// based on a [`Language`].
    ///
    /// ## Deutsches Synonym
    /// [`mit_hilfe_und_version_frühes_beenden_mit_sprache`](argumente::Argumente::mit_hilfe_und_version_frühes_beenden_mit_sprache).
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn with_help_and_version_early_exit_with_language(
        self,
        variant: &dyn CreateHelpText,
        program_name: &str,
        program_description: Option<&str>,
        program_version: &str,
        language: Language,
    ) -> Self {
        self.with_help_and_version_early_exit(
            variant,
            Description::new_with_language(
                language.version_long,
                language.version_short,
                Some(language.version_description),
                None,
                language,
            ),
            Description::new_with_language(
                language.help_long,
                language.help_short,
                Some(language.help_description),
                None,
                language,
            ),
            program_name,
            program_description,
            program_version,
            language.default,
            language.allowed_values,
            language.options,
            language.syntax_prefix,
            language.syntax_padding,
            language.alternative_prefix,
            language.alternative_separator,
        )
    }
}

/// Calculates the maximum width of an argument syntax.
///
/// Helper for `Arguments::with_help_early_exit`.
fn max_syntax_width(helps: &NonEmpty<help::Alternatives>, alternative_prefix: &str) -> usize {
    helps
        .iter()
        .filter_map(|argument| match argument {
            help::Alternatives::Single(argument) => Some(argument.syntax.len()),
            help::Alternatives::Alternatives(alternatives) => {
                #[allow(clippy::arithmetic_side_effects)]
                let width =
                    alternative_prefix.len() + max_syntax_width(alternatives, alternative_prefix);
                Some(width)
            },
            help::Alternatives::Empty => None,
        })
        .max()
        .expect("NonEmpty")
}

/// Writes help text for an entry or all of its alternatives.
///
/// # Panics
/// Panics if `max_syntax_width` is too small for an entry's prefix and syntax.
fn write_argument_or_alternatives(
    output: &mut String,
    current_prefix: Cow<'_, str>,
    max_syntax_width: usize,
    syntax_padding: char,
    entry: &help::Alternatives,
    alternative_prefix: &str,
    alternative_separator: char,
) {
    match entry {
        help::Alternatives::Single(argument) => {
            let help::Help { syntax, help } = argument;
            output.push_str(&current_prefix);
            output.push_str(syntax);
            #[allow(clippy::arithmetic_side_effects)]
            let padding = max_syntax_width - current_prefix.len() - syntax.len();
            let mut buffer: [u8; 4] = [0; 4];
            let padding_string = syntax_padding.encode_utf8(&mut buffer).repeat(padding);
            output.push_str(&padding_string);
            if let Some(help) = help {
                output.push_str(help);
            }
            output.push('\n');
        },
        help::Alternatives::Alternatives(alternatives) => {
            #[allow(clippy::arithmetic_side_effects)]
            let separator_width = max_syntax_width - current_prefix.len();
            let mut buffer: [u8; 4] = [0; 4];
            let separator = format!(
                "{current_prefix}{}",
                alternative_separator.encode_utf8(&mut buffer).repeat(separator_width)
            );
            let mut next_prefix = current_prefix.into_owned();
            next_prefix.push_str(alternative_prefix);
            let mut first = true;
            for alternative in alternatives.iter() {
                if first {
                    first = false;
                } else {
                    output.push_str(&separator);
                    output.push('\n');
                }
                write_argument_or_alternatives(
                    output,
                    Cow::Borrowed(&next_prefix),
                    max_syntax_width,
                    syntax_padding,
                    alternative,
                    alternative_prefix,
                    alternative_separator,
                );
            }
        },
        help::Alternatives::Empty => {},
    }
}
