//! Implementations of [`ParseArgument`] for directly supported value types.

use std::{
    borrow::Cow,
    convert::identity,
    ffi::{OsStr, OsString},
    fmt::Display,
    str::FromStr,
};

#[cfg(feature = "german")]
use crate::{arguments::argumente::Argumente, description::Beschreibung, language::Sprache};
use crate::{
    arguments::{
        Arguments,
        flag::Flag,
        value::{EnumArgument, Value},
    },
    description::Description,
    dyn_to_owned,
    language::Language,
    outcome::ParseError,
    unicode::Compare,
};

/// Trait for types directly usable with the `#[derive(Parse)]` macro for [`crate::Parse`].
///
/// ## Deutsch
/// Trait für Typen, die direkt mit dem `#[derive(Parse)]`-Makro für [`Parse`] verwendet werden
/// können.
pub trait ParseArgument: Sized {
    /// Creates [`Arguments`] with the configured properties.
    ///
    /// `invert_prefix` is intended as the prefix to invert flag arguments, and `meta_var` is the
    /// metavariable used in help text for value arguments.
    ///
    /// ## Deutsch
    /// Erstellt [`Arguments`] mit den konfigurierten Eigenschaften.
    ///
    /// `invert_prefix` ist für Flag-Argumente gedacht, `meta_var` ist die Meta-Variable für
    /// Wert-Argumente im Hilfetext.
    fn arguments<'t>(
        description: Description<'t, Self>,
        invert_prefix: impl Into<Compare<'t>>,
        invert_infix: impl Into<Compare<'t>>,
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
    ) -> Arguments<'t, Self, String>;

    /// Returns the default value for an omitted argument, if it has one.
    ///
    /// ## Deutsch
    /// Gibt den Standardwert für ein nicht angegebenes Argument zurück, falls vorhanden.
    fn default() -> Option<Self>;

    /// Creates an argument configuration with localized defaults.
    ///
    /// ## Deutsches Synonym
    /// [`argumente_mit_sprache`](ParseArgument::argumente_mit_sprache)
    #[inline]
    fn arguments_with_language(
        description: Description<'_, Self>,
        language: Language,
    ) -> Arguments<'_, Self, String> {
        Self::arguments(
            description,
            language.invert_prefix,
            language.invert_infix,
            language.value_infix,
            language.meta_var,
        )
    }

    /// Creates a German mirror argument configuration with localized defaults.
    ///
    /// ## Deutsch
    /// Erstellt eine deutsche Spiegel-Konfiguration mit lokalisierten Standardwerten.
    ///
    /// ## English synonym
    /// [`arguments_with_language`](ParseArgument::arguments_with_language)
    #[inline]
    #[cfg(feature = "german")]
    fn argumente_mit_sprache(
        beschreibung: Beschreibung<'_, Self>,
        sprache: Sprache,
    ) -> Argumente<'_, Self, String> {
        Self::arguments_with_language(beschreibung.into(), sprache.into()).into()
    }

    /// Creates an argument configuration with English defaults.
    ///
    /// ## Deutsche Version
    /// [`neu`](ParseArgument::neu)
    #[inline]
    fn new(description: Description<'_, Self>) -> Arguments<'_, Self, String> {
        Self::arguments_with_language(description, Language::ENGLISH)
    }

    /// Creates a German mirror argument configuration with German defaults.
    ///
    /// ## Deutsch
    /// Erstellt eine deutsche Spiegel-Konfiguration mit deutschen Standardwerten.
    ///
    /// ## English version
    /// [`new`](ParseArgument::new)
    #[inline]
    #[cfg(feature = "german")]
    fn neu(beschreibung: Beschreibung<'_, Self>) -> Argumente<'_, Self, String> {
        Self::argumente_mit_sprache(beschreibung, Sprache::DEUTSCH)
    }
}

impl ParseArgument for bool {
    #[inline]
    fn arguments<'t>(
        description: Description<'t, Self>,
        invert_prefix: impl Into<Compare<'t>>,
        invert_infix: impl Into<Compare<'t>>,
        _value_infix: impl Into<Compare<'t>>,
        _meta_var: &'t str,
    ) -> Arguments<'t, Self, String> {
        Arguments::from(Flag {
            description,
            invert_prefix: invert_prefix.into(),
            invert_infix: invert_infix.into(),
            display: Cow::Borrowed(&<bool as ToString>::to_string),
            convert: Cow::Borrowed(&identity),
        })
    }

    #[inline]
    fn default() -> Option<Self> {
        Some(false)
    }
}

impl ParseArgument for String {
    #[inline]
    fn arguments<'t>(
        description: Description<'t, Self>,
        _invert_prefix: impl Into<Compare<'t>>,
        _invert_infix: impl Into<Compare<'t>>,
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
    ) -> Arguments<'t, Self, String> {
        Arguments::from(Value {
            description,
            value_infix: value_infix.into(),
            meta_var,
            possible_values: None,
            parse: Cow::Borrowed(&|os_str: &OsStr| {
                if let Some(string) = os_str.to_str() {
                    Ok(String::from(string))
                } else {
                    Err(ParseError::InvalidString(OsString::from(os_str)))
                }
            }),
            display: Cow::Borrowed(&<String as Clone>::clone),
            display_error: Cow::Borrowed(&<String as Clone>::clone),
        })
    }

    #[inline]
    fn default() -> Option<Self> {
        None
    }
}

/// Implements [`ParseArgument`] for primitive numeric types.
///
/// ## Deutsch
/// Implementiert [`ParseArgument`] für primitive Zahlentypen.
macro_rules! impl_parse_argument {
    ($($type:ty),*$(,)?) => {$(
        impl ParseArgument for $type {
            #[inline]
            fn arguments<'t>(
                description: Description<'t,Self>,
                _invert_prefix: impl Into<Compare<'t>>,
                _invert_infix: impl Into<Compare<'t>>,
                value_infix: impl Into<Compare<'t>>,
                meta_var: &'t str,
            ) -> Arguments<'t, Self, String> {
                Arguments::from(Value {
                    description,
                    value_infix: value_infix.into(),
                    meta_var,
                    possible_values: None,
                    parse: Cow::Borrowed(&|os_str: &OsStr| {
                        if let Some(string) = os_str.to_str() {
                            string.parse().map_err(
                                |err: <$type as FromStr>::Err| ParseError::ParseError(err.to_string()).into()
                            )
                        } else {
                            Err(ParseError::InvalidString(os_str.to_owned()).into())
                        }
                    }),
                    display: Cow::Borrowed(&<$type as ToString>::to_string),
                    display_error: Cow::Borrowed(&<String as Clone>::clone),
                })
            }

            #[inline]
            fn default() -> Option<Self> {
                None
            }
        }
    )*};
}
impl_parse_argument! {i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize, f32, f64}

impl<T: EnumArgument + Display + Clone> ParseArgument for T {
    #[inline]
    fn arguments<'t>(
        description: Description<'t, Self>,
        _invert_prefix: impl Into<Compare<'t>>,
        _invert_infix: impl Into<Compare<'t>>,
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
    ) -> Arguments<'t, Self, String> {
        let boxed_parse: Box<dyn dyn_to_owned::Parse<'t, T, ParseError<String>>> =
            Box::new(move |os_str: &OsStr| {
                let Some(string) = os_str.to_str() else {
                    return Err(ParseError::InvalidString(OsString::from(os_str)));
                };
                <T as EnumArgument>::varianten()
                    .into_iter()
                    .flatten()
                    .find(
                        #[expect(clippy::min_ident_chars, reason = "t: T")]
                        |t| t.to_string() == string,
                    )
                    .ok_or_else(|| ParseError::ParseError(String::from(string)))
            });
        Arguments::from(Value {
            description,
            value_infix: value_infix.into(),
            meta_var,
            possible_values: <T as EnumArgument>::varianten(),
            parse: Cow::Owned(boxed_parse),
            display: Cow::Borrowed(&<T as ToString>::to_string),
            display_error: Cow::Borrowed(&<String as Clone>::clone),
        })
    }

    #[inline]
    fn default() -> Option<Self> {
        None
    }
}
