//! A single command-line argument.

use std::{
    borrow::Cow,
    ffi::OsString,
    fmt::{self, Debug, Display},
};

use void::Void;

use crate::{
    arguments::{
        ParseMergedShortFormsResult,
        early_exit::{EarlyExit, FrühesBeenden},
        flag::Flag,
        help::{Help, Hilfe},
        value::{Value, Wert},
    },
    dyn_to_owned::Show,
};

/// Configuration of a single command-line argument.
///
/// ## Deutsch
/// Konfiguration eines einzelnen Kommandozeilen-Arguments.
#[must_use]
pub enum SingleArgument<'t, T, Error> {
    /// A flag argument.
    Flag(Flag<'t, T>),
    /// A flag argument that causes an early exit.
    EarlyExit {
        /// The flag and displayed message.
        early_exit: EarlyExit<'t>,
        /// The value if the flag is absent.
        value: T,
        /// Displays the default value.
        display: Cow<'t, dyn Show<'t, T>>,
    },
    /// A value argument.
    Value(Value<'t, T, Error>),
}

/// German mirror of [`SingleArgument`].
#[must_use]
pub enum EinzelArgument<'t, T, Fehler> {
    /// Ein Flag-Argument.
    Flag(Flag<'t, T>),
    /// Ein Flag-Argument, das zu einem frühen Beenden führt.
    FrühesBeenden {
        /// Die Flag und angezeigte Nachricht.
        frühes_beenden: FrühesBeenden<'t>,
        /// Der Wert, wenn die Flag nicht übergeben wird.
        wert: T,
        /// Anzeige des Standardwertes.
        anzeige: Cow<'t, dyn Show<'t, T>>,
    },
    /// Ein Wert-Argument.
    Wert(Wert<'t, T, Fehler>),
}

impl<T: Debug, Error: Debug> Debug for SingleArgument<'_, T, Error> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flag(flag) => formatter.debug_tuple("Flag").field(flag).finish(),
            Self::EarlyExit { early_exit, value, display: _ } => formatter
                .debug_struct("EarlyExit")
                .field("early_exit", early_exit)
                .field("value", value)
                .field("display", &"<closure>")
                .finish(),
            Self::Value(value) => formatter.debug_tuple("Value").field(value).finish(),
        }
    }
}

impl<T: Debug, Fehler: Debug> Debug for EinzelArgument<'_, T, Fehler> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flag(flag) => formatter.debug_tuple("Flag").field(flag).finish(),
            Self::FrühesBeenden { frühes_beenden, wert, anzeige: _ } => formatter
                .debug_struct("FrühesBeenden")
                .field("frühes_beenden", frühes_beenden)
                .field("wert", wert)
                .field("anzeige", &"<closure>")
                .finish(),
            Self::Wert(wert) => formatter.debug_tuple("Wert").field(wert).finish(),
        }
    }
}

impl<'t, T, Error> From<SingleArgument<'t, T, Error>> for EinzelArgument<'t, T, Error> {
    fn from(argument: SingleArgument<'t, T, Error>) -> Self {
        match argument {
            SingleArgument::Flag(flag) => Self::Flag(flag),
            SingleArgument::EarlyExit { early_exit, value, display } => Self::FrühesBeenden {
                frühes_beenden: early_exit.into(),
                wert: value,
                anzeige: display,
            },
            SingleArgument::Value(value) => Self::Wert(value.into()),
        }
    }
}

impl<'t, T, Fehler> From<EinzelArgument<'t, T, Fehler>> for SingleArgument<'t, T, Fehler> {
    fn from(argument: EinzelArgument<'t, T, Fehler>) -> Self {
        match argument {
            EinzelArgument::Flag(flag) => Self::Flag(flag),
            EinzelArgument::FrühesBeenden { frühes_beenden, wert, anzeige } => {
                Self::EarlyExit {
                    early_exit: frühes_beenden.into(), value: wert, display: anzeige
                }
            },
            EinzelArgument::Wert(wert) => Self::Value(wert.into()),
        }
    }
}

impl<'t, T, Error> From<Flag<'t, T>> for SingleArgument<'t, T, Error> {
    fn from(flag: Flag<'t, T>) -> Self {
        Self::Flag(flag)
    }
}

impl<'t, T, Fehler> From<Flag<'t, T>> for EinzelArgument<'t, T, Fehler> {
    fn from(flag: Flag<'t, T>) -> Self {
        SingleArgument::<T, Fehler>::from(flag).into()
    }
}

impl<'t, T: Display, Error> From<(EarlyExit<'t>, T)> for SingleArgument<'t, T, Error> {
    fn from((early_exit, value): (EarlyExit<'t>, T)) -> Self {
        Self::EarlyExit { early_exit, value, display: Cow::Borrowed(&ToString::to_string) }
    }
}

impl<'t, T: Display, Fehler> From<(FrühesBeenden<'t>, T)> for EinzelArgument<'t, T, Fehler> {
    fn from((frühes_beenden, wert): (FrühesBeenden<'t>, T)) -> Self {
        SingleArgument::<T, Fehler>::from((frühes_beenden.into(), wert)).into()
    }
}

impl<'t, Error> From<EarlyExit<'t>> for SingleArgument<'t, (), Error> {
    fn from(early_exit: EarlyExit<'t>) -> Self {
        Self::EarlyExit {
            early_exit,
            value: (),
            display: Cow::Borrowed(&|value| format!("{value:?}")),
        }
    }
}

impl<'t, Fehler> From<FrühesBeenden<'t>> for EinzelArgument<'t, (), Fehler> {
    fn from(frühes_beenden: FrühesBeenden<'t>) -> Self {
        SingleArgument::<(), Fehler>::from(EarlyExit::from(frühes_beenden)).into()
    }
}

impl<'t, T, Error> From<Value<'t, T, Error>> for SingleArgument<'t, T, Error> {
    fn from(value: Value<'t, T, Error>) -> Self {
        Self::Value(value)
    }
}

impl<'t, T, Fehler> From<Wert<'t, T, Fehler>> for EinzelArgument<'t, T, Fehler> {
    fn from(wert: Wert<'t, T, Fehler>) -> Self {
        SingleArgument::<T, Fehler>::from(Value::from(wert)).into()
    }
}

impl<'t, T: Display> SingleArgument<'t, T, Void> {
    /// Creates a flag argument with suitable type parameters.
    #[inline]
    pub fn flag(flag: Flag<'t, T>) -> Self {
        Self::Flag(flag)
    }

    /// Creates an early-exit argument with suitable type parameters.
    #[inline]
    pub fn early_exit_with_value(early_exit: EarlyExit<'t>, value: T) -> Self {
        Self::from((early_exit, value))
    }
}

impl<'t, T: Display> EinzelArgument<'t, T, Void> {
    /// Erzeugt ein Flag-Argument mit passenden Typ-Parametern.
    #[inline]
    pub fn flag(flag: Flag<'t, T>) -> Self {
        SingleArgument::flag(flag).into()
    }

    /// Erzeugt ein Frühes-Beenden-Argument mit passenden Typ-Parametern.
    #[inline]
    pub fn frühes_beenden_mit_wert(frühes_beenden: FrühesBeenden<'t>, wert: T) -> Self {
        SingleArgument::early_exit_with_value(frühes_beenden.into(), wert).into()
    }
}

impl<'t> SingleArgument<'t, (), Void> {
    /// Creates an early-exit argument with suitable type parameters.
    #[inline]
    pub fn early_exit(early_exit: EarlyExit<'t>) -> Self {
        Self::from(early_exit)
    }
}

impl<'t> EinzelArgument<'t, (), Void> {
    /// Erzeugt ein Frühes-Beenden-Argument mit passenden Typ-Parametern.
    #[inline]
    pub fn frühes_beenden(frühes_beenden: FrühesBeenden<'t>) -> Self {
        SingleArgument::early_exit(frühes_beenden.into()).into()
    }
}

impl<'t, T, Error> SingleArgument<'t, T, Error> {
    /// Creates a value argument with suitable type parameters.
    #[inline]
    pub fn value(value: Value<'t, T, Error>) -> Self {
        Self::Value(value)
    }
}

impl<'t, T, Fehler> EinzelArgument<'t, T, Fehler> {
    /// Erzeugt ein Wert-Argument mit passenden Typ-Parametern.
    #[inline]
    pub fn wert(wert: Wert<'t, T, Fehler>) -> Self {
        SingleArgument::value(wert.into()).into()
    }
}

enum SingleArgumentRef<'a, 't, T, Error> {
    English(&'a SingleArgument<'t, T, Error>),
    German(&'a EinzelArgument<'t, T, Error>),
}

impl<'a, 't, T, Error> SingleArgumentRef<'a, 't, T, Error> {
    fn create_help_text(&self, meta_default: &str, meta_possible_values: &str) -> Help {
        match self {
            Self::English(SingleArgument::Flag(flag)) => flag.create_help_text(meta_default),
            Self::English(SingleArgument::EarlyExit { early_exit, .. }) => {
                early_exit.create_help_text()
            },
            Self::English(SingleArgument::Value(value)) => {
                value.create_help_text(meta_default, meta_possible_values)
            },
            Self::German(EinzelArgument::Flag(flag)) => flag.create_help_text(meta_default),
            Self::German(EinzelArgument::FrühesBeenden { frühes_beenden, .. }) => {
                frühes_beenden.erzeuge_hilfe_text().into()
            },
            Self::German(EinzelArgument::Wert(wert)) => {
                wert.erzeuge_hilfe_text(meta_default, meta_possible_values).into()
            },
        }
    }

    fn as_string_value(self) -> SingleArgument<'a, String, String> {
        match self {
            Self::English(SingleArgument::Flag(flag))
            | Self::German(EinzelArgument::Flag(flag)) => {
                SingleArgument::Flag(flag.as_string_flag())
            },
            Self::English(SingleArgument::EarlyExit { early_exit, value, display }) => {
                SingleArgument::EarlyExit {
                    early_exit: early_exit.clone(),
                    value: display(value),
                    display: Cow::Borrowed(&Clone::clone),
                }
            },
            Self::German(EinzelArgument::FrühesBeenden { frühes_beenden, wert, anzeige }) => {
                SingleArgument::EarlyExit {
                    early_exit: frühes_beenden.clone().into(),
                    value: anzeige(wert),
                    display: Cow::Borrowed(&Clone::clone),
                }
            },
            Self::English(SingleArgument::Value(value)) => {
                SingleArgument::Value(value.as_string_value())
            },
            Self::German(EinzelArgument::Wert(wert)) => {
                SingleArgument::Value(wert.als_string_wert().into())
            },
        }
    }

    fn parse_short_form(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'a, 't, T, Error> {
        match self {
            Self::English(SingleArgument::Flag(flag))
            | Self::German(EinzelArgument::Flag(flag)) => flag.parse_short_form::<Error>(args),
            Self::English(SingleArgument::EarlyExit { early_exit, .. }) => {
                early_exit.parse_short_form::<T, Error>(args)
            },
            Self::German(EinzelArgument::FrühesBeenden { frühes_beenden, .. }) => {
                frühes_beenden.parse_short_form::<T, Error>(args)
            },
            Self::English(SingleArgument::Value(value)) => value.parse_short_form(args),
            Self::German(EinzelArgument::Wert(wert)) => wert.parse_short_form(args),
        }
    }

    fn parse_merged_short_forms(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'a, 't, T, Error> {
        match self {
            Self::English(SingleArgument::Flag(flag))
            | Self::German(EinzelArgument::Flag(flag)) => {
                flag.parse_merged_short_forms::<Error>(args)
            },
            Self::English(SingleArgument::EarlyExit { early_exit, .. }) => {
                early_exit.parse_merged_short_forms::<T, Error>(args)
            },
            Self::German(EinzelArgument::FrühesBeenden { frühes_beenden, .. }) => {
                frühes_beenden.parse_merged_short_forms::<T, Error>(args)
            },
            Self::English(SingleArgument::Value(value)) => value.parse_merged_short_forms(args),
            Self::German(EinzelArgument::Wert(wert)) => wert.parse_merged_short_forms(args),
        }
    }
}

impl<'t, T, Error> SingleArgument<'t, T, Error> {
    /// Creates this argument's syntax and corresponding help text.
    #[inline]
    pub fn create_help_text(&self, meta_default: &str, meta_possible_values: &str) -> Help {
        SingleArgumentRef::English(self).create_help_text(meta_default, meta_possible_values)
    }

    /// Converts this argument's value and parsing error to strings using its display functions.
    #[inline]
    pub fn as_string_value(&self) -> SingleArgument<'_, String, String> {
        SingleArgumentRef::English(self).as_string_value()
    }

    /// Parses standalone short-form arguments.
    #[inline]
    pub fn parse_short_form(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'_, 't, T, Error> {
        SingleArgumentRef::English(self).parse_short_form(args)
    }

    /// Parses merged short-form arguments.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'_, 't, T, Error> {
        SingleArgumentRef::English(self).parse_merged_short_forms(args)
    }
}

impl<'t, T, Fehler> EinzelArgument<'t, T, Fehler> {
    /// Erzeugt die Syntax und den zugehörigen Hilfetext dieses Arguments.
    #[inline]
    pub fn erzeuge_hilfe_text(&self, meta_standard: &str, meta_erlaubte_werte: &str) -> Hilfe {
        SingleArgumentRef::German(self).create_help_text(meta_standard, meta_erlaubte_werte).into()
    }

    /// Konvertiert Wert und Parse-Fehler mittels ihrer Anzeigefunktionen in Strings.
    #[inline]
    pub fn als_string_wert(&self) -> EinzelArgument<'_, String, String> {
        SingleArgumentRef::German(self).as_string_value().into()
    }

    /// Parst eigenständige kurze Argumentformen.
    #[inline]
    pub fn parse_short_form(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'_, 't, T, Fehler> {
        SingleArgumentRef::German(self).parse_short_form(args)
    }

    /// Parst zusammengefasste kurze Argumentformen.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'_, 't, T, Fehler> {
        SingleArgumentRef::German(self).parse_merged_short_forms(args)
    }
}
