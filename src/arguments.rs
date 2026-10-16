//! Definition von akzeptierten Kommandozeilen-Argumenten.

use std::{
    borrow::Cow,
    env,
    ffi::{OsStr, OsString},
    fmt::{self, Debug},
    path::Path,
};

use nonempty::{NonEmpty, nonempty};
use void::Void;

pub(crate) use crate::description::ArgumentInput;
use crate::{
    Description,
    arguments::{
        combine::Combine,
        flag::Flag,
        help::CreateHelpText,
        parsing::{ParsedLongForms, ParsedMergedShortForms, ParsedShortForms, RecognizedArguments},
        single_argument::SingleArgument,
        value::Value,
    },
    language::Language,
    outcome::Result as OutcomeResult,
};

#[cfg(feature = "german")]
pub mod argumente;
pub mod combine;
pub mod early_exit;
pub mod flag;
pub mod help;
pub mod parsing;
pub mod single_argument;
pub mod value;

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
#[expect(
    clippy::large_enum_variant,
    reason = "The alternatives variant is intentionally boxed only where needed for the public \
              representation."
)]
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
    #[inline]
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
    #[inline]
    fn from(argument: SingleArgument<'t, T, Error>) -> Self {
        Self::Single(argument)
    }
}

impl<'t, T, Error> From<Flag<'t, T>> for Arguments<'t, T, Error> {
    #[inline]
    fn from(flag: Flag<'t, T>) -> Self {
        Self::Single(flag.into())
    }
}

impl<'t, T, Error> From<Value<'t, T, Error>> for Arguments<'t, T, Error> {
    #[inline]
    fn from(value: Value<'t, T, Error>) -> Self {
        Self::Single(value.into())
    }
}

impl<T, Error> From<NonEmpty<Self>> for Arguments<'_, T, Error> {
    #[inline]
    fn from(alternatives: NonEmpty<Self>) -> Self {
        Self::Alternatives(Box::new(alternatives))
    }
}

impl<T, Error> From<Box<NonEmpty<Self>>> for Arguments<'_, T, Error> {
    #[inline]
    fn from(alternatives: Box<NonEmpty<Self>>) -> Self {
        Self::Alternatives(alternatives)
    }
}

impl<'t, T, Error> Arguments<'t, T, Error> {
    /// Creates a single-argument variant with suitable type parameters.
    #[inline]
    pub fn single_argument(argument: SingleArgument<'t, T, Error>) -> Self {
        Self::Single(argument)
    }

    /// Creates a combined-arguments variant with suitable type parameters.
    #[inline]
    pub fn combine(combine: impl 't + Combine<'t, T, Error>) -> Self {
        Self::Combined(Box::new(combine))
    }

    /// Creates an alternatives variant with suitable type parameters.
    #[inline]
    pub fn alternatives(alternatives: NonEmpty<Self>) -> Self {
        Self::Alternatives(Box::new(alternatives))
    }
}

/// A type-erased combination whose result errors are converted after parsing.
///
/// The wrapper retains the original parser tree, so it can convert errors from arbitrary
/// user-defined [`Combine`] implementations without requiring a non-object-safe generic method
/// on that trait.
struct ConvertedArguments<'t, T, Error, NewError> {
    /// The parser definition producing the original error type.
    arguments: Arguments<'t, T, Error>,
    /// Converts errors produced by the wrapped definition.
    mapper: Box<dyn 't + Fn(Error) -> NewError>,
}

impl<'t, T, Error, NewError> Combine<'t, T, NewError>
    for ConvertedArguments<'t, T, Error, NewError>
{
    #[inline]
    fn create_help_text(
        &self,
        variant: &dyn CreateHelpText,
        meta_default: &str,
        meta_possible_values: &str,
    ) -> NonEmpty<help::Alternatives> {
        self.arguments.create_help_text(variant, meta_default, meta_possible_values)
    }

    #[inline]
    fn debug_fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("ConvertedArguments").field(&"<parser>").finish()
    }

    #[inline]
    fn parse_merged_short_forms(
        &self,
        args: Box<dyn Iterator<Item = OsString> + '_>,
    ) -> NonEmpty<ParsedMergedShortForms<'t>> {
        self.arguments.parse_merged_short_forms(args)
    }

    #[inline]
    fn parse_merged_short_inputs(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedMergedShortForms<'t>> {
        self.arguments.parse_merged_short_inputs(args)
    }

    #[inline]
    fn parse_short_forms(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedShortForms<'t>> {
        self.arguments.parse_short_forms(args)
    }

    #[inline]
    fn parse_long_forms(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedLongForms<'t>> {
        self.arguments.parse_long_forms(args)
    }

    #[inline]
    fn validate(&self, recognized: RecognizedArguments<'t>) -> OutcomeResult<'t, (), NewError> {
        self.arguments.validate(recognized).convert_error(&*self.mapper)
    }

    #[inline]
    fn accumulate(
        self: Box<Self>,
        recognized: RecognizedArguments<'t>,
    ) -> OutcomeResult<'t, T, NewError> {
        let Self { arguments, mapper } = *self;
        arguments.accumulate(recognized).convert_error(&*mapper)
    }
}

#[expect(
    clippy::multiple_inherent_impl,
    reason = "This conversion implementation is separated from construction and parsing impls."
)]
impl<'t, T, Error> Arguments<'t, T, Error> {
    /// Converts errors with the supplied functions.
    ///
    /// The error-display callback is retained for API compatibility. Error displays are only used
    /// when a value argument is rendered as a string; that operation has no equivalent for an
    /// arbitrary type-erased [`Combine`] implementation.
    #[inline]
    pub fn convert_error<NewError>(
        self,
        mapper: impl 't + Fn(Error) -> NewError + Clone,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> Arguments<'t, T, NewError> {
        match self {
            Self::Single(argument) => {
                Arguments::Single(argument.convert_error(mapper, display_new_error))
            },
            Self::Alternatives(alternatives) => {
                Arguments::Alternatives(Box::new(alternatives.map(|arguments| {
                    arguments.convert_error(mapper.clone(), display_new_error.clone())
                })))
            },
            arguments @ Self::Combined(_) => Arguments::Combined(Box::new(ConvertedArguments {
                arguments,
                mapper: Box::new(mapper),
            })),
        }
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

// TODO can we use `!`, stabilized in rust 1.100 here?
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
            Self::Single(argument) => {
                nonempty![help::Alternatives::Single(variant.create_help_text(
                    argument.as_string_value(),
                    meta_default,
                    meta_possible_values,
                ))]
            },
            Self::Combined(combine) => {
                combine.create_help_text(variant, meta_default, meta_possible_values)
            },
            Self::Alternatives(alternatives) => {
                // Avoid .as_ref() to refer to the Box<> method.
                let alternatives: &NonEmpty<_> = alternatives;
                alternatives.as_ref().map(|argument| {
                    help::Alternatives::Alternatives(Box::new(argument.create_help_text(
                        variant,
                        meta_default,
                        meta_possible_values,
                    )))
                })
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
    #[expect(
        clippy::too_many_arguments,
        reason = "This public localization API accepts each independently configurable message \
                  explicitly."
    )]
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
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "We accept the potential overflow here."
            )]
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
    #[expect(
        clippy::too_many_arguments,
        reason = "This public localization API accepts each independently configurable message \
                  explicitly."
    )]
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
                #[expect(
                    clippy::arithmetic_side_effects,
                    reason = "The widths originate from validated display measurements and are \
                              intentionally combined."
                )]
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
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "The widths originate from validated display measurements and are \
                          intentionally combined."
            )]
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
            #[expect(
                clippy::arithmetic_side_effects,
                reason = "The widths originate from validated display measurements and are \
                          intentionally combined."
            )]
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
