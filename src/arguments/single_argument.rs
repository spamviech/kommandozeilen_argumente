//! A single command-line argument.

use std::{
    borrow::Cow,
    ffi::OsString,
    fmt::{self, Debug, Display},
};

use void::Void;

use crate::{
    arguments::{
        ParsedLongForms, ParsedMergedShortForms, ParsedShortForms, RecognizedArguments,
        early_exit::{EarlyExit, FrühesBeenden},
        flag::Flag,
        help::{Help, Hilfe},
        value::{Value, Wert},
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

    // FIXME why are there not english variants for these methods in SingleArgument?
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
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedShortForms<'t> {
        SingleArgumentRef::German(self).parse_short_form(args)
    }

    /// Parst lange Argumentformen.
    #[inline]
    pub fn parse_long_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedLongForms<'t> {
        SingleArgumentRef::German(self).parse_long_form(args)
    }

    /// Parst zusammengefasste kurze Argumentformen.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParsedMergedShortForms<'t> {
        SingleArgumentRef::German(self).parse_merged_short_forms(args)
    }
}

// FIXME split up english/german Ref-types
/// Borrowed view over either language-specific single-argument representation.
enum SingleArgumentRef<'a, 't, T, Error> {
    /// Borrows the English-primary representation.
    English(&'a SingleArgument<'t, T, Error>),
    /// Borrows the German mirror representation.
    German(&'a EinzelArgument<'t, T, Error>),
}

impl<'a, 't, T, Error> SingleArgumentRef<'a, 't, T, Error> {
    /// Creates help text for the referenced argument.
    fn create_help_text(&self, meta_default: &str, meta_possible_values: &str) -> Help {
        match self {
            Self::English(SingleArgument::EarlyExit { early_exit, .. }) => {
                early_exit.create_help_text()
            },
            Self::English(SingleArgument::Value(value)) => {
                value.create_help_text(meta_default, meta_possible_values)
            },
            Self::English(SingleArgument::Flag(flag))
            | Self::German(EinzelArgument::Flag(flag)) => flag.create_help_text(meta_default),
            Self::German(EinzelArgument::FrühesBeenden { frühes_beenden, .. }) => {
                frühes_beenden.erzeuge_hilfe_text().into()
            },
            Self::German(EinzelArgument::Wert(wert)) => {
                wert.erzeuge_hilfe_text(meta_default, meta_possible_values).into()
            },
        }
    }

    /// Converts the referenced argument to its string-rendered representation.
    fn as_string_value(&self) -> SingleArgument<'a, String, String> {
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

    /// Parses non-merged short forms for the referenced argument.
    fn parse_short_form(self, args: impl Iterator<Item = ArgumentInput>) -> ParsedShortForms<'t> {
        match self {
            Self::English(SingleArgument::Flag(flag))
            | Self::German(EinzelArgument::Flag(flag)) => flag.parse_short_form(args),
            Self::English(SingleArgument::EarlyExit { early_exit, .. }) => {
                early_exit.parse_short_form(args)
            },
            Self::German(EinzelArgument::FrühesBeenden { frühes_beenden, .. }) => {
                frühes_beenden.parse_short_form(args)
            },
            Self::English(SingleArgument::Value(value)) => value.parse_short_form(args),
            Self::German(EinzelArgument::Wert(wert)) => wert.parse_short_form(args),
        }
    }

    /// Parses long forms for the referenced argument.
    fn parse_long_form(self, args: impl Iterator<Item = ArgumentInput>) -> ParsedLongForms<'t> {
        match self {
            Self::English(SingleArgument::Flag(flag))
            | Self::German(EinzelArgument::Flag(flag)) => flag.parse_long_form(args),
            Self::English(SingleArgument::EarlyExit { early_exit, .. }) => {
                early_exit.parse_long_form(args)
            },
            Self::German(EinzelArgument::FrühesBeenden { frühes_beenden, .. }) => {
                frühes_beenden.parse_long_form(args)
            },
            Self::English(SingleArgument::Value(value)) => value.parse_long_form(args),
            Self::German(EinzelArgument::Wert(wert)) => wert.parse_long_form(args),
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
        match self {
            Self::English(SingleArgument::Flag(flag))
            | Self::German(EinzelArgument::Flag(flag)) => flag.parse_merged_short_inputs(args),
            Self::English(SingleArgument::EarlyExit { early_exit, .. }) => {
                early_exit.parse_merged_short_inputs(args)
            },
            Self::German(EinzelArgument::FrühesBeenden { frühes_beenden, .. }) => frühes_beenden
                .parse_merged_short_forms(args.filter_map(|argument| match argument {
                    ArgumentInput::Unchanged(argument) => Some(argument),
                    ArgumentInput::AdjustedMergedShortNames(_) => None,
                })),
            Self::English(SingleArgument::Value(value)) => value.parse_merged_short_inputs(args),
            Self::German(EinzelArgument::Wert(wert)) => {
                wert.parse_merged_short_forms(args.filter_map(|argument| match argument {
                    ArgumentInput::Unchanged(argument) => Some(argument),
                    ArgumentInput::AdjustedMergedShortNames(_) => None,
                }))
            },
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
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedShortForms<'t> {
        SingleArgumentRef::English(self).parse_short_form(args)
    }

    /// Parses long-form arguments.
    #[inline]
    pub fn parse_long_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedLongForms<'t> {
        SingleArgumentRef::English(self).parse_long_form(args)
    }

    /// Parses merged short-form arguments.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParsedMergedShortForms<'t> {
        SingleArgumentRef::English(self).parse_merged_short_forms(args)
    }

    /// Parses merged short-form arguments while retaining adjusted-block remainders.
    #[inline]
    pub(crate) fn parse_merged_short_inputs(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedMergedShortForms<'t> {
        SingleArgumentRef::English(self).parse_merged_short_inputs(args)
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
