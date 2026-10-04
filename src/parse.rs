//! Traits for types that can be parsed from command-line arguments.

use std::{
    borrow::Cow,
    convert::identity,
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    num::NonZeroI32,
    str::FromStr,
};

use nonempty::NonEmpty;

use crate::{
    arguments::{
        Arguments,
        argumente::Argumente,
        combine::Combine,
        flag::Flag,
        help::{self, CreateHelpText},
        single_argument::SingleArgument,
        value::{EnumArgument, Value},
    },
    description::{ArgumentInput, Beschreibung, Description},
    dyn_to_owned::{self, Bool, Show},
    language::{Language, Sprache},
    outcome::{Ergebnis, Error, Fehler, ParseError, Result},
    unicode::Compare,
};

#[rustfmt::skip]
#[cfg(any(feature = "derive", all(doc, not(doctest))))]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use kommandozeilen_argumente_derive::Parse;

/// Trait for types directly usable with the [derive macro](derive@Parse) for [`Parse`].
///
/// ## Deutsch
/// Trait für Typen, die direkt mit dem [derive-Makro](derive@Parse) für [`Parse`] verwendet
/// werden können.
#[allow(clippy::module_name_repetitions)]
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
    #[allow(clippy::needless_lifetimes)]
    fn arguments_with_language<'t>(
        description: Description<'t, Self>,
        language: Language,
    ) -> Arguments<'t, Self, String> {
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
    #[allow(clippy::needless_lifetimes)]
    fn argumente_mit_sprache<'t>(
        beschreibung: Beschreibung<'t, Self>,
        sprache: Sprache,
    ) -> Argumente<'t, Self, String> {
        Self::arguments_with_language(beschreibung.into(), sprache.into()).into()
    }

    /// Creates an argument configuration with English defaults.
    ///
    /// ## Deutsche Version
    /// [`neu`](ParseArgument::neu)
    #[inline]
    #[allow(clippy::needless_lifetimes)]
    fn new<'t>(description: Description<'t, Self>) -> Arguments<'t, Self, String> {
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
    #[allow(clippy::needless_lifetimes)]
    fn neu<'t>(beschreibung: Beschreibung<'t, Self>) -> Argumente<'t, Self, String> {
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
            description: description.into(),
            invert_prefix: invert_prefix.into().into(),
            invert_infix: invert_infix.into().into(),
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
                    Err(ParseError::InvalidString(OsString::from(os_str)).into())
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

/// Helper type for the [`ParseArgument`] implementation of [`Option<T>`].
struct OptionArguments<'t, F, T, E> {
    /// Adjusts the result of parsing an [`Option<T>`].
    adjust_result: F,
    /// Argument definition used to parse an [`Option<T>`].
    arguments: Arguments<'t, T, E>,
}

impl<'t, T, E, F: FnOnce(Result<'t, T, E>) -> Result<'t, T, E>> Combine<'t, T, E>
    for OptionArguments<'t, F, T, E>
{
    #[inline]
    fn create_help_text(
        &self,
        variant: &dyn CreateHelpText,
        meta_default: &str,
        meta_allowed_values: &str,
    ) -> NonEmpty<help::Alternativen> {
        self.arguments.create_help_text(variant, meta_default, meta_allowed_values).map(Into::into)
    }
}

/// Creates the [`Show`] closure for [`Option<T>`] as a trait object.
fn create_boxed_option_display<'t, T>(
    display: Cow<'t, dyn Show<'t, T>>,
) -> Box<dyn 't + Show<'t, Option<T>>> {
    Box::new(move |opt: &Option<T>| {
        #[allow(clippy::min_ident_chars)]
        if let Some(t) = opt { display(t) } else { String::from("None") }
    })
}

/// Creates the closure that adjusts the final [`Result`] for an [`Option<T>`].
fn create_result_adjuster<'t, T: Clone>(
    default: Option<T>,
) -> impl FnOnce(Result<'t, T, String>) -> Result<'t, T, String> {
    |result| match (result, default) {
        (Result::Error(errors), Some(default)) => {
            let mut final_result = Some(Result::Value(default.clone()));
            for error in &errors {
                if let Error::ParseError(_parse_error) = error {
                    final_result = None;
                    break;
                }
            }
            if let Some(final_result) = final_result { final_result } else { Result::Error(errors) }
        },
        (result, _) => result,
    }
}

/// Creates the [`Description`] used to invoke [`ParseArgument::arguments`] for an [`Option<T>`].
fn create_description<'t, T>(description: &Description<'t, Option<T>>) -> Description<'t, T> {
    let name_long_prefix = description.name.long_prefix.clone();
    let long = description.name.long.clone();
    let name_short_prefix = description.name.short_prefix.clone();
    let short = description.name.short.clone();
    Description::new(
        name_long_prefix,
        long.clone(),
        name_short_prefix,
        short.clone(),
        None::<&str>,
        None,
    )
}

impl<T: 'static + ParseArgument + Clone + Debug + Display> ParseArgument for Option<T> {
    #[inline]
    fn arguments<'t>(
        description: Description<'t, Self>,
        invert_prefix: impl Into<Compare<'t>>,
        invert_infix: impl Into<Compare<'t>>,
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
    ) -> Arguments<'t, Self, String> {
        let value_infix_compare = value_infix.into();
        let arguments = <T as ParseArgument>::arguments(
            create_description(&description),
            invert_prefix,
            invert_infix,
            value_infix_compare,
            meta_var,
        );
        let adjust_result = create_result_adjuster(description.default.clone());
        #[allow(clippy::shadow_unrelated)]
        match arguments {
            Arguments::Single(SingleArgument::Flag(Flag {
                description: _,
                invert_prefix,
                invert_infix,
                convert,
                display,
            })) => {
                let boxed_convert: Box<dyn Bool<'t, Option<T>>> =
                    Box::new(move |value| Some(convert(value)));
                Arguments::Single(SingleArgument::Flag(Flag {
                    description: description.into(),
                    invert_prefix,
                    invert_infix,
                    convert: Cow::Owned(boxed_convert),
                    display: Cow::Owned(create_boxed_option_display(display)),
                }))
            },
            Arguments::Single(SingleArgument::EarlyExit { early_exit, value, display }) => {
                // `default` is guaranteed to be [`None`], so `description` can be moved.
                Arguments::Single(SingleArgument::EarlyExit {
                    early_exit,
                    value: Some(value),
                    display: Cow::Owned(create_boxed_option_display(display)),
                })
            },
            Arguments::Single(SingleArgument::Value(Value {
                description: _,
                value_infix,
                meta_var,
                possible_values,
                parse,
                display,
                display_error,
            })) => {
                let boxed_parse: Box<dyn dyn_to_owned::Parse<'t, Option<T>, ParseError<String>>> =
                    Box::new(move |os_str: &OsStr| match parse(os_str) {
                        Ok(value) => Ok(Some(value)),
                        Err(_error) if os_str == "None" => Ok(None),
                        Err(error) => Err(error),
                    });
                let possible_values = possible_values.map(|nonempty| {
                    let mut nonempty = nonempty.map(Some);
                    nonempty.push(None);
                    nonempty
                });
                let value = Arguments::from(Value {
                    description,
                    value_infix,
                    meta_var,
                    possible_values,
                    parse: Cow::Owned(boxed_parse),
                    display: Cow::Owned(create_boxed_option_display(display)),
                    display_error,
                });
                Arguments::combine(OptionArguments { adjust_result, arguments: value })
            },
            Arguments::Combined(combine) => Arguments::combine(OptionArguments {
                adjust_result,
                arguments: Arguments::combine((Some, Arguments::Combined(combine))),
            }),
            Arguments::Alternatives(alternatives) => Arguments::combine(OptionArguments {
                adjust_result,
                arguments: Arguments::combine((Some, Arguments::Alternatives(alternatives))),
            }),
        }
    }

    #[inline]
    fn default() -> Option<Self> {
        Some(None)
    }
}

impl<T: 'static + EnumArgument + Display + Clone> ParseArgument for T {
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
                    return Err(ParseError::InvalidString(OsString::from(os_str)).into());
                };
                <T as EnumArgument>::varianten()
                    .into_iter()
                    .flatten()
                    .find(
                        #[allow(clippy::min_ident_chars)]
                        |t| t.to_string() == string,
                    )
                    .ok_or_else(|| ParseError::ParseError(String::from(string)).into())
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

/// Allows parsing from command-line arguments based on a default configuration.
///
/// With the `derive` feature, an implementation can be [generated automatically](derive@Parse).
///
/// ## Deutsch
/// Erlaubt das Parsen von Kommandozeilen-Argumenten anhand einer Standardkonfiguration.
///
/// Mit dem `derive`-Feature kann eine Implementierung [automatisch erzeugt werden](derive@Parse).
pub trait Parse: Sized {
    /// Possible parse error, the automatically created implementation uses [`String`].
    ///
    /// ## Deutsch
    /// Möglicher Parse-Fehler, die automatisch erzeugte Implementierung verwendet [`String`].
    type Error;

    /// Create a description, how command line arguments should be parsed.
    ///
    /// ## Deutsch
    /// Erzeuge eine Beschreibung, wie Kommandozeilen-Argumente geparst werden sollen.
    fn arguments<'t>() -> Arguments<'t, Self, Self::Error>;

    /// Parse the given command line arguments to create the requested type.
    ///
    /// ## Deutsch
    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    #[inline]
    fn parse<'t>(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<'t, Self, Self::Error>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        Self::arguments().parse(args)
    }

    /// Parse [`args_os`](std::env::args_os) and try to create the requested type.
    ///
    /// ## Deutsches Synonym
    /// [`parse_aus_env`](Parse::parse_aus_env)
    #[inline]
    fn parse_from_env<'t>() -> (Result<'t, Self, Self::Error>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        Self::arguments().parse_from_env()
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    ///
    /// ## English synonym
    /// [`parse_from_env`](Parse::parse_from_env)
    #[inline]
    fn parse_aus_env<'t>() -> (Ergebnis<'t, Self, Self::Error>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        let (result, remaining) = Self::parse_from_env();
        (result.into(), remaining)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_mit_frühen_beenden`](Parse::parse_mit_frühen_beenden)
    #[inline]
    #[allow(clippy::type_complexity)]
    fn parse_with_early_exit<'t>(
        args: impl Iterator<Item = OsString>,
    ) -> (std::result::Result<Self, NonEmpty<Error<'t, Self::Error>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        Self::arguments().parse_with_early_exit(args)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [`parse_with_early_exit`](Parse::parse_with_early_exit)
    #[inline]
    #[allow(clippy::type_complexity)]
    fn parse_mit_frühen_beenden<'t>(
        args: impl Iterator<Item = OsString>,
    ) -> (std::result::Result<Self, NonEmpty<Fehler<'t, Self::Error>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        let (result, remaining) = Self::parse_with_early_exit(args);
        (result.map_err(|errors| errors.map(Into::into)), remaining)
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_aus_env_mit_frühen_beenden`](Argumente::parse_aus_env_mit_frühen_beenden)
    #[inline]
    #[allow(clippy::type_complexity)]
    fn parse_from_env_with_early_exit<'t>()
    -> (std::result::Result<Self, NonEmpty<Error<'t, Self::Error>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        Self::arguments().parse_from_env_with_early_exit()
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [`parse_from_env_with_early_exit`](Parse::parse_from_env_with_early_exit)
    #[inline]
    #[allow(clippy::type_complexity)]
    fn parse_aus_env_mit_frühen_beenden<'t>()
    -> (std::result::Result<Self, NonEmpty<Fehler<'t, Self::Error>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        let (result, remaining) = Self::parse_from_env_with_early_exit();
        (result.map_err(|errors| errors.map(Into::into)), remaining)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig`](Parse::parse_vollständig)
    #[inline]
    #[must_use]
    fn parse_complete(
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_complete(
            args,
            error_code,
            missing_flag,
            missing_value,
            parse_error,
            invalid_string,
            unused_arg,
        )
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
    ///
    /// ## English synonym
    /// [`parse_complete`](Parse::parse_complete)
    #[inline]
    #[must_use]
    fn parse_vollständig(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        arg_nicht_verwendet: &str,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_complete(
            args,
            fehler_code,
            fehlende_flag,
            fehlender_wert,
            parse_fehler,
            invalider_string,
            arg_nicht_verwendet,
        )
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig_mit_sprache`](Parse::parse_vollständig_mit_sprache)
    #[inline]
    #[must_use]
    fn parse_complete_with_language(
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        language: Language,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_complete_with_language(args, error_code, language)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
    ///
    /// ## English synonym
    /// [`parse_complete_with_language`](Parse::parse_complete_with_language)
    #[inline]
    #[must_use]
    fn parse_vollständig_mit_sprache(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        sprache: Sprache,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_complete_with_language(args, fehler_code, sprache.into())
    }

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// [`parse_complete_with_language`](Parse::parse_complete_with_language) with
    /// [`Language::ENGLISH`].
    ///
    /// ## Deutsche version
    /// [`parse_mit_fehlermeldung`](Parse::parse_mit_fehlermeldung)
    #[inline]
    #[must_use]
    fn parse_with_error_message(
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_with_error_message(args, error_code)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
    ///
    /// [`parse_vollständig_mit_sprache`](Parse::parse_vollständig_mit_sprache) mit
    /// [`Sprache::DEUTSCH`].
    ///
    /// ## English version
    /// [`parse_with_error_message`](Parse::parse_with_error_message)
    #[inline]
    #[must_use]
    fn parse_mit_fehlermeldung(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_vollständig_mit_sprache(args, fehler_code, Sprache::DEUTSCH)
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig_aus_env`](Parse::parse_vollständig_aus_env)
    #[inline]
    #[must_use]
    fn parse_complete_from_env(
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_complete_from_env(
            error_code,
            missing_flag,
            missing_value,
            parse_error,
            invalid_string,
            unused_arg,
        )
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
    ///
    /// ## English synonym
    /// [`parse_complete_from_env`](Parse::parse_complete_from_env)
    #[inline]
    #[must_use]
    fn parse_vollständig_aus_env(
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        arg_nicht_verwendet: &str,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_complete_from_env(
            fehler_code,
            fehlende_flag,
            fehlender_wert,
            parse_fehler,
            invalider_string,
            arg_nicht_verwendet,
        )
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig_mit_sprache_aus_env`](Parse::parse_vollständig_mit_sprache_aus_env)
    #[inline]
    #[must_use]
    fn parse_complete_with_language_from_env(error_code: NonZeroI32, language: Language) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_complete_with_language_from_env(error_code, language)
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
    ///
    /// ## English synonym
    /// [`parse_complete_with_language_from_env`](Parse::parse_complete_with_language_from_env)
    #[inline]
    #[must_use]
    fn parse_vollständig_mit_sprache_aus_env(fehler_code: NonZeroI32, sprache: Sprache) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_complete_with_language_from_env(fehler_code, sprache.into())
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// [`parse_complete_with_language_from_env`](Parse::parse_complete_with_language_from_env)
    /// with [`Language::ENGLISH`].
    ///
    /// ## Deutsche Version
    /// [`parse_mit_fehlermeldung_aus_env`](Parse::parse_mit_fehlermeldung_aus_env)
    #[inline]
    #[must_use]
    fn parse_with_error_message_from_env(error_code: NonZeroI32) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_with_error_message_from_env(error_code)
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
    ///
    /// [`parse_vollständig_mit_sprache_aus_env`](Parse::parse_vollständig_mit_sprache_aus_env)
    /// mit [`Sprache::DEUTSCH`].
    ///
    /// ## English version
    /// [`parse_with_error_message_from_env`](Parse::parse_with_error_message_from_env)
    #[inline]
    #[must_use]
    fn parse_mit_fehlermeldung_aus_env(fehler_code: NonZeroI32) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_vollständig_mit_sprache_aus_env(fehler_code, Sprache::DEUTSCH)
    }
}
