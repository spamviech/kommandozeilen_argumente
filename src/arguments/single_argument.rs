//! A single command-line argument.

use std::{
    borrow::Cow,
    ffi::OsString,
    fmt::{self, Debug, Display},
};

use void::Void;

#[cfg(feature = "german")]
use crate::arguments::{early_exit::FrühesBeenden, help::Hilfe, value::Wert};
use crate::{
    arguments::{
        early_exit::EarlyExit, flag::Flag, help::Help, parsing::ParsedLongForms,
        parsing::ParsedMergedShortForms, parsing::ParsedShortForms, parsing::RecognizedArguments,
        value::Value,
    },
    description::ArgumentInput,
    dyn_to_owned::Show,
    outcome::{Error, Result},
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
#[cfg(feature = "german")]
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
    #[inline]
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

#[cfg(feature = "german")]
impl<T: Debug, Fehler: Debug> Debug for EinzelArgument<'_, T, Fehler> {
    #[inline]
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

#[cfg(feature = "german")]
impl<'t, T, Error> From<SingleArgument<'t, T, Error>> for EinzelArgument<'t, T, Error> {
    #[inline]
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

#[cfg(feature = "german")]
impl<'t, T, Fehler> From<EinzelArgument<'t, T, Fehler>> for SingleArgument<'t, T, Fehler> {
    #[inline]
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
    #[inline]
    fn from(flag: Flag<'t, T>) -> Self {
        Self::Flag(flag)
    }
}

#[cfg(feature = "german")]
impl<'t, T, Fehler> From<Flag<'t, T>> for EinzelArgument<'t, T, Fehler> {
    #[inline]
    fn from(flag: Flag<'t, T>) -> Self {
        SingleArgument::<T, Fehler>::from(flag).into()
    }
}

impl<'t, T: Display, Error> From<(EarlyExit<'t>, T)> for SingleArgument<'t, T, Error> {
    #[inline]
    fn from((early_exit, value): (EarlyExit<'t>, T)) -> Self {
        Self::EarlyExit { early_exit, value, display: Cow::Borrowed(&ToString::to_string) }
    }
}

#[cfg(feature = "german")]
impl<'t, T: Display, Fehler> From<(FrühesBeenden<'t>, T)> for EinzelArgument<'t, T, Fehler> {
    #[inline]
    fn from((frühes_beenden, wert): (FrühesBeenden<'t>, T)) -> Self {
        SingleArgument::<T, Fehler>::from((frühes_beenden.into(), wert)).into()
    }
}

impl<'t, Error> From<EarlyExit<'t>> for SingleArgument<'t, (), Error> {
    #[inline]
    fn from(early_exit: EarlyExit<'t>) -> Self {
        Self::EarlyExit {
            early_exit,
            value: (),
            display: Cow::Borrowed(&|value| format!("{value:?}")),
        }
    }
}

#[cfg(feature = "german")]
impl<'t, Fehler> From<FrühesBeenden<'t>> for EinzelArgument<'t, (), Fehler> {
    #[inline]
    fn from(frühes_beenden: FrühesBeenden<'t>) -> Self {
        SingleArgument::<(), Fehler>::from(EarlyExit::from(frühes_beenden)).into()
    }
}

impl<'t, T, Error> From<Value<'t, T, Error>> for SingleArgument<'t, T, Error> {
    #[inline]
    fn from(value: Value<'t, T, Error>) -> Self {
        Self::Value(value)
    }
}

#[cfg(feature = "german")]
impl<'t, T, Fehler> From<Wert<'t, T, Fehler>> for EinzelArgument<'t, T, Fehler> {
    #[inline]
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

#[cfg(feature = "german")]
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

#[cfg(feature = "german")]
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

    /// Creates this argument's syntax and corresponding help text.
    #[inline]
    pub fn create_help_text(&self, meta_default: &str, meta_possible_values: &str) -> Help {
        SingleArgumentRef(self).create_help_text(meta_default, meta_possible_values)
    }

    /// Converts this argument's value and parsing error to strings using its display functions.
    #[inline]
    pub fn as_string_value(&self) -> SingleArgument<'_, String, String> {
        SingleArgumentRef(self).as_string_value()
    }

    /// Parses standalone short-form arguments.
    #[inline]
    pub fn parse_short_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedShortForms<'t> {
        SingleArgumentRef(self).parse_short_form(args)
    }

    /// Parses long-form arguments.
    #[inline]
    pub fn parse_long_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedLongForms<'t> {
        SingleArgumentRef(self).parse_long_form(args)
    }

    /// Parses merged short-form arguments.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParsedMergedShortForms<'t> {
        SingleArgumentRef(self).parse_merged_short_forms(args)
    }
}

#[cfg(feature = "german")]
impl<'t, T, Fehler> EinzelArgument<'t, T, Fehler> {
    /// Erzeugt ein Wert-Argument mit passenden Typ-Parametern.
    #[inline]
    pub fn wert(wert: Wert<'t, T, Fehler>) -> Self {
        SingleArgument::value(wert.into()).into()
    }

    /// Erzeugt die Syntax und den zugehörigen Hilfetext dieses Arguments.
    #[inline]
    pub fn erzeuge_hilfe_text(&self, meta_standard: &str, meta_erlaubte_werte: &str) -> Hilfe {
        EinzelArgumentRef(self).erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
    }

    /// Konvertiert Wert und Parse-Fehler mittels ihrer Anzeigefunktionen in Strings.
    #[inline]
    pub fn als_string_wert(&self) -> EinzelArgument<'_, String, String> {
        EinzelArgumentRef(self).als_string_wert()
    }

    /// Parst eigenständige kurze Argumentformen.
    #[inline]
    pub fn parse_short_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedShortForms<'t> {
        EinzelArgumentRef(self).parse_short_form(args)
    }

    /// Parst lange Argumentformen.
    #[inline]
    pub fn parse_long_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedLongForms<'t> {
        EinzelArgumentRef(self).parse_long_form(args)
    }

    /// Parst zusammengefasste kurze Argumentformen.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParsedMergedShortForms<'t> {
        EinzelArgumentRef(self).parse_merged_short_forms(args)
    }
}

/// Borrowed view over an English single-argument representation.
struct SingleArgumentRef<'a, 't, T, Error>(&'a SingleArgument<'t, T, Error>);

impl<'a, 't, T, Error> SingleArgumentRef<'a, 't, T, Error> {
    /// Creates help text for the referenced argument.
    fn create_help_text(&self, meta_default: &str, meta_possible_values: &str) -> Help {
        match self.0 {
            SingleArgument::EarlyExit { early_exit, .. } => early_exit.create_help_text(),
            SingleArgument::Value(value) => {
                value.create_help_text(meta_default, meta_possible_values)
            },
            SingleArgument::Flag(flag) => flag.create_help_text(meta_default),
        }
    }

    /// Converts the referenced argument to its string-rendered representation.
    fn as_string_value(&self) -> SingleArgument<'a, String, String> {
        match self.0 {
            SingleArgument::Flag(flag) => SingleArgument::Flag(flag.as_string_flag()),
            SingleArgument::EarlyExit { early_exit, value, display } => SingleArgument::EarlyExit {
                early_exit: early_exit.clone(),
                value: display(value),
                display: Cow::Borrowed(&Clone::clone),
            },
            SingleArgument::Value(value) => SingleArgument::Value(value.as_string_value()),
        }
    }

    /// Parses non-merged short forms for the referenced argument.
    fn parse_short_form(self, args: impl Iterator<Item = ArgumentInput>) -> ParsedShortForms<'t> {
        match self.0 {
            SingleArgument::Flag(flag) => flag.parse_short_form(args),
            SingleArgument::EarlyExit { early_exit, .. } => early_exit.parse_short_form(args),
            SingleArgument::Value(value) => value.parse_short_form(args),
        }
    }

    /// Parses long forms for the referenced argument.
    fn parse_long_form(self, args: impl Iterator<Item = ArgumentInput>) -> ParsedLongForms<'t> {
        match self.0 {
            SingleArgument::Flag(flag) => flag.parse_long_form(args),
            SingleArgument::EarlyExit { early_exit, .. } => early_exit.parse_long_form(args),
            SingleArgument::Value(value) => value.parse_long_form(args),
        }
    }

    /// Parses merged short forms for the referenced argument.
    fn parse_merged_short_forms(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> ParsedMergedShortForms<'t> {
        self.parse_merged_short_inputs(args.map(ArgumentInput::Unchanged))
    }

    /// Parses normalized merged-short-form inputs for the referenced argument.
    fn parse_merged_short_inputs(
        self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedMergedShortForms<'t> {
        match self.0 {
            SingleArgument::Flag(flag) => flag.parse_merged_short_inputs(args),
            SingleArgument::EarlyExit { early_exit, .. } => {
                early_exit.parse_merged_short_inputs(args)
            },
            SingleArgument::Value(value) => value.parse_merged_short_inputs(args),
        }
    }
}

/// Borrowed view over a German single-argument representation.
#[cfg(feature = "german")]
struct EinzelArgumentRef<'a, 't, T, Fehler>(&'a EinzelArgument<'t, T, Fehler>);

#[cfg(feature = "german")]
impl<'a, 't, T, Fehler> EinzelArgumentRef<'a, 't, T, Fehler> {
    /// Erzeugt Hilfetext für das referenzierte Argument.
    fn erzeuge_hilfe_text(&self, meta_standard: &str, meta_erlaubte_werte: &str) -> Hilfe {
        match self.0 {
            EinzelArgument::Flag(flag) => flag.create_help_text(meta_standard).into(),
            EinzelArgument::FrühesBeenden { frühes_beenden, .. } => {
                frühes_beenden.erzeuge_hilfe_text()
            },
            EinzelArgument::Wert(wert) => {
                wert.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
            },
        }
    }

    /// Konvertiert das referenzierte Argument in seine String-Repräsentation.
    fn als_string_wert(&self) -> EinzelArgument<'a, String, String> {
        match self.0 {
            EinzelArgument::Flag(flag) => EinzelArgument::Flag(flag.as_string_flag()),
            EinzelArgument::FrühesBeenden { frühes_beenden, wert, anzeige } => {
                EinzelArgument::FrühesBeenden {
                    frühes_beenden: frühes_beenden.clone(),
                    wert: anzeige(wert),
                    anzeige: Cow::Borrowed(&Clone::clone),
                }
            },
            EinzelArgument::Wert(wert) => EinzelArgument::Wert(wert.als_string_wert()),
        }
    }

    /// Parst eigenständige kurze Argumentformen.
    fn parse_short_form(self, args: impl Iterator<Item = ArgumentInput>) -> ParsedShortForms<'t> {
        match self.0 {
            EinzelArgument::Flag(flag) => flag.parse_short_form(args),
            EinzelArgument::FrühesBeenden { frühes_beenden, .. } => {
                frühes_beenden.parse_short_form(args)
            },
            EinzelArgument::Wert(wert) => wert.parse_short_form(args),
        }
    }

    /// Parst lange Argumentformen.
    fn parse_long_form(self, args: impl Iterator<Item = ArgumentInput>) -> ParsedLongForms<'t> {
        match self.0 {
            EinzelArgument::Flag(flag) => flag.parse_long_form(args),
            EinzelArgument::FrühesBeenden { frühes_beenden, .. } => {
                frühes_beenden.parse_long_form(args)
            },
            EinzelArgument::Wert(wert) => wert.parse_long_form(args),
        }
    }

    /// Parst zusammengefasste kurze Argumentformen.
    fn parse_merged_short_forms(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> ParsedMergedShortForms<'t> {
        match self.0 {
            EinzelArgument::Flag(flag) => flag.parse_merged_short_forms(args),
            EinzelArgument::FrühesBeenden { frühes_beenden, .. } => {
                frühes_beenden.parse_merged_short_forms(args)
            },
            EinzelArgument::Wert(wert) => wert.parse_merged_short_forms(args),
        }
    }
}

impl<'t, T, ParseError> SingleArgument<'t, T, ParseError> {
    /// Converts the parse error type of a value argument.
    pub(crate) fn convert_error<NewError>(
        self,
        mapper: impl 't + Fn(ParseError) -> NewError + Clone,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> SingleArgument<'t, T, NewError> {
        match self {
            Self::Flag(flag) => SingleArgument::Flag(flag),
            Self::EarlyExit { early_exit, value, display } => {
                SingleArgument::EarlyExit { early_exit, value, display }
            },
            Self::Value(value) => {
                SingleArgument::Value(value.convert_error(mapper, display_new_error))
            },
        }
    }

    /// Validates typed accumulation without consuming defaults or callbacks.
    pub(crate) fn validate(
        &self,
        mut recognized: RecognizedArguments<'t>,
    ) -> Result<'t, (), ParseError> {
        if let Some(messages) = nonempty::NonEmpty::from_vec(
            recognized.early_exits.into_iter().map(|early_exit| early_exit.message).collect(),
        ) {
            return Result::EarlyExit(messages);
        }
        match self {
            Self::Flag(flag) => {
                let value = flag
                    .description
                    .name
                    .long
                    .iter()
                    .chain(flag.description.name.short.iter())
                    .find_map(|name| {
                        recognized
                            .flags
                            .iter()
                            .rev()
                            .find(|recognized_flag| recognized_flag.name == name.as_str())
                            .map(|recognized_flag| recognized_flag.value)
                    });
                if value.is_some() || flag.description.default.is_some() {
                    Result::Value(())
                } else {
                    Result::Error(nonempty::nonempty![Error::MissingFlag {
                        name: flag.description.name.clone(),
                        invert_prefix: flag.invert_prefix.string.clone(),
                        invert_infix: flag.invert_infix.string.clone(),
                    }])
                }
            },
            Self::EarlyExit { .. } => Result::Value(()),
            Self::Value(value) => match value.take_parsed_value(&mut recognized.values) {
                Ok(Some(_)) => Result::Value(()),
                Ok(None) if value.description.default.is_some() => Result::Value(()),
                Ok(None) => Result::Error(nonempty::nonempty![Error::MissingValue {
                    name: value.description.name.clone(),
                    value_infix: value.value_infix.string.clone(),
                    meta_var: value.meta_var,
                }]),
                Err(error) => Result::Error(nonempty::nonempty![Error::ParseError(error)]),
            },
        }
    }

    /// Accumulates recognized inputs into this argument's typed result.
    pub(crate) fn accumulate(
        self,
        mut recognized: RecognizedArguments<'t>,
    ) -> Result<'t, T, ParseError> {
        if let Some(messages) = nonempty::NonEmpty::from_vec(
            recognized.early_exits.into_iter().map(|early_exit| early_exit.message).collect(),
        ) {
            return Result::EarlyExit(messages);
        }
        match self {
            Self::Flag(flag) => {
                let value = flag
                    .description
                    .name
                    .long
                    .iter()
                    .chain(flag.description.name.short.iter())
                    .find_map(|name| {
                        recognized
                            .flags
                            .iter()
                            .rev()
                            .find(|recognized_flag| recognized_flag.name == name.as_str())
                            .map(|recognized_flag| recognized_flag.value)
                    });
                match value {
                    Some(value) => Result::Value((flag.convert)(value)),
                    None => match flag.description.default {
                        Some(default) => Result::Value(default),
                        None => Result::Error(nonempty::nonempty![Error::MissingFlag {
                            name: flag.description.name,
                            invert_prefix: flag.invert_prefix.string,
                            invert_infix: flag.invert_infix.string,
                        }]),
                    },
                }
            },
            Self::EarlyExit { value, .. } => Result::Value(value),
            Self::Value(value) => match value.take_parsed_value(&mut recognized.values) {
                Ok(Some(value)) => Result::Value(value),
                Ok(None) => match value.description.default {
                    Some(default) => Result::Value(default),
                    None => Result::Error(nonempty::nonempty![Error::MissingValue {
                        name: value.description.name,
                        value_infix: value.value_infix.string,
                        meta_var: value.meta_var,
                    }]),
                },
                Err(error) => Result::Error(nonempty::nonempty![Error::ParseError(error)]),
            },
        }
    }

    /// Parses merged short-form arguments while retaining adjusted-block remainders.
    #[inline]
    pub(crate) fn parse_merged_short_inputs(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedMergedShortForms<'t> {
        SingleArgumentRef(self).parse_merged_short_inputs(args)
    }

    /// Produces all merged-short recognition candidates for this argument.
    pub(crate) fn parse_merged_short_input_candidates(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> nonempty::NonEmpty<ParsedMergedShortForms<'t>> {
        if let Self::Value(value) = self {
            value.parse_merged_short_input_candidates(args)
        } else {
            nonempty::NonEmpty::singleton(self.parse_merged_short_inputs(args))
        }
    }
}
