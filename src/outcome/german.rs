//! German mirror types for parsing outcomes and errors.

use std::{borrow::Cow, ffi::OsString, fmt::Display};

use nonempty::NonEmpty;

use super::{
    AlternativeErrors, AnnotatedParseError, Error, IntermediateResult, ParseError, Result,
};
use crate::{description::Name, language::Sprache, unicode::Normalized};

/// Deutsches Spiegelbild von [`IntermediateResult`].
#[derive(Debug)]
#[must_use]
pub enum ZwischenErgebnis<'t, T, E, A> {
    /// Ein erfolgreich geparster Wert.
    Wert(T),
    /// Ein frühes Beenden mit anzuzeigenden Nachrichten.
    FrühesBeenden(NonEmpty<Cow<'t, str>>),
    /// Parse-Fehler.
    Fehler(NonEmpty<KommentierterParseFehler<'t, E>>),
    /// Es ist noch kein eindeutiges Endergebnis verfügbar.
    Incomplete(A),
}
/// Deutsches Spiegelbild von [`Result`].
#[derive(Debug)]
#[must_use]
pub enum Ergebnis<'t, T, E> {
    /// Ein erfolgreich geparster Wert.
    Wert(T),
    /// Ein frühes Beenden mit anzuzeigenden Nachrichten.
    FrühesBeenden(NonEmpty<Cow<'t, str>>),
    /// Parse-Fehler.
    Fehler(NonEmpty<Fehler<'t, E>>),
}
/// Deutsche Spiegelung von [`AlternativeErrors`].
#[derive(Debug, Clone)]
#[must_use]
pub struct AlternativenFehler<'t, E>(pub NonEmpty<NonEmpty<Fehler<'t, E>>>);

/// Deutsches Spiegelbild von [`Error`].
#[derive(Debug, Clone)]
pub enum Fehler<'t, E> {
    /// Fehler jeder abgelehnten Alternative.
    Alternativen(Box<AlternativenFehler<'t, E>>),
    /// Eine benötigte Flag fehlte.
    FehlendeFlag {
        /// Alle Namen der Flag.
        name: Name<'t>,
        /// Präfix für eine invertierte Flag.
        invertiere_präfix: Normalized<'t>,
        /// Infix nach dem invertierenden Präfix.
        invertiere_infix: Normalized<'t>,
    },
    /// Ein benötigtes Wert-Argument fehlte.
    FehlenderWert {
        /// Alle Namen des Wert-Arguments.
        name: Name<'t>,
        /// Infix zwischen Name und Wert.
        wert_infix: Normalized<'t>,
        /// Die Wert-Metavariable.
        meta_var: &'t str,
    },
    /// Ein Wert konnte nicht geparst werden.
    ParseFehler(KommentierterParseFehler<'t, E>),
}
/// Deutsches Spiegelbild von [`AnnotatedParseError`].
#[derive(Debug, Clone)]
#[must_use]
pub struct KommentierterParseFehler<'t, E> {
    /// Alle Namen des Wert-Arguments.
    pub name: Name<'t>,
    /// Infix zwischen Name und Wert.
    pub wert_infix: Normalized<'t>,
    /// Die Wert-Metavariable.
    pub meta_var: &'t str,
    /// Der zugrundeliegende Parse-Fehler.
    pub fehler: ParseFehler<E>,
}
/// Deutsches Spiegelbild von [`ParseError`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFehler<E> {
    /// Der String ist kein valides UTF-8.
    InvaliderString(OsString),
    /// Das Parsen des UTF-8-Strings schlug fehl.
    ParseFehler(E),
}

impl<'t, T, E, A> From<IntermediateResult<'t, T, E, A>> for ZwischenErgebnis<'t, T, E, A> {
    #[inline]
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
    #[inline]
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
    #[inline]
    fn from(value: Result<'t, T, E>) -> Self {
        match value {
            Result::Value(value) => Self::Wert(value),
            Result::EarlyExit(value) => Self::FrühesBeenden(value),
            Result::Error(value) => Self::Fehler(value.map(Into::into)),
        }
    }
}
impl<'t, T, E> From<Ergebnis<'t, T, E>> for Result<'t, T, E> {
    #[inline]
    fn from(value: Ergebnis<'t, T, E>) -> Self {
        match value {
            Ergebnis::Wert(value) => Self::Value(value),
            Ergebnis::FrühesBeenden(value) => Self::EarlyExit(value),
            Ergebnis::Fehler(value) => Self::Error(value.map(Into::into)),
        }
    }
}
impl<'t, E> From<Error<'t, E>> for Fehler<'t, E> {
    #[inline]
    fn from(value: Error<'t, E>) -> Self {
        match value {
            Error::Alternatives(alternatives) => Self::Alternativen(Box::new(AlternativenFehler(
                alternatives.0.map(|errors| errors.map(Into::into)),
            ))),
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
    #[inline]
    fn from(value: Fehler<'t, E>) -> Self {
        match value {
            Fehler::Alternativen(alternativen) => Self::Alternatives(Box::new(AlternativeErrors(
                alternativen.0.map(|errors| errors.map(Into::into)),
            ))),
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
    #[inline]
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
    #[inline]
    fn from(value: KommentierterParseFehler<'t, E>) -> Self {
        Self {
            name: value.name,
            value_infix: value.wert_infix,
            meta_var: value.meta_var,
            error: value.fehler.into(),
        }
    }
}
impl<E: Into<F>, F> From<ParseError<E>> for ParseFehler<F> {
    #[inline]
    fn from(value: ParseError<E>) -> Self {
        match value {
            ParseError::InvalidString(value) => Self::InvaliderString(value),
            ParseError::ParseError(value) => Self::ParseFehler(value.into()),
        }
    }
}
impl<E, F: Into<E>> From<ParseFehler<F>> for ParseError<E> {
    #[inline]
    fn from(value: ParseFehler<F>) -> Self {
        match value {
            ParseFehler::InvaliderString(value) => Self::InvalidString(value),
            ParseFehler::ParseFehler(value) => Self::ParseError(value.into()),
        }
    }
}

impl<'t, T, E, A> ZwischenErgebnis<'t, T, E, A> {
    /// Konvertiert einen erfolgreich geparsten Wert.
    #[inline]
    pub fn konvertiere<S>(self, mapper: impl FnOnce(T) -> S) -> ZwischenErgebnis<'t, S, E, A> {
        IntermediateResult::from(self).convert(mapper).into()
    }

    /// Konvertiert die enthaltenen Fehler.
    #[inline]
    pub fn konvertiere_fehler<F>(self, mapper: impl Fn(E) -> F) -> ZwischenErgebnis<'t, T, F, A> {
        IntermediateResult::from(self).convert_error(mapper).into()
    }

    /// Konvertiert einen unvollständigen Wert.
    #[inline]
    pub fn konvertiere_incomplete<B>(
        self,
        mapper: impl FnOnce(A) -> B,
    ) -> ZwischenErgebnis<'t, T, E, B> {
        IntermediateResult::from(self).convert_incomplete(mapper).into()
    }
}
impl<'t, T, E> Ergebnis<'t, T, E> {
    /// Konvertiert einen erfolgreich geparsten Wert.
    #[inline]
    pub fn konvertiere<S>(self, mapper: impl FnOnce(T) -> S) -> Ergebnis<'t, S, E> {
        Result::from(self).convert(mapper).into()
    }

    /// Konvertiert die enthaltenen Fehler.
    #[inline]
    pub fn konvertiere_fehler<F>(self, mapper: impl Fn(E) -> F) -> Ergebnis<'t, T, F> {
        Result::from(self).convert_error(mapper).into()
    }
}
impl<'t, E> Fehler<'t, E> {
    /// Konvertiert den enthaltenen Fehler.
    #[inline]
    pub fn konvertiere<F>(self, mapper: impl Fn(E) -> F) -> Fehler<'t, F> {
        Error::from(self).convert(mapper).into()
    }
}
impl<'t, E> KommentierterParseFehler<'t, E> {
    /// Konvertiert den enthaltenen Fehler.
    #[inline]
    pub fn konvertiere<F>(self, mapper: impl FnOnce(E) -> F) -> KommentierterParseFehler<'t, F> {
        AnnotatedParseError::from(self).convert(mapper).into()
    }
}
impl<E> ParseFehler<E> {
    /// Konvertiert den enthaltenen Fehler.
    #[inline]
    pub fn konvertiere<F>(self, mapper: impl FnOnce(E) -> F) -> ParseFehler<F> {
        ParseError::from(self).convert(mapper).into()
    }
}
impl<E: Display> Fehler<'_, E> {
    /// Erstellt eine menschenlesbare Fehlermeldung mit deutschen Standardwerten.
    #[inline]
    pub fn fehlermeldung(&self) -> String {
        self.erstelle_fehlermeldung_mit_sprache(Sprache::DEUTSCH)
    }

    /// Erstellt eine menschenlesbare Fehlermeldung mit `sprache`.
    #[inline]
    pub fn erstelle_fehlermeldung_mit_sprache(&self, sprache: Sprache) -> String {
        self.erstelle_fehlermeldung_mit_trennzeichen(
            sprache.fehlende_flag,
            sprache.fehlender_wert,
            sprache.parse_fehler,
            sprache.invalider_string,
            sprache.alternative_fehler_trennzeichen,
        )
    }

    /// Erstellt eine menschenlesbare Fehlermeldung.
    #[inline]
    pub fn erstelle_fehlermeldung(
        &self,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
    ) -> String {
        self.erstelle_fehlermeldung_mit_trennzeichen(
            fehlende_flag,
            fehlender_wert,
            parse_fehler,
            invalider_string,
            Sprache::DEUTSCH.alternative_fehler_trennzeichen,
        )
    }

    /// Erstellt eine menschenlesbare Fehlermeldung mit einem Alternativen-Fehlertrennzeichen.
    fn erstelle_fehlermeldung_mit_trennzeichen(
        &self,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        alternative_fehler_trennzeichen: &str,
    ) -> String {
        match self {
            Self::Alternativen(alternativen) => alternativen
                .0
                .iter()
                .map(|fehler| {
                    fehler
                        .iter()
                        .map(|fehler| {
                            fehler.erstelle_fehlermeldung_mit_trennzeichen(
                                fehlende_flag,
                                fehlender_wert,
                                parse_fehler,
                                invalider_string,
                                alternative_fehler_trennzeichen,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .collect::<Vec<_>>()
                .join(&format!("\n{alternative_fehler_trennzeichen}\n")),
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
