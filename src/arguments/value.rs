//! Value arguments.

use std::{
    borrow::Cow,
    collections::HashMap,
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
    str::FromStr,
};

use nonempty::NonEmpty;

use crate::{
    ParseFehler,
    arguments::{
        ParseResult, ParsedValue, ParsedValueName,
        help::{Help, Hilfe},
    },
    description::{ArgumentInput, Beschreibung, Description, Name},
    dyn_to_owned::{Parse, Show},
    language::{Language, Sprache},
    outcome::ParseError,
    unicode::{Compare, Vergleich},
};

#[rustfmt::skip]
#[cfg(any(feature = "derive", all(doc, not(doctest))))]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use kommandozeilen_argumente_derive::EnumArgument;

/// A type with a fixed set of values and a parsing method.
/// Intended for sum types containing only unit variants.
///
/// With the `derive` feature, its implementation can be
/// [generated automatically](derive@EnumArgument).
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
#[must_use]
pub struct Value<'t, T, Error> {
    /// General description of the argument.
    pub description: Description<'t, T>,
    /// Infix that separates an inline value from its name.
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
#[must_use]
pub struct Wert<'t, T, Fehler> {
    /// Allgemeine Beschreibung des Arguments.
    pub beschreibung: Beschreibung<'t, T>,
    /// Infix, das einen Wert im selben Argument vom Namen trennt.
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
            Box::new(move |value: &OsStr| parse(value).map_err(ParseFehler::from));
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
            Box::new(move |value: &OsStr| parse(value).map_err(ParseError::from));
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

fn parse_short_value_forms<'definition, 'argument, T, E>(
    name: &Name<'argument>,
    value_infix: &Compare<'argument>,
    args: impl Iterator<Item = OsString>,
) -> ParseResult<'definition, 'argument, T, E> {
    let mut values = HashMap::new();
    let mut args = args.peekable();
    let mut remaining = Vec::new();
    while let Some(argument) = args.next() {
        let input = Cow::Owned(argument.to_string_lossy().into_owned());
        if let Some((name, inline_value)) = name.parse_short_with_value(value_infix, &argument) {
            if let Some(value) =
                inline_value.map(|value| Cow::Owned(value.to_string_lossy().into_owned())).or_else(
                    || args.next().map(|value| Cow::Owned(value.to_string_lossy().into_owned())),
                )
            {
                let _ = values.insert(
                    ParsedValueName { name: Cow::Owned(name.into()) },
                    ParsedValue { value, input },
                );
            }
        } else {
            remaining.push(ArgumentInput::Unchanged(argument));
        }
    }
    ParseResult {
        definition: None,
        early_exits: Vec::new(),
        flags: Vec::new(),
        values,
        remaining,
    }
}

fn parse_long_value_forms<'definition, 'argument, T, E>(
    name: &Name<'argument>,
    value_infix: &Compare<'argument>,
    args: impl Iterator<Item = OsString>,
) -> ParseResult<'definition, 'argument, T, E> {
    let mut values = HashMap::new();
    let mut args = args.peekable();
    let mut remaining = Vec::new();
    while let Some(argument) = args.next() {
        let input = Cow::Owned(argument.to_string_lossy().into_owned());
        if let Some((name, inline_value)) = name.parse_long_with_value(value_infix, &argument) {
            if let Some(value) =
                inline_value.map(|value| Cow::Owned(value.to_string_lossy().into_owned())).or_else(
                    || args.next().map(|value| Cow::Owned(value.to_string_lossy().into_owned())),
                )
            {
                let _ = values.insert(
                    ParsedValueName { name: Cow::Owned(name.into()) },
                    ParsedValue { value, input },
                );
            }
        } else {
            remaining.push(ArgumentInput::Unchanged(argument));
        }
    }
    ParseResult {
        definition: None,
        early_exits: Vec::new(),
        flags: Vec::new(),
        values,
        remaining,
    }
}

fn parse_value_merged_short_forms<'definition, 'argument, T, E>(
    name: &Name<'argument>,
    value_infix: &Compare<'argument>,
    args: impl Iterator<Item = OsString>,
) -> ParseResult<'definition, 'argument, T, E> {
    let mut values = HashMap::new();
    let mut args = args.peekable();
    let mut remaining = Vec::new();
    while let Some(argument) = args.next() {
        let input = Cow::Owned(argument.to_string_lossy().into_owned());
        let argument = ArgumentInput::Unchanged(argument);
        if let Some(parsed) = name.parse_with_value_merge_short_forms(value_infix, &argument) {
            let value = parsed.inline_value.map_or_else(
                || args.next().map(|value| Cow::Owned(value.to_string_lossy().into_owned().into())),
                |value| Some(Cow::Owned(value.into())),
            );
            if let Some(value) = value {
                let _ = values.insert(
                    ParsedValueName { name: Cow::Owned(parsed.name.into()) },
                    ParsedValue { value, input },
                );
            }
            if let Some(argument) = parsed.remaining {
                remaining.push(argument);
            }
        } else {
            remaining.push(argument);
        }
    }
    ParseResult {
        definition: None,
        early_exits: Vec::new(),
        flags: Vec::new(),
        values,
        remaining,
    }
}
/// Creates syntax and help text from the shared parts of a German or English value argument.
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
    let mut syntax = format!("{}", long_prefix.as_str());
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
    /// Creates this argument's syntax and help text.
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

    /// Parses standalone short-form arguments.
    #[inline]
    pub fn parse_short_form(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseResult<'_, 't, T, E> {
        parse_short_value_forms(&self.description.name, &self.value_infix, args)
    }

    /// Parses long-form arguments.
    #[inline]
    pub fn parse_long_form(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseResult<'_, 't, T, E> {
        parse_long_value_forms(&self.description.name, &self.value_infix, args)
    }

    /// Parses merged short-form arguments.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseResult<'_, 't, T, E> {
        parse_value_merged_short_forms(&self.description.name, &self.value_infix, args)
    }
}
#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::{Description, arguments::value::Value};

    #[test]
    fn parses_merged_short_value_with_and_without_infix() {
        let value =
            Value::<String, _>::new(Description::new("--", "output", "-", "o", None, None), None);
        for input in ["-ovalue", "-o=value"] {
            let result = value.parse_merged_short_forms([OsString::from(input)].into_iter());
            assert_eq!(result.values.len(), 1);
            assert_eq!(result.values.values().next().unwrap().value, "value");
            assert!(result.remaining.is_empty());
        }
    }

    #[test]
    fn parses_standalone_short_values() {
        let value =
            Value::<String, _>::new(Description::new("--", "output", "-", "o", None, None), None);
        for input in ["-ovalue", "-o=value"] {
            let result = value.parse_short_form([OsString::from(input)].into_iter());
            assert_eq!(result.values.len(), 1);
            assert_eq!(result.values.values().next().unwrap().value, "value");
            assert!(result.remaining.is_empty());
        }
        let result =
            value.parse_short_form([OsString::from("-o"), OsString::from("value")].into_iter());
        assert_eq!(result.values.values().next().unwrap().value, "value");

        let result = value.parse_short_form([OsString::from("--output")].into_iter());
        assert!(result.values.is_empty());
        assert_eq!(result.remaining.len(), 1);
    }

    #[test]
    fn consumes_the_next_argument_for_a_merged_short_value() {
        let value =
            Value::<String, _>::new(Description::new("--", "output", "-", "o", None, None), None);
        let result = value
            .parse_merged_short_forms([OsString::from("-o"), OsString::from("value")].into_iter());

        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values.values().next().unwrap().value, "value");
        assert!(result.remaining.is_empty());
    }

    #[test]
    fn parses_long_values_with_inline_and_next_argument_values() {
        let value =
            Value::<String, _>::new(Description::new("--", "output", "-", "o", None, None), None);
        let result = value.parse_long_form([OsString::from("--output=value")].into_iter());
        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values.values().next().unwrap().value, "value");
        assert!(result.remaining.is_empty());
        let result = value
            .parse_long_form([OsString::from("--output"), OsString::from("value")].into_iter());
        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values.values().next().unwrap().value, "value");
        assert!(result.remaining.is_empty());
    }

    #[test]
    fn preserves_raw_value_after_a_normalized_long_name() {
        let value =
            Value::<String, _>::new(Description::new("--", "café", "-", "c", None, None), None);
        let result = value.parse_long_form([OsString::from("--cafe\u{301}=value")].into_iter());

        assert_eq!(result.values.len(), 1);
        assert_eq!(result.values.values().next().unwrap().value, "value");
        assert!(result.remaining.is_empty());
    }

    #[test]
    fn preserves_raw_value_after_normalized_short_names() {
        let value =
            Value::<String, _>::new(Description::new("--", "output", "-", "é", None, None), None);
        for parse in [Value::parse_short_form, Value::parse_merged_short_forms] {
            let result = parse(&value, [OsString::from("-e\u{301}=value")].into_iter());
            assert_eq!(result.values.len(), 1);
            assert_eq!(result.values.values().next().unwrap().value, "value");
            assert!(result.remaining.is_empty());
        }
    }
}

impl<'t, T, E> Wert<'t, T, E> {
    /// Parst eigenständige kurze Argumentformen.
    #[inline]
    pub fn parse_short_form(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseResult<'_, 't, T, E> {
        let value_infix = self.wert_infix.clone().into();
        parse_short_value_forms(&self.beschreibung.name, &value_infix, args)
    }

    /// Parst lange Argumentformen.
    #[inline]
    pub fn parse_long_form(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseResult<'_, 't, T, E> {
        let value_infix = self.wert_infix.clone().into();
        parse_long_value_forms(&self.beschreibung.name, &value_infix, args)
    }

    /// Parst zusammengefasste kurze Argumentformen.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseResult<'_, 't, T, E> {
        let value_infix = self.wert_infix.clone().into();
        parse_value_merged_short_forms(&self.beschreibung.name, &value_infix, args)
    }

    /// Erzeugt Syntax und Hilfetext für dieses Argument.
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
