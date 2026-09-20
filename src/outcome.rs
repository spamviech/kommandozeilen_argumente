//! Results and errors produced while parsing command-line arguments.

use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
    iter, result,
};

use either::Either;
use nonempty::NonEmpty;

use crate::{
    description::{AdjustedMergedShortNames, Name},
    language::{Language, Sprache},
    unicode::Normalized,
};

/// Result of parsing one argument.
pub enum SingeArgResult<'t, T, E> {
    /// The argument was completely parsed.
    FullParse {
        /// The current argument after removing the parsed short name.
        adjusted_arg: Option<AdjustedMergedShortNames>,
        /// The parsing result.
        result: result::Result<T, ParseError<E>>,
    },
    /// Only the argument name was parsed; the next argument is its value.
    NameOnly {
        /// The current argument after removing the parsed short name.
        adjusted_arg: Option<AdjustedMergedShortNames>,
        /// Parses the directly following argument as the value.
        parse_next_arg: Box<dyn 't + Fn(&OsStr) -> SingeArgResult<'t, T, E>>,
    },
    /// More input is required.
    IncompleteParse {
        /// The current argument after removing the parsed short name.
        adjusted_arg: Option<AdjustedMergedShortNames>,
        /// Parses a following argument in this parsing context.
        parse_following_arg: Box<dyn 't + Fn(&OsStr) -> Option<SingeArgResult<'t, T, E>>>,
    },
}

impl<T: Debug, E: Debug> Debug for SingeArgResult<'_, T, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FullParse { adjusted_arg, result } => {
                formatter.debug_tuple("FullParse").field(adjusted_arg).field(result).finish()
            },
            Self::NameOnly { adjusted_arg, .. } => formatter
                .debug_struct("NameOnly")
                .field("adjusted_arg", adjusted_arg)
                .field("parse_next_arg", &"<closure>")
                .finish(),
            Self::IncompleteParse { adjusted_arg, .. } => formatter
                .debug_struct("IncompleteParse")
                .field("adjusted_arg", adjusted_arg)
                .field("parse_following_arg", &"<closure>")
                .finish(),
        }
    }
}

impl<'t, T: 't, E: 't> SingeArgResult<'t, T, E> {
    /// Converts a successful value.
    pub fn convert<S>(self, mapper: impl 't + Clone + Fn(T) -> S) -> SingeArgResult<'t, S, E> {
        match self {
            Self::FullParse { adjusted_arg, result } => {
                SingeArgResult::FullParse { adjusted_arg, result: result.map(mapper) }
            },
            Self::NameOnly { adjusted_arg, parse_next_arg } => SingeArgResult::NameOnly {
                adjusted_arg,
                parse_next_arg: Box::new(move |arg| parse_next_arg(arg).convert(mapper.clone())),
            },
            Self::IncompleteParse { adjusted_arg, parse_following_arg } => {
                SingeArgResult::IncompleteParse {
                    adjusted_arg,
                    parse_following_arg: Box::new(move |arg| {
                        parse_following_arg(arg).map(|value| value.convert(mapper.clone()))
                    }),
                }
            },
        }
    }
}

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

/// An error encountered while parsing command-line arguments.
#[derive(Debug, Clone)]
pub enum Error<'t, E> {
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
    pub fn convert<S>(self, mapper: impl FnOnce(T) -> S) -> IntermediateResult<'t, S, E, A> {
        match self {
            Self::Value(value) => IntermediateResult::Value(mapper(value)),
            Self::EarlyExit(messages) => IntermediateResult::EarlyExit(messages),
            Self::Error(errors) => IntermediateResult::Error(errors),
            Self::Incomplete(value) => IntermediateResult::Incomplete(value),
        }
    }
    /// Converts the embedded errors.
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
    pub fn convert<S>(self, mapper: impl FnOnce(T) -> S) -> Result<'t, S, E> {
        match self {
            Self::Value(value) => Result::Value(mapper(value)),
            Self::EarlyExit(messages) => Result::EarlyExit(messages),
            Self::Error(errors) => Result::Error(errors),
        }
    }
    /// Converts the embedded errors.
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
    pub fn convert<F>(self, mapper: impl FnOnce(E) -> F) -> Error<'t, F> {
        match self {
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
    pub fn convert<F>(self, mapper: impl FnOnce(E) -> F) -> AnnotatedParseError<'t, F> {
        let Self { name, value_infix, meta_var, error } = self;
        AnnotatedParseError { name, value_infix, meta_var, error: error.convert(mapper) }
    }
}
impl<E> ParseError<E> {
    /// Converts the embedded error.
    pub fn convert<F>(self, mapper: impl FnOnce(E) -> F) -> ParseError<F> {
        match self {
            Self::InvalidString(value) => ParseError::InvalidString(value),
            Self::ParseError(error) => ParseError::ParseError(mapper(error)),
        }
    }
}

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
    pub fn error_message(&self) -> String {
        self.create_error_message_with_language(Language::ENGLISH)
    }
    /// Creates a human-readable error message using `language`.
    pub fn create_error_message_with_language(&self, language: Language) -> String {
        self.create_error_message(
            language.missing_flag,
            language.missing_value,
            language.parse_error,
            language.invalid_string,
        )
    }
    /// Creates a human-readable error message.
    pub fn create_error_message(
        &self,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
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

/// Deutsches Spiegelbild von [`IntermediateResult`].
#[derive(Debug)]
#[must_use]
pub enum ZwischenErgebnis<'t, T, E, A> {
    Wert(T),
    FrühesBeenden(NonEmpty<Cow<'t, str>>),
    Fehler(NonEmpty<KommentierterParseFehler<'t, E>>),
    Incomplete(A),
}
/// Deutsches Spiegelbild von [`Result`].
#[derive(Debug)]
#[must_use]
pub enum Ergebnis<'t, T, E> {
    Wert(T),
    FrühesBeenden(NonEmpty<Cow<'t, str>>),
    Fehler(NonEmpty<Fehler<'t, E>>),
}
/// Deutsches Spiegelbild von [`Error`].
#[derive(Debug, Clone)]
pub enum Fehler<'t, E> {
    FehlendeFlag {
        name: Name<'t>,
        invertiere_präfix: Normalized<'t>,
        invertiere_infix: Normalized<'t>,
    },
    FehlenderWert {
        name: Name<'t>,
        wert_infix: Normalized<'t>,
        meta_var: &'t str,
    },
    ParseFehler(KommentierterParseFehler<'t, E>),
}
/// Deutsches Spiegelbild von [`AnnotatedParseError`].
#[derive(Debug, Clone)]
#[must_use]
pub struct KommentierterParseFehler<'t, E> {
    pub name: Name<'t>,
    pub wert_infix: Normalized<'t>,
    pub meta_var: &'t str,
    pub fehler: ParseFehler<E>,
}
/// Deutsches Spiegelbild von [`ParseError`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFehler<E> {
    InvaliderString(OsString),
    ParseFehler(E),
}

impl<'t, T, E, A> From<IntermediateResult<'t, T, E, A>> for ZwischenErgebnis<'t, T, E, A> {
    fn from(value: IntermediateResult<'t, T, E, A>) -> Self {
        match value {
            IntermediateResult::Value(value) => Self::Wert(value),
            IntermediateResult::EarlyExit(value) => Self::FrühesBeenden(value),
            IntermediateResult::Error(value) => Self::Fehler(value.map(Into::into)),
            IntermediateResult::Incomplete(value) => Self::Incomplete(value),
        }
    }
}
impl<'t, T, E, A> From<ZwischenErgebnis<'t, T, E, A>> for IntermediateResult<'t, T, E, A> {
    fn from(value: ZwischenErgebnis<'t, T, E, A>) -> Self {
        match value {
            ZwischenErgebnis::Wert(value) => Self::Value(value),
            ZwischenErgebnis::FrühesBeenden(value) => Self::EarlyExit(value),
            ZwischenErgebnis::Fehler(value) => Self::Error(value.map(Into::into)),
            ZwischenErgebnis::Incomplete(value) => Self::Incomplete(value),
        }
    }
}
impl<'t, T, E> From<Result<'t, T, E>> for Ergebnis<'t, T, E> {
    fn from(value: Result<'t, T, E>) -> Self {
        match value {
            Result::Value(value) => Self::Wert(value),
            Result::EarlyExit(value) => Self::FrühesBeenden(value),
            Result::Error(value) => Self::Fehler(value.map(Into::into)),
        }
    }
}
impl<'t, T, E> From<Ergebnis<'t, T, E>> for Result<'t, T, E> {
    fn from(value: Ergebnis<'t, T, E>) -> Self {
        match value {
            Ergebnis::Wert(value) => Self::Value(value),
            Ergebnis::FrühesBeenden(value) => Self::EarlyExit(value),
            Ergebnis::Fehler(value) => Self::Error(value.map(Into::into)),
        }
    }
}
impl<'t, E> From<Error<'t, E>> for Fehler<'t, E> {
    fn from(value: Error<'t, E>) -> Self {
        match value {
            Error::MissingFlag { name, invert_prefix, invert_infix } => Self::FehlendeFlag {
                name,
                invertiere_präfix: invert_prefix,
                invertiere_infix: invert_infix,
            },
            Error::MissingValue { name, value_infix, meta_var } => {
                Self::FehlenderWert { name, wert_infix: value_infix, meta_var }
            },
            Error::ParseError(value) => Self::ParseFehler(value.into()),
        }
    }
}
impl<'t, E> From<Fehler<'t, E>> for Error<'t, E> {
    fn from(value: Fehler<'t, E>) -> Self {
        match value {
            Fehler::FehlendeFlag { name, invertiere_präfix, invertiere_infix } => {
                Self::MissingFlag {
                    name,
                    invert_prefix: invertiere_präfix,
                    invert_infix: invertiere_infix,
                }
            },
            Fehler::FehlenderWert { name, wert_infix, meta_var } => {
                Self::MissingValue { name, value_infix: wert_infix, meta_var }
            },
            Fehler::ParseFehler(value) => Self::ParseError(value.into()),
        }
    }
}
impl<'t, E> From<AnnotatedParseError<'t, E>> for KommentierterParseFehler<'t, E> {
    fn from(value: AnnotatedParseError<'t, E>) -> Self {
        Self {
            name: value.name,
            wert_infix: value.value_infix,
            meta_var: value.meta_var,
            fehler: value.error.into(),
        }
    }
}
impl<'t, E> From<KommentierterParseFehler<'t, E>> for AnnotatedParseError<'t, E> {
    fn from(value: KommentierterParseFehler<'t, E>) -> Self {
        Self {
            name: value.name,
            value_infix: value.wert_infix,
            meta_var: value.meta_var,
            error: value.fehler.into(),
        }
    }
}
impl<E> From<ParseError<E>> for ParseFehler<E> {
    fn from(value: ParseError<E>) -> Self {
        match value {
            ParseError::InvalidString(value) => Self::InvaliderString(value),
            ParseError::ParseError(value) => Self::ParseFehler(value),
        }
    }
}
impl<E> From<ParseFehler<E>> for ParseError<E> {
    fn from(value: ParseFehler<E>) -> Self {
        match value {
            ParseFehler::InvaliderString(value) => Self::InvalidString(value),
            ParseFehler::ParseFehler(value) => Self::ParseError(value),
        }
    }
}

impl<'t, T, E, A> ZwischenErgebnis<'t, T, E, A> {
    pub fn konvertiere<S>(self, mapper: impl FnOnce(T) -> S) -> ZwischenErgebnis<'t, S, E, A> {
        IntermediateResult::from(self).convert(mapper).into()
    }
    pub fn konvertiere_fehler<F>(self, mapper: impl Fn(E) -> F) -> ZwischenErgebnis<'t, T, F, A> {
        IntermediateResult::from(self).convert_error(mapper).into()
    }
    pub fn konvertiere_incomplete<B>(
        self,
        mapper: impl FnOnce(A) -> B,
    ) -> ZwischenErgebnis<'t, T, E, B> {
        IntermediateResult::from(self).convert_incomplete(mapper).into()
    }
}
impl<'t, T, E> Ergebnis<'t, T, E> {
    pub fn konvertiere<S>(self, mapper: impl FnOnce(T) -> S) -> Ergebnis<'t, S, E> {
        Result::from(self).convert(mapper).into()
    }
    pub fn konvertiere_fehler<F>(self, mapper: impl Fn(E) -> F) -> Ergebnis<'t, T, F> {
        Result::from(self).convert_error(mapper).into()
    }
}
impl<'t, E> Fehler<'t, E> {
    pub fn konvertiere<F>(self, mapper: impl FnOnce(E) -> F) -> Fehler<'t, F> {
        Error::from(self).convert(mapper).into()
    }
}
impl<'t, E> KommentierterParseFehler<'t, E> {
    pub fn konvertiere<F>(self, mapper: impl FnOnce(E) -> F) -> KommentierterParseFehler<'t, F> {
        AnnotatedParseError::from(self).convert(mapper).into()
    }
}
impl<E> ParseFehler<E> {
    pub fn konvertiere<F>(self, mapper: impl FnOnce(E) -> F) -> ParseFehler<F> {
        ParseError::from(self).convert(mapper).into()
    }
}
impl<E: Display> Fehler<'_, E> {
    pub fn fehlermeldung(&self) -> String {
        self.erstelle_fehlermeldung_mit_sprache(Sprache::DEUTSCH)
    }
    pub fn erstelle_fehlermeldung_mit_sprache(&self, sprache: Sprache) -> String {
        self.erstelle_fehlermeldung(
            sprache.fehlende_flag,
            sprache.fehlender_wert,
            sprache.parse_fehler,
            sprache.invalider_string,
        )
    }
    pub fn erstelle_fehlermeldung(
        &self,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
    ) -> String {
        match self {
            Self::FehlendeFlag { name, invertiere_präfix, invertiere_infix } => {
                Error::<E>::MissingFlag {
                    name: name.clone(),
                    invert_prefix: invertiere_präfix.clone(),
                    invert_infix: invertiere_infix.clone(),
                }
                .create_error_message(
                    fehlende_flag,
                    fehlender_wert,
                    parse_fehler,
                    invalider_string,
                )
            },
            Self::FehlenderWert { name, wert_infix, meta_var } => Error::<E>::MissingValue {
                name: name.clone(),
                value_infix: wert_infix.clone(),
                meta_var,
            }
            .create_error_message(fehlende_flag, fehlender_wert, parse_fehler, invalider_string),
            Self::ParseFehler(KommentierterParseFehler { fehler, .. }) => match fehler {
                ParseFehler::InvaliderString(value) => format!("{invalider_string}: {value:?}"),
                ParseFehler::ParseFehler(error) => format!("{parse_fehler}: {error}"),
            },
        }
    }
}
