//! English-primary parsing outcomes and errors.

use std::{borrow::Cow, ffi::OsString, fmt::Display, iter};

use either::Either;
use nonempty::NonEmpty;

use crate::{description::Name, language::Language, unicode::Normalized};

/// An intermediate parsing result.
#[derive(Debug)]
#[must_use]
pub enum IntermediateResult<'t, T, E, A> {
    /// A successfully parsed value.
    Value(T),
    /// An early exit with messages to display.
    EarlyExit(NonEmpty<Cow<'t, str>>),
    /// Parse errors.
    Error(NonEmpty<AnnotatedParseError<'t, E>>),
    /// No unambiguous final result is available yet.
    Incomplete(A),
}

/// A parsing result.
#[derive(Debug)]
#[must_use]
pub enum Result<'t, T, E> {
    /// A successfully parsed value.
    Value(T),
    /// An early exit with messages to display.
    EarlyExit(NonEmpty<Cow<'t, str>>),
    /// Parse errors.
    Error(NonEmpty<Error<'t, E>>),
}

/// Errors from each rejected command-line argument alternative.
#[derive(Debug, Clone)]
#[must_use]
pub struct AlternativeErrors<'t, E>(pub NonEmpty<NonEmpty<Error<'t, E>>>);

/// An error encountered while parsing command-line arguments.
#[derive(Debug, Clone)]
pub enum Error<'t, E> {
    /// Errors from each rejected alternative.
    Alternatives(Box<AlternativeErrors<'t, E>>),
    /// A required flag was absent.
    MissingFlag {
        /// All names of the flag.
        name: Name<'t>,
        /// Prefix used for an inverted flag.
        invert_prefix: Normalized<'t>,
        /// Infix following the inversion prefix.
        invert_infix: Normalized<'t>,
    },
    /// A required value argument was absent.
    MissingValue {
        /// All names of the value argument.
        name: Name<'t>,
        /// Infix separating name and value.
        value_infix: Normalized<'t>,
        /// The value metavariable.
        meta_var: &'t str,
    },
    /// A value could not be parsed.
    ParseError(AnnotatedParseError<'t, E>),
}

/// A parse error annotated with its argument metadata.
#[derive(Debug, Clone)]
#[must_use]
pub struct AnnotatedParseError<'t, E> {
    /// All names of the value argument.
    pub name: Name<'t>,
    /// Infix separating name and value.
    pub value_infix: Normalized<'t>,
    /// The value metavariable.
    pub meta_var: &'t str,
    /// The underlying parse error.
    pub error: ParseError<E>,
}

/// An error while converting an operating-system string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError<E> {
    /// The string is not valid UTF-8.
    InvalidString(OsString),
    /// Parsing the UTF-8 string failed.
    ParseError(E),
}

impl<'t, T, E, A> IntermediateResult<'t, T, E, A> {
    /// Converts a successful value.
    #[inline]
    pub fn convert<S>(self, mapper: impl FnOnce(T) -> S) -> IntermediateResult<'t, S, E, A> {
        match self {
            Self::Value(value) => IntermediateResult::Value(mapper(value)),
            Self::EarlyExit(messages) => IntermediateResult::EarlyExit(messages),
            Self::Error(errors) => IntermediateResult::Error(errors),
            Self::Incomplete(value) => IntermediateResult::Incomplete(value),
        }
    }

    /// Converts the embedded errors.
    #[inline]
    pub fn convert_error<F>(self, mapper: impl Fn(E) -> F) -> IntermediateResult<'t, T, F, A> {
        match self {
            Self::Value(value) => IntermediateResult::Value(value),
            Self::EarlyExit(messages) => IntermediateResult::EarlyExit(messages),
            Self::Error(errors) => {
                IntermediateResult::Error(errors.map(|error| error.convert(&mapper)))
            },
            Self::Incomplete(value) => IntermediateResult::Incomplete(value),
        }
    }

    /// Converts an incomplete value.
    #[inline]
    pub fn convert_incomplete<B>(
        self,
        mapper: impl FnOnce(A) -> B,
    ) -> IntermediateResult<'t, T, E, B> {
        match self {
            Self::Value(value) => IntermediateResult::Value(value),
            Self::EarlyExit(messages) => IntermediateResult::EarlyExit(messages),
            Self::Error(errors) => IntermediateResult::Error(errors),
            Self::Incomplete(value) => IntermediateResult::Incomplete(mapper(value)),
        }
    }
}

impl<'t, T, E> Result<'t, T, E> {
    /// Converts a successful value.
    #[inline]
    pub fn convert<S>(self, mapper: impl FnOnce(T) -> S) -> Result<'t, S, E> {
        match self {
            Self::Value(value) => Result::Value(mapper(value)),
            Self::EarlyExit(messages) => Result::EarlyExit(messages),
            Self::Error(errors) => Result::Error(errors),
        }
    }

    /// Converts the embedded errors.
    #[inline]
    pub fn convert_error<F>(self, mapper: impl Fn(E) -> F) -> Result<'t, T, F> {
        match self {
            Self::Value(value) => Result::Value(value),
            Self::EarlyExit(messages) => Result::EarlyExit(messages),
            Self::Error(errors) => Result::Error(errors.map(|error| error.convert(&mapper))),
        }
    }
}

impl<'t, E> Error<'t, E> {
    /// Converts the embedded error.
    #[inline]
    pub fn convert<F>(self, mapper: impl Fn(E) -> F) -> Error<'t, F> {
        self.convert_ref(&mapper)
    }

    /// Converts the embedded error using a reusable mapper.
    fn convert_ref<F>(self, mapper: &impl Fn(E) -> F) -> Error<'t, F> {
        match self {
            Self::Alternatives(alternatives) => Error::Alternatives(Box::new(AlternativeErrors(
                alternatives.0.map(|errors| errors.map(|error| error.convert_ref(mapper))),
            ))),
            Self::MissingFlag { name, invert_prefix, invert_infix } => {
                Error::MissingFlag { name, invert_prefix, invert_infix }
            },
            Self::MissingValue { name, value_infix, meta_var } => {
                Error::MissingValue { name, value_infix, meta_var }
            },
            Self::ParseError(error) => Error::ParseError(error.convert(mapper)),
        }
    }
}
impl<'t, E> AnnotatedParseError<'t, E> {
    /// Converts the embedded error.
    #[inline]
    pub fn convert<F>(self, mapper: impl FnOnce(E) -> F) -> AnnotatedParseError<'t, F> {
        let Self { name, value_infix, meta_var, error } = self;
        AnnotatedParseError { name, value_infix, meta_var, error: error.convert(mapper) }
    }
}
impl<E> ParseError<E> {
    /// Converts the embedded error.
    #[inline]
    pub fn convert<F>(self, mapper: impl FnOnce(E) -> F) -> ParseError<F> {
        match self {
            Self::InvalidString(value) => ParseError::InvalidString(value),
            Self::ParseError(error) => ParseError::ParseError(mapper(error)),
        }
    }
}
/// Appends a regular-expression fragment for the supplied names.
fn names_regex<S: AsRef<str>>(string: &mut String, head: &S, tail: &[S]) {
    if !tail.is_empty() {
        string.push('(');
    }
    for (index, name) in iter::once(head).chain(tail).enumerate() {
        if index != 0 {
            string.push('|');
        }
        string.push_str(name.as_ref());
    }
    if !tail.is_empty() {
        string.push(')');
    }
}

impl<E: Display> Error<'_, E> {
    /// Creates a human-readable error message using English defaults.
    #[inline]
    pub fn error_message(&self) -> String {
        self.create_error_message_with_language(Language::ENGLISH)
    }

    /// Creates a human-readable error message using `language`.
    #[inline]
    pub fn create_error_message_with_language(&self, language: Language) -> String {
        self.create_error_message_with_separator(
            language.missing_flag,
            language.missing_value,
            language.parse_error,
            language.invalid_string,
            language.alternative_error_separator,
        )
    }

    /// Creates a human-readable error message.
    #[inline]
    pub fn create_error_message(
        &self,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
    ) -> String {
        self.create_error_message_with_separator(
            missing_flag,
            missing_value,
            parse_error,
            invalid_string,
            Language::ENGLISH.alternative_error_separator,
        )
    }

    /// Creates a human-readable error message using an alternative error separator.
    fn create_error_message_with_separator(
        &self,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        alternative_error_separator: &str,
    ) -> String {
        fn message(
            description: &str,
            Name { long_prefix, long, short_prefix, short }: &Name<'_>,
            detail: Either<(&Normalized<'_>, &Normalized<'_>), (&Normalized<'_>, &str)>,
        ) -> String {
            let mut result = format!("{description}: {}", long_prefix.as_ref());
            match detail {
                Either::Left((prefix, infix)) => {
                    result.push('[');
                    result.push_str(prefix.as_ref());
                    result.push_str(infix.as_ref());
                    result.push(']');
                    names_regex(&mut result, &long.head, &long.tail);
                },
                Either::Right((infix, meta_var)) => {
                    names_regex(&mut result, &long.head, &long.tail);
                    result.push_str("( |");
                    result.push_str(infix.as_ref());
                    result.push(')');
                    result.push_str(meta_var);
                },
            }
            if let Some((head, tail)) = short.split_first() {
                result.push_str(" | ");
                result.push_str(short_prefix.as_ref());
                names_regex(&mut result, head, tail);
                if let Either::Right((infix, meta_var)) = detail {
                    result.push_str("[ |");
                    result.push_str(infix.as_ref());
                    result.push(']');
                    result.push_str(meta_var);
                }
            }
            result
        }
        match self {
            Self::Alternatives(alternatives) => alternatives
                .0
                .iter()
                .map(|errors| {
                    errors
                        .iter()
                        .map(|error| {
                            error.create_error_message_with_separator(
                                missing_flag,
                                missing_value,
                                parse_error,
                                invalid_string,
                                alternative_error_separator,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .collect::<Vec<_>>()
                .join(&format!("\n{alternative_error_separator}\n")),
            Self::MissingFlag { name, invert_prefix, invert_infix } => {
                message(missing_flag, name, Either::Left((invert_prefix, invert_infix)))
            },
            Self::MissingValue { name, value_infix, meta_var } => {
                message(missing_value, name, Either::Right((value_infix, meta_var)))
            },
            Self::ParseError(AnnotatedParseError { name, value_infix, meta_var, error }) => {
                let (description, displayed) = match error {
                    ParseError::InvalidString(value) => (invalid_string, format!("{value:?}")),
                    ParseError::ParseError(error) => (parse_error, error.to_string()),
                };
                format!(
                    "{}\n{displayed}",
                    message(description, name, Either::Right((value_infix, meta_var)))
                )
            },
        }
    }
}
