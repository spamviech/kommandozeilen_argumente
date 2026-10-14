//! Value arguments.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
    str::FromStr,
};

use nonempty::NonEmpty;

use crate::{
    ParseFehler,
    arguments::{
        ParsedLongForms, ParsedMergedShortForms, ParsedRawValue, ParsedShortForms, ParsedValueName,
        help::{Help, Hilfe},
    },
    description::{ArgumentInput, Beschreibung, Description, Name},
    dyn_to_owned::{Parse, Show},
    language::{Language, Sprache},
    outcome::{AnnotatedParseError, ParseError},
    unicode::{Compare, Vergleich},
};

#[rustfmt::skip]
#[cfg(feature = "derive")]
#[cfg_attr(all(doc, feature = "derive", not(doctest)), doc(cfg(feature = "derive")))]
pub use kommandozeilen_argumente_derive::EnumArgument;

/// A type with a fixed set of values and a parsing method.
/// Intended for sum types containing only unit variants.
///
/// With the `derive` feature, its implementation can be
/// generated automatically with `#[derive(EnumArgument)]`.
///
/// ## Deutsch
/// Trait für Typen mit einer festen Anzahl an Werten und einer Methode zum Parsen.
pub trait EnumArgument: Sized {
    /// All variants of the type.
    #[must_use]
    fn varianten() -> Option<NonEmpty<Self>>;

    /// All variants of the type.
    #[inline]
    #[must_use]
    fn variants() -> Option<NonEmpty<Self>> {
        Self::varianten()
    }

    /// Parses a value from `arg`.
    ///
    /// # Errors
    ///
    /// Returns a parse error if `arg` cannot be parsed.
    fn parse_enum(arg: &OsStr) -> Result<Self, ParseError<String>>;
}

/// A value argument.
///
/// Note: When specifying the value in the same argument, i.e. separated by a `value_infix`,
/// or without one directly after a short name,
/// only Unicode (utf8) string representations for the value are supported.
/// The parse-function receives a [`Normalized`](crate::unicode::Normalized) String in this case.
#[must_use]
pub struct Value<'t, T, Error> {
    /// General description of the argument.
    pub description: Description<'t, T>,
    /// Infix that separates an inline value from its name.
    ///
    /// Note: When using short names, `value_infix` longer than a
    /// [`Grapheme`](unicode_segmentation::UnicodeSegmentation::graphemes) are not supported.
    pub value_infix: Compare<'t>,
    /// Metavariable used in help text.
    pub meta_var: &'t str,
    /// Values displayed in help text.
    pub possible_values: Option<NonEmpty<T>>,
    /// Parses an operating-system string into a value.
    pub parse: Cow<'t, dyn Parse<'t, T, ParseError<Error>>>,
    /// Displays a value.
    pub display: Cow<'t, dyn Show<'t, T>>,
    /// Displays a parsing error.
    pub display_error: Cow<'t, dyn Show<'t, Error>>,
}

/// German mirror of [`Value`].
///
/// Anmerkung: Wenn ein Wert im selben Argument wie der Name angegeben wird,
/// durch ein `wert_infix` getrennt, oder ohne Trennzeichen direkt nach einem Kurznamen,
/// werden nur Unicode String Darstellungen für den Wert unterstützt.
/// Die parse-Funktion erhält dabei eine [`Normalisiert`](crate::unicode::Normalisiert)en String.
#[must_use]
pub struct Wert<'t, T, Fehler> {
    /// Allgemeine Beschreibung des Arguments.
    pub beschreibung: Beschreibung<'t, T>,
    /// Infix, das einen Wert im selben Argument vom Namen trennt.
    ///
    /// Anmerkung: In Kombination mit Kurznamen werten `wert_infix`e länger als ein
    /// [`Grapheme`](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// nicht unterstützt.
    pub wert_infix: Vergleich<'t>,
    /// Meta-Variable im Hilfetext.
    pub meta_var: &'t str,
    /// Im Hilfetext angezeigte erlaubte Werte.
    pub mögliche_werte: Option<NonEmpty<T>>,
    /// Parst einen Betriebssystem-String in einen Wert.
    pub parse: Cow<'t, dyn Parse<'t, T, ParseFehler<Fehler>>>,
    /// Zeigt einen Wert an.
    pub anzeige: Cow<'t, dyn Show<'t, T>>,
    /// Zeigt einen Parse-Fehler an.
    pub anzeige_fehler: Cow<'t, dyn Show<'t, Fehler>>,
}

impl<'t, T, E> From<Value<'t, T, E>> for Wert<'t, T, E> {
    #[inline]
    fn from(value: Value<'t, T, E>) -> Self {
        let Value {
            description,
            value_infix,
            meta_var,
            possible_values,
            parse,
            display,
            display_error,
        } = value;
        let parse_boxed: Box<dyn 't + Parse<'t, T, ParseFehler<E>>> =
            Box::new(move |input: &OsStr| parse(input).map_err(ParseFehler::from));
        Self {
            beschreibung: description.into(),
            wert_infix: value_infix.into(),
            meta_var,
            mögliche_werte: possible_values,
            parse: Cow::Owned(parse_boxed),
            anzeige: display,
            anzeige_fehler: display_error,
        }
    }
}

impl<'t, T, E> From<Wert<'t, T, E>> for Value<'t, T, E> {
    #[inline]
    fn from(value: Wert<'t, T, E>) -> Self {
        let Wert {
            beschreibung,
            wert_infix,
            meta_var,
            mögliche_werte,
            parse,
            anzeige,
            anzeige_fehler,
        } = value;
        let parse_boxed: Box<dyn 't + Parse<'t, T, ParseError<E>>> =
            Box::new(move |input: &OsStr| parse(input).map_err(ParseError::from));
        Self {
            description: beschreibung.into(),
            value_infix: wert_infix.into(),
            meta_var,
            possible_values: mögliche_werte,
            parse: Cow::Owned(parse_boxed),
            display: anzeige,
            display_error: anzeige_fehler,
        }
    }
}

impl<T: Debug, E> Debug for Value<'_, T, E> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Value")
            .field("description", &self.description)
            .field("value_infix", &self.value_infix)
            .field("meta_var", &self.meta_var)
            .field("possible_values", &self.possible_values)
            .field("parse", &"<closure>")
            .field("display", &"<closure>")
            .field("display_error", &"<closure>")
            .finish()
    }
}
impl<T: Debug, E> Debug for Wert<'_, T, E> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Wert")
            .field("beschreibung", &self.beschreibung)
            .field("wert_infix", &self.wert_infix)
            .field("meta_var", &self.meta_var)
            .field("mögliche_werte", &self.mögliche_werte)
            .field("parse", &"<closure>")
            .field("anzeige", &"<closure>")
            .field("anzeige_fehler", &"<closure>")
            .finish()
    }
}

impl<'t, T: Display + FromStr> Value<'t, T, <T as FromStr>::Err>
where
    <T as FromStr>::Err: Display,
{
    /// Creates a value argument based on [`FromStr`].
    #[inline]
    pub fn new(description: Description<'t, T>, possible_values: Option<NonEmpty<T>>) -> Self {
        Self::new_with_language(description, possible_values, Language::ENGLISH)
    }

    /// Creates a value argument with localized defaults.
    #[inline]
    pub fn new_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        language: Language,
    ) -> Self {
        Self {
            description,
            value_infix: language.value_infix.into(),
            meta_var: language.meta_var,
            possible_values,
            parse: Cow::Borrowed(&|value: &OsStr| {
                value
                    .to_str()
                    .ok_or_else(|| ParseError::InvalidString(OsString::from(value)))
                    .and_then(|string| string.parse().map_err(ParseError::ParseError))
            }),
            display: Cow::Borrowed(&<T as ToString>::to_string),
            display_error: Cow::Borrowed(&<<T as FromStr>::Err as ToString>::to_string),
        }
    }
}
impl<'t, T: Display + EnumArgument> Value<'t, T, String> {
    /// Creates a value argument based on [`EnumArgument`].
    #[inline]
    pub fn new_enum(description: Description<'t, T>) -> Self {
        Self::new_enum_with_language(description, Language::ENGLISH)
    }

    /// Creates a value argument with localized defaults.
    #[inline]
    pub fn new_enum_with_language(description: Description<'t, T>, language: Language) -> Self {
        Self {
            description,
            value_infix: language.value_infix.into(),
            meta_var: language.meta_var,
            possible_values: EnumArgument::variants(),
            parse: Cow::Borrowed(&EnumArgument::parse_enum),
            display: Cow::Borrowed(&<T as ToString>::to_string),
            display_error: Cow::Borrowed(&Clone::clone),
        }
    }
}
impl<'t, T: Display + FromStr> Wert<'t, T, <T as FromStr>::Err>
where
    <T as FromStr>::Err: Display,
{
    /// Erzeugt ein Wert-Argument auf Grundlage von [`FromStr`].
    #[inline]
    pub fn neu(beschreibung: Beschreibung<'t, T>, mögliche_werte: Option<NonEmpty<T>>) -> Self {
        Value::new_with_language(beschreibung.into(), mögliche_werte, Language::GERMAN).into()
    }

    /// Erzeugt ein Wert-Argument mit lokalisierten Standardwerten.
    #[inline]
    pub fn neu_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        mögliche_werte: Option<NonEmpty<T>>,
        sprache: Sprache,
    ) -> Self {
        Value::new_with_language(beschreibung.into(), mögliche_werte, sprache.into()).into()
    }
}
impl<'t, T: Display + EnumArgument> Wert<'t, T, String> {
    /// Erzeugt ein Wert-Argument auf Grundlage von [`EnumArgument`].
    #[inline]
    pub fn neu_enum(beschreibung: Beschreibung<'t, T>) -> Self {
        Value::new_enum_with_language(beschreibung.into(), Language::GERMAN).into()
    }

    /// Erzeugt ein Wert-Argument mit lokalisierten Standardwerten.
    #[inline]
    pub fn neu_enum_mit_sprache(beschreibung: Beschreibung<'t, T>, sprache: Sprache) -> Self {
        Value::new_enum_with_language(beschreibung.into(), sprache.into()).into()
    }
}
/// Appends comma-separated displayed values to `string`.
#[expect(
    clippy::ptr_arg,
    reason = "Lifetimes don't work out (easily) using as_ref and &dyn Show<'_,T>"
)]
fn show_elements<'t, T: 't>(
    string: &mut String,
    display: &Cow<'_, dyn Show<'_, T>>,
    values: impl IntoIterator<Item = &'t T>,
) {
    for (index, value) in values.into_iter().enumerate() {
        if index != 0 {
            string.push_str(", ");
        }
        string.push_str(&display(value));
    }
}
/// Converts a typed value argument into its string-rendered representation.
#[expect(
    clippy::ptr_arg,
    clippy::too_many_arguments,
    reason = "Internal function, we are using Cow everywhere. Takes all configuration options \
              individually."
)]
fn as_string_value<'a, 't, T, E, PE: Into<ParseError<E>>>(
    name: &'a Name<'t>,
    help: Option<&'t str>,
    default: Option<&'a T>,
    value_infix: Compare<'t>,
    meta_var: &'t str,
    possible_values: Option<&'a NonEmpty<T>>,
    parse: &'a Cow<'t, dyn Parse<'t, T, PE>>,
    display: &'a Cow<'t, dyn Show<'t, T>>,
    display_error: &'a Cow<'t, dyn Show<'t, E>>,
) -> Value<'a, String, String>
where
    't: 'a,
{
    let parse_boxed: Box<dyn 'a + Parse<'a, String, ParseError<String>>> =
        Box::new(|value: &OsStr| {
            parse(value)
                .map(|value| display(&value))
                .map_err(|error| error.into().convert(|error| display_error(&error)))
        });
    Value {
        description: Description { name: name.clone(), help, default: default.map(&**display) },
        value_infix,
        meta_var,
        possible_values: NonEmpty::from_vec(
            possible_values.into_iter().flatten().map(&**display).collect(),
        ),
        parse: Cow::Owned(parse_boxed),
        display: Cow::Borrowed(&Clone::clone),
        display_error: Cow::Borrowed(&Clone::clone),
    }
}
/// Parses non-merged short value forms from the supplied inputs.
fn parse_short_value_forms<'argument>(
    name: &Name<'argument>,
    value_infix: &Compare<'argument>,
    args: impl Iterator<Item = ArgumentInput>,
) -> ParsedShortForms<'argument> {
    let mut values = HashMap::new();
    let mut missing_values: HashSet<ParsedValueName<'argument>> = HashSet::default();
    let mut args = args.peekable();
    let mut remaining = Vec::new();
    while let Some(argument) = args.next() {
        match argument {
            ArgumentInput::Unchanged(argument) => {
                let input = Cow::Owned(argument.to_string_lossy().into_owned());
                if let Some((name, inline_value)) =
                    name.parse_short_with_value(value_infix, &argument)
                {
                    let parsed_name = ParsedValueName { name: Cow::Owned(name.into()) };
                    if let Some(value) = inline_value
                        .map(|normalized| OsString::from(normalized.as_str()))
                        .or_else(|| {
                            args.next_if(|arg| matches!(arg, ArgumentInput::Unchanged(_))).and_then(
                                |arg| match arg {
                                    ArgumentInput::Unchanged(value) => Some(value),
                                    ArgumentInput::AdjustedMergedShortNames(_) => None,
                                },
                            )
                        })
                    {
                        let _ = values.insert(parsed_name, ParsedRawValue { value, input });
                    } else {
                        let _ = missing_values.insert(parsed_name);
                    }
                } else {
                    remaining.push(ArgumentInput::Unchanged(argument));
                }
            },
            adjusted @ ArgumentInput::AdjustedMergedShortNames(_) => remaining.push(adjusted),
        }
    }
    ParsedShortForms {
        early_exits: Vec::new(),
        flags: Vec::new(),
        missing_values,
        values,
        remaining,
    }
}
/// Parses long value forms from the supplied inputs.
fn parse_long_value_forms<'argument>(
    name: &Name<'argument>,
    value_infix: &Compare<'argument>,
    args: impl Iterator<Item = ArgumentInput>,
) -> ParsedLongForms<'argument> {
    let mut values = HashMap::new();
    let mut missing_values: HashSet<ParsedValueName<'argument>> = HashSet::default();
    let mut args = args.peekable();
    let mut remaining = Vec::new();
    while let Some(argument) = args.next() {
        match argument {
            ArgumentInput::Unchanged(argument) => {
                let input = Cow::Owned(argument.to_string_lossy().into_owned());
                if let Some((name, inline_value)) =
                    name.parse_long_with_value(value_infix, &argument)
                {
                    let parsed_name = ParsedValueName { name: Cow::Owned(name.into()) };
                    if let Some(value) = inline_value
                        .map(|normalized| OsString::from(normalized.as_str()))
                        .or_else(|| {
                            args.next_if(|arg| matches!(arg, ArgumentInput::Unchanged(_))).and_then(
                                |arg| match arg {
                                    ArgumentInput::Unchanged(value) => Some(value),
                                    ArgumentInput::AdjustedMergedShortNames(_) => None,
                                },
                            )
                        })
                    {
                        let _ = values.insert(parsed_name, ParsedRawValue { value, input });
                    } else {
                        let _ = missing_values.insert(parsed_name);
                    }
                } else {
                    remaining.push(ArgumentInput::Unchanged(argument));
                }
            },
            adjusted @ ArgumentInput::AdjustedMergedShortNames(_) => remaining.push(adjusted),
        }
    }
    ParsedLongForms {
        early_exits: Vec::new(),
        flags: Vec::new(),
        missing_values,
        values,
        remaining,
    }
}
/// Parses merged short value forms from the supplied inputs.
fn parse_value_merged_short_forms<'argument>(
    name: &Name<'argument>,
    value_infix: &Compare<'argument>,
    args: impl Iterator<Item = ArgumentInput>,
) -> ParsedMergedShortForms<'argument> {
    let mut values = HashMap::new();
    let mut missing_values: HashSet<ParsedValueName<'argument>> = HashSet::default();
    let mut args = args.peekable();
    let mut remaining = Vec::new();
    while let Some(argument) = args.next() {
        let input = match &argument {
            ArgumentInput::Unchanged(argument) => {
                Cow::Owned(argument.to_string_lossy().into_owned())
            },
            ArgumentInput::AdjustedMergedShortNames(_) => Cow::Borrowed(""),
        };
        if let Some(parsed) = name.parse_with_value_merge_short_forms(value_infix, &argument) {
            let value = parsed.inline_value.map_or_else(
                || {
                    args.next_if(|arg| matches!(arg, ArgumentInput::Unchanged(_))).and_then(|arg| {
                        match arg {
                            ArgumentInput::Unchanged(value) => Some(value),
                            ArgumentInput::AdjustedMergedShortNames(_) => None,
                        }
                    })
                },
                |value| Some(OsString::from(value.into_string())),
            );
            let parsed_name = ParsedValueName { name: Cow::Owned(parsed.name.into()) };
            if let Some(value) = value {
                let _ = values.insert(parsed_name, ParsedRawValue { value, input });
            } else {
                let _ = missing_values.insert(parsed_name);
            }
            if let Some(remaining_input) = parsed.remaining {
                remaining.push(remaining_input);
            }
        } else {
            remaining.push(argument);
        }
    }
    ParsedMergedShortForms {
        early_exits: Vec::new(),
        flags: Vec::new(),
        missing_values,
        values,
        remaining,
    }
}
/// Creates syntax and help text from the shared parts of a German or English value argument.
#[expect(
    clippy::ptr_arg,
    clippy::too_many_arguments,
    reason = "Internal method, we are using Cow everywhere."
)]
fn create_value_help_text<T>(
    name: &Name<'_>,
    help: Option<&str>,
    default: Option<&T>,
    value_infix: &str,
    meta_var: &str,
    possible_values: Option<&NonEmpty<T>>,
    display: &Cow<'_, dyn Show<'_, T>>,
    meta_default: &str,
    meta_allowed_values: &str,
) -> Help {
    let Name { long_prefix, long, short_prefix, short } = name;
    let mut syntax = long_prefix.as_str().to_owned();
    Name::alternatives_as_regex(&long.head, &long.tail, &mut syntax);
    syntax.push_str("( |");
    syntax.push_str(value_infix);
    syntax.push(')');
    syntax.push_str(meta_var);
    if let Some((head, tail)) = short.split_first() {
        syntax.push_str(" | ");
        syntax.push_str(short_prefix.as_str());
        Name::alternatives_as_regex(head, tail, &mut syntax);
        syntax.push_str("[ |");
        syntax.push_str(value_infix);
        syntax.push(']');
        syntax.push_str(meta_var);
    }
    let suffix = match (default, possible_values) {
        (None, None) => None,
        (None, Some(values)) => {
            let mut text = format!("[{meta_allowed_values}: ");
            show_elements(&mut text, display, values);
            text.push(']');
            Some(text)
        },
        (Some(value), None) => Some(format!("[{meta_default}: {}]", display(value))),
        (Some(value), Some(values)) => {
            let mut text = format!("[{meta_default}: {}, {meta_allowed_values}: ", display(value));
            show_elements(&mut text, display, values);
            text.push(']');
            Some(text)
        },
    };
    let help = match (help, suffix) {
        (None, suffix) => suffix,
        (Some(help), None) => Some(help.to_owned()),
        (Some(help), Some(suffix)) => Some(format!("{help} {suffix}")),
    };
    Help { syntax, help }
}

impl<'t, T, E> Value<'t, T, E> {
    /// Converts this argument's parse error type.
    pub(crate) fn convert_error<NewError>(
        self,
        mapper: impl 't + Fn(E) -> NewError + Clone,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> Value<'t, T, NewError> {
        let Self {
            description,
            value_infix,
            meta_var,
            possible_values,
            parse,
            display,
            display_error: _,
        } = self;
        let parse: Box<dyn 't + Parse<'t, T, ParseError<NewError>>> =
            Box::new(move |input| parse(input).map_err(|error| error.convert(mapper.clone())));
        let display_error: Box<dyn 't + Show<'t, NewError>> = Box::new(display_new_error);
        Value {
            description,
            value_infix,
            meta_var,
            possible_values,
            parse: Cow::Owned(parse),
            display,
            display_error: Cow::Owned(display_error),
        }
    }

    /// Creates this argument's syntax and help text.
    #[inline]
    pub fn create_help_text(&self, meta_default: &str, meta_allowed_values: &str) -> Help {
        create_value_help_text(
            &self.description.name,
            self.description.help,
            self.description.default.as_ref(),
            self.value_infix.as_str(),
            self.meta_var,
            self.possible_values.as_ref(),
            &self.display,
            meta_default,
            meta_allowed_values,
        )
    }

    /// Converts this value argument to strings using its display functions.
    #[inline]
    pub fn as_string_value(&self) -> Value<'_, String, String> {
        as_string_value(
            &self.description.name,
            self.description.help,
            self.description.default.as_ref(),
            self.value_infix.clone(),
            self.meta_var,
            self.possible_values.as_ref(),
            &self.parse,
            &self.display,
            &self.display_error,
        )
    }

    /// Removes and parses this argument's matching raw value.
    ///
    /// The map is candidate-local during alternative evaluation, so consuming a value here cannot
    /// prevent another alternative from attempting to parse the same original input.
    ///
    /// ## Errors
    ///
    /// If no matching value was found.
    #[inline]
    #[expect(
        clippy::result_large_err,
        reason = "Only move Error to the Heap when required by the public interface."
    )]
    pub fn take_parsed_value(
        &self,
        values: &mut HashMap<ParsedValueName<'t>, ParsedRawValue<'t>>,
    ) -> Result<Option<T>, AnnotatedParseError<'t, E>> {
        let raw_value =
            self.description.name.long.iter().chain(self.description.name.short.iter()).find_map(
                |name| values.remove(&ParsedValueName { name: Cow::Owned(name.as_str().into()) }),
            );
        raw_value
            .map(|raw_value| {
                (self.parse)(&raw_value.value).map_err(|error| AnnotatedParseError {
                    name: self.description.name.clone(),
                    value_infix: self.value_infix.string.clone(),
                    meta_var: self.meta_var,
                    error,
                })
            })
            .transpose()
    }

    /// Parses standalone short-form arguments.
    #[inline]
    pub fn parse_short_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedShortForms<'t> {
        parse_short_value_forms(&self.description.name, &self.value_infix, args)
    }

    /// Parses long-form arguments.
    #[inline]
    pub fn parse_long_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedLongForms<'t> {
        parse_long_value_forms(&self.description.name, &self.value_infix, args)
    }

    /// Parses merged short-form arguments.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParsedMergedShortForms<'t> {
        self.parse_merged_short_inputs(args.map(ArgumentInput::Unchanged))
    }

    /// Parses merged short-form arguments while retaining adjusted-block remainders.
    #[inline]
    pub(crate) fn parse_merged_short_inputs(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedMergedShortForms<'t> {
        parse_value_merged_short_forms(&self.description.name, &self.value_infix, args)
    }

    /// Produces both interpretations of a matched merged short value.
    ///
    /// A value-like suffix can instead be a sequence of subsequent short flags.  The matched
    /// branch is returned first; the untouched branch lets the enclosing combination recognize
    /// those flags and is validated before the parser definition is consumed.
    pub(crate) fn parse_merged_short_input_candidates(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> NonEmpty<ParsedMergedShortForms<'t>> {
        let args = args.collect::<Vec<_>>();
        let parsed = self.parse_merged_short_inputs(args.iter().cloned());
        if parsed.values.is_empty() && parsed.missing_values.is_empty() {
            NonEmpty::singleton(parsed)
        } else {
            NonEmpty {
                head: parsed,
                tail: vec![ParsedMergedShortForms {
                    early_exits: Vec::new(),
                    flags: Vec::new(),
                    missing_values: HashSet::default(),
                    values: HashMap::default(),
                    remaining: args,
                }],
            }
        }
    }
}

impl<'t, T, E> Wert<'t, T, E> {
    /// Parst eigenständige kurze Argumentformen.
    #[inline]
    pub fn parse_short_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedShortForms<'t> {
        let value_infix = self.wert_infix.clone().into();
        parse_short_value_forms(&self.beschreibung.name, &value_infix, args)
    }

    /// Parst lange Argumentformen.
    #[inline]
    pub fn parse_long_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedLongForms<'t> {
        let value_infix = self.wert_infix.clone().into();
        parse_long_value_forms(&self.beschreibung.name, &value_infix, args)
    }

    /// Parst zusammengefasste kurze Argumentformen.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParsedMergedShortForms<'t> {
        let value_infix = self.wert_infix.clone().into();
        parse_value_merged_short_forms(
            &self.beschreibung.name,
            &value_infix,
            args.map(ArgumentInput::Unchanged),
        )
    }

    /// Erzeugt Syntax und Hilfetext für dieses Argument.
    #[inline]
    pub fn erzeuge_hilfe_text(&self, meta_standard: &str, meta_erlaubte_werte: &str) -> Hilfe {
        create_value_help_text(
            &self.beschreibung.name,
            self.beschreibung.hilfe,
            self.beschreibung.standard.as_ref(),
            self.wert_infix.as_str(),
            self.meta_var,
            self.mögliche_werte.as_ref(),
            &self.anzeige,
            meta_standard,
            meta_erlaubte_werte,
        )
        .into()
    }

    /// Konvertiert dieses Wert-Argument mittels seiner Anzeigefunktionen in Strings.
    #[inline]
    pub fn als_string_wert(&self) -> Wert<'_, String, String> {
        as_string_value(
            &self.beschreibung.name,
            self.beschreibung.hilfe,
            self.beschreibung.standard.as_ref(),
            self.wert_infix.clone().into(),
            self.meta_var,
            self.mögliche_werte.as_ref(),
            &self.parse,
            &self.anzeige,
            &self.anzeige_fehler,
        )
        .into()
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::unwrap_used,
        reason = "Simplify writing assert_eq for the single element of a map."
    )]

    use std::{
        borrow::Cow,
        ffi::{OsStr, OsString},
    };

    use crate::{
        ArgumentInput, Description, arguments::value::Value, dyn_to_owned::Parse,
        outcome::ParseError,
    };

    const EMPTY_REMAINING: [ArgumentInput; 0] = [];

    #[test]
    fn parses_merged_short_value_with_and_without_infix() {
        let value =
            Value::<String, _>::new(Description::new("--", "output", "-", "o", None, None), None);
        for input in ["-ovalue", "-o=value"] {
            let result = value.parse_merged_short_forms([OsString::from(input)].into_iter());
            assert_eq!(result.values.len(), 1);
            assert_eq!(result.values.values().next().unwrap().value, "value");
            assert_eq!(result.remaining, EMPTY_REMAINING);
        }
    }

    #[test]
    fn parses_standalone_short_values() {
        let value =
            Value::<String, _>::new(Description::new("--", "output", "-", "o", None, None), None);
        for input in ["-ovalue", "-o=value"] {
            let result = value.parse_short_form(
                [OsString::from(input)].into_iter().map(ArgumentInput::Unchanged),
            );
            assert_eq!(result.values.len(), 1);
            assert_eq!(result.values.values().next().unwrap().value, "value");
            assert_eq!(result.remaining, EMPTY_REMAINING);
        }
        {
            let result = value.parse_short_form(
                [OsString::from("-o"), OsString::from("value")]
                    .into_iter()
                    .map(ArgumentInput::Unchanged),
            );
            assert_eq!(result.values.values().next().unwrap().value, "value");
        }

        {
            let result = value.parse_short_form(
                [OsString::from("--output")].into_iter().map(ArgumentInput::Unchanged),
            );
            assert!(result.values.is_empty());
            assert_eq!(result.remaining.len(), 1);
        }
    }

    #[test]
    fn consumes_the_next_argument_for_a_merged_short_value() {
        let value =
            Value::<String, _>::new(Description::new("--", "output", "-", "o", None, None), None);
        let result = value
            .parse_merged_short_forms([OsString::from("-o"), OsString::from("value")].into_iter());

        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values.values().next().unwrap().value, "value");
        assert_eq!(result.remaining, EMPTY_REMAINING);
    }

    #[test]
    fn parses_long_values_with_inline_and_next_argument_values() {
        let value =
            Value::<String, _>::new(Description::new("--", "output", "-", "o", None, None), None);
        {
            let result = value.parse_long_form(
                [OsString::from("--output=value")].into_iter().map(ArgumentInput::Unchanged),
            );
            assert_eq!(result.values.len(), 1);
            assert_eq!(result.values.values().next().unwrap().value, "value");
            assert_eq!(result.remaining, EMPTY_REMAINING);
        }
        {
            let result = value.parse_long_form(
                [OsString::from("--output"), OsString::from("value")]
                    .into_iter()
                    .map(ArgumentInput::Unchanged),
            );
            assert_eq!(result.values.len(), 1);
            assert_eq!(result.values.values().next().unwrap().value, "value");
            assert_eq!(result.remaining, EMPTY_REMAINING);
        }
    }

    #[test]
    fn preserves_raw_value_after_a_normalized_long_name() {
        let value =
            Value::<String, _>::new(Description::new("--", "café", "-", "c", None, None), None);
        let result = value.parse_long_form(
            [OsString::from("--cafe\u{301}=value")].into_iter().map(ArgumentInput::Unchanged),
        );

        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values.values().next().unwrap().value, "value");
        assert_eq!(result.remaining, EMPTY_REMAINING);
    }

    #[test]
    fn preserves_raw_value_after_normalized_short_names() {
        let value =
            Value::<String, _>::new(Description::new("--", "output", "-", "é", None, None), None);
        {
            let result = value.parse_short_form(
                [OsString::from("-e\u{301}=value")].into_iter().map(ArgumentInput::Unchanged),
            );
            assert_eq!(result.values.len(), 1);
            assert_eq!(result.values.values().next().unwrap().value, "value");
            assert_eq!(result.remaining, EMPTY_REMAINING);
        }
        {
            let result =
                value.parse_merged_short_forms([OsString::from("-e\u{301}=value")].into_iter());
            assert_eq!(result.values.len(), 1);
            assert_eq!(result.values.values().next().unwrap().value, "value");
            assert_eq!(result.remaining, EMPTY_REMAINING);
        }
    }

    #[test]
    fn takes_and_parses_its_raw_value() {
        let value =
            Value::<u16, _>::new(Description::new("--", "port", "-", "p", None, None), None);
        let mut result = value.parse_long_form(
            [OsString::from("--port=8080")].into_iter().map(ArgumentInput::Unchanged),
        );

        assert_eq!(value.take_parsed_value(&mut result.values).unwrap(), Some(8080));
        assert!(result.values.is_empty());
    }

    #[test]
    fn annotates_value_conversion_errors() {
        let value =
            Value::<u16, _>::new(Description::new("--", "port", "-", "p", None, None), None);
        let mut result = value.parse_long_form(
            [OsString::from("--port=invalid")].into_iter().map(ArgumentInput::Unchanged),
        );

        let error = value.take_parsed_value(&mut result.values).unwrap_err();
        assert_eq!(error.name.long.head.as_str(), "port");
        assert!(matches!(error.error, ParseError::ParseError(_)));
    }

    #[test]
    fn returns_none_when_its_value_was_not_matched() {
        let value =
            Value::<u16, _>::new(Description::new("--", "port", "-", "p", None, None), None);
        let mut result = value.parse_long_form(
            [OsString::from("--other=8080")].into_iter().map(ArgumentInput::Unchanged),
        );

        assert_eq!(value.take_parsed_value(&mut result.values).unwrap(), None);
    }

    #[test]
    fn keeps_the_last_duplicate_raw_value() {
        let value =
            Value::<u16, _>::new(Description::new("--", "port", "-", "p", None, None), None);
        let mut result = value.parse_long_form(
            [OsString::from("--port=8080"), OsString::from("--port=9090")]
                .into_iter()
                .map(ArgumentInput::Unchanged),
        );

        assert_eq!(value.take_parsed_value(&mut result.values).unwrap(), Some(9090));
    }

    #[test]
    fn parses_a_value_borrowing_from_its_definition_lifetime() {
        #[derive(Debug, PartialEq)]
        struct Borrowed<'value>(&'value str);

        let borrowed = String::from("definition value");
        let parse: Box<dyn Parse<'_, Borrowed<'_>, ParseError<()>>> =
            Box::new(|_: &OsStr| Ok(Borrowed(borrowed.as_str())));
        let value = Value {
            description: Description::new("--", "borrowed", "-", "b", None, None),
            value_infix: "=".into(),
            meta_var: "BORROWED",
            possible_values: None,
            parse: Cow::Owned(parse),
            display: Cow::Borrowed(&|value: &Borrowed<'_>| value.0.into()),
            display_error: Cow::Borrowed(&|(): &()| String::new()),
        };
        let mut result = value.parse_long_form(
            [OsString::from("--borrowed=input")].into_iter().map(ArgumentInput::Unchanged),
        );

        assert_eq!(
            value.take_parsed_value(&mut result.values).unwrap(),
            Some(Borrowed("definition value"))
        );
    }
}
