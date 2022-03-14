//! Trait für Typen, die aus Kommandozeilen-Argumenten geparst werden können.

use std::{ffi::OsString, fmt::Display, num::NonZeroI32, str::FromStr};

use nonempty::NonEmpty;

use crate::{
    argumente::{wert::EnumArgument, Argumente, Arguments},
    beschreibung::{Beschreibung, Description, Konfiguration},
    ergebnis::{Ergebnis, Error, Fehler, ParseFehler},
    sprache::{Language, Sprache},
};

#[cfg(any(feature = "derive", all(doc, not(doctest))))]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use kommandozeilen_argumente_derive::Parse;

/// Trait für Typen, die direkt mit dem (derive-Macro)[derive@Parse]
/// für das [Parse]-Trait verwendet werden können.
///
/// ## English
/// Trait for types directly usable with the [derive macro](derive@Parse] for the [Parse] trait.
pub trait ParseArgument: Sized {
    /// Erstelle ein [Argumente] mit den konfigurierten Eigenschaften.
    ///
    /// `invertiere_präfix` ist für Flag-Argumente gedacht,
    /// `meta_var` für Wert-Argumente.
    ///
    /// ## English
    /// Create and [Arguments] with the configured properties.
    ///
    /// `invertiere_präfix` is intended as the prefix to invert flag arguments,
    /// `meta_var` is intended as the meta-variable used in the help text for value arguments.
    fn argumente(
        beschreibung: Beschreibung<Self>,
        invertiere_präfix: &'static str,
        meta_var: &str,
    ) -> Argumente<Self, String>;

    /// Sollen Argumente dieses Typs normalerweise einen Standard-Wert haben?
    ///
    /// ## English
    /// Should arguments of this type have a default value if left unspecified?
    fn standard() -> Option<Self>;

    /// Erstelle ein [Argumente] für die übergebene [Beschreibung].
    ///
    /// ## English synonym
    /// [arguments_with_language](ParseArgument::arguments_with_language)
    #[inline(always)]
    fn argumente_mit_sprache(
        beschreibung: Beschreibung<Self>,
        sprache: Sprache,
    ) -> Argumente<Self, String> {
        Self::argumente(beschreibung, sprache.invertiere_präfix, sprache.meta_var)
    }

    /// Create an [Arguments] for the given [Description].
    ///
    /// ## Deutsches Synonym
    /// [argumente_mit_sprache](ParseArgument::argumente_mit_sprache)
    #[inline(always)]
    fn arguments_with_language(
        description: Description<Self>,
        language: Language,
    ) -> Arguments<Self, String> {
        Self::argumente_mit_sprache(description, language)
    }

    /// Erstelle ein [Argumente] für die übergebene [Beschreibung].
    ///
    /// ## English version
    /// [new](ParseArgument::new)
    #[inline(always)]
    fn neu(beschreibung: Beschreibung<Self>) -> Argumente<Self, String> {
        Self::argumente_mit_sprache(beschreibung, Sprache::DEUTSCH)
    }

    /// Create an [Argumente] for the [Beschreibung].
    ///
    /// ## Deutsche Version
    /// [neu](ParseArgument::neu)
    #[inline(always)]
    fn new(beschreibung: Beschreibung<Self>) -> Argumente<Self, String> {
        Self::argumente_mit_sprache(beschreibung, Sprache::ENGLISH)
    }
}

impl ParseArgument for bool {
    fn argumente(
        beschreibung: Beschreibung<Self>,
        invertiere_präfix: &'static str,
        _meta_var: &str,
    ) -> Argumente<Self, String> {
        Argumente::flag_bool(beschreibung, invertiere_präfix)
    }

    fn standard() -> Option<Self> {
        Some(false)
    }
}

impl ParseArgument for String {
    fn argumente(
        beschreibung: Beschreibung<Self>,
        _invertiere_präfix: &'static str,
        meta_var: &str,
    ) -> Argumente<Self, String> {
        Argumente::wert_display(beschreibung, meta_var.to_owned(), None, |os_str| {
            if let Some(string) = os_str.to_str() {
                Ok(string.to_owned())
            } else {
                Err(ParseFehler::InvaliderString(os_str.to_owned()))
            }
        })
    }

    fn standard() -> Option<Self> {
        None
    }
}

macro_rules! impl_parse_argument {
    ($($type:ty),*$(,)?) => {$(
        impl ParseArgument for $type {
            fn argumente(
                beschreibung: Beschreibung<Self>,
                _invertiere_präfix: &'static str,
                meta_var: &str,
            ) -> Argumente<Self, String> {
                Argumente::wert_display(beschreibung, meta_var.to_owned(), None, |os_str| {
                    if let Some(string) = os_str.to_str() {
                        string.parse().map_err(
                            |err: <$type as FromStr>::Err| ParseFehler::ParseFehler(err.to_string())
                        )
                    } else {
                        Err(ParseFehler::InvaliderString(os_str.to_owned()))
                    }
                })
            }

            fn standard() -> Option<Self> {
                None
            }
        }
    )*};
}
impl_parse_argument! {i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize, f32, f64}

impl<T: 'static + ParseArgument + Clone + Display> ParseArgument for Option<T> {
    fn argumente(
        beschreibung: Beschreibung<Self>,
        invertiere_präfix: &'static str,
        meta_var: &str,
    ) -> Argumente<Self, String> {
        let Beschreibung { lang, kurz, .. } = &beschreibung;
        let lang_namen = lang.clone();
        let kurz_namen = kurz.clone();
        let Argumente { parse, .. } =
            T::argumente(Beschreibung::neu(lang, kurz, None, None), invertiere_präfix, meta_var);
        let (beschreibung_string, option_standard) = beschreibung
            .als_string_beschreibung_allgemein(|opt| {
                if let Some(t) = opt {
                    t.to_string()
                } else {
                    "None".to_owned()
                }
            });
        type F<T> = Box<dyn Fn(NonEmpty<Fehler<String>>) -> Ergebnis<Option<T>, String>>;
        let verwende_standard: F<T> = if let Some(standard) = option_standard {
            Box::new(move |fehler_sammlung| {
                let mut fehler_iter =
                    fehler_sammlung.into_iter().filter_map(|fehler| match fehler {
                        Fehler::FehlenderWert { lang, kurz, meta_var } => {
                            if lang == lang_namen && kurz == kurz_namen {
                                None
                            } else {
                                Some(Fehler::FehlenderWert { lang, kurz, meta_var })
                            }
                        },
                        fehler => Some(fehler),
                    });
                if let Some(head) = fehler_iter.next() {
                    let tail = fehler_iter.collect();
                    Ergebnis::Fehler(NonEmpty { head, tail })
                } else {
                    Ergebnis::Wert(standard.clone())
                }
            })
        } else {
            Box::new(Ergebnis::Fehler)
        };
        Argumente {
            konfigurationen: vec![Konfiguration::Wert {
                beschreibung: beschreibung_string,
                meta_var: meta_var.to_owned(),
                mögliche_werte: None,
            }],
            flag_kurzformen: Vec::new(),
            parse: Box::new(move |args| {
                let (ergebnis, nicht_verwendet) = parse(args);
                let option_ergebnis = match ergebnis {
                    Ergebnis::Wert(wert) => Ergebnis::Wert(Some(wert)),
                    Ergebnis::FrühesBeenden(nachrichten) => Ergebnis::FrühesBeenden(nachrichten),
                    Ergebnis::Fehler(fehler_sammlung) => verwende_standard(fehler_sammlung),
                };
                (option_ergebnis, nicht_verwendet)
            }),
        }
    }

    fn standard() -> Option<Self> {
        Some(None)
    }
}

impl<T: 'static + EnumArgument + Display + Clone> ParseArgument for T {
    fn argumente(
        beschreibung: Beschreibung<Self>,
        _invertiere_präfix: &'static str,
        meta_var: &str,
    ) -> Argumente<Self, String> {
        Argumente::wert_enum_display(beschreibung, meta_var.to_owned())
    }

    fn standard() -> Option<Self> {
        None
    }
}

/// Erlaube parsen aus Kommandozeilen-Argumenten ausgehend einer Standard-Konfiguration.
///
/// Mit aktiviertem `derive`-Feature kann diese [automatisch erzeugt werden](derive@Parse).
///
/// ## English
/// Allow parsing from command line arguments, based on a default configuration.
///
/// With active `derive`-feature, the implementation can be [automatically created](derive@Parse).
pub trait Parse: Sized {
    /// Möglicher Parse-Fehler, die automatisch erzeugte Implementierung verwendet [String].
    ///
    /// ## English
    /// Possible parse error, the automatically created implementation uses [String].
    type Fehler;

    /// Erzeuge eine Beschreibung, wie Kommandozeilen-Argumente geparst werden sollen.
    ///
    /// ## English
    /// Create a description, how command line arguments should be parsed.
    fn kommandozeilen_argumente() -> Argumente<Self, Self::Fehler>;

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    ///
    /// ## English
    /// Parse the given command line arguments to create the requested type.
    #[inline(always)]
    fn parse(
        args: impl Iterator<Item = OsString>,
    ) -> (Ergebnis<Self, Self::Fehler>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse(args)
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    ///
    /// ## English synonym
    /// [parse_from_env](Parse::parse_from_env)
    #[inline(always)]
    fn parse_aus_env() -> (Ergebnis<Self, Self::Fehler>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_aus_env()
    }

    /// Parse [args_os](std::env::args_os) and try to create the requested type.
    ///
    /// ## Deutsches Synonym
    /// [parse_aus_env](Parse::parse_aus_env)
    #[inline(always)]
    fn parse_from_env() -> (Ergebnis<Self, Self::Fehler>, Vec<OsString>) {
        Self::parse_aus_env()
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [parse_from_env_with_early_exit](Parse::parse_from_env_with_early_exit)
    #[inline(always)]
    fn parse_aus_env_mit_frühen_beenden(
    ) -> (Result<Self, NonEmpty<Fehler<Self::Fehler>>>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_aus_env_mit_frühen_beenden()
    }

    /// Parse [args_os](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [parse_aus_env_mit_frühen_beenden](Argumente::parse_aus_env_mit_frühen_beenden)
    #[inline(always)]
    fn parse_from_env_with_early_exit(
    ) -> (Result<Self, NonEmpty<Error<Self::Fehler>>>, Vec<OsString>) {
        Self::parse_aus_env_mit_frühen_beenden()
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [parse_with_early_exit](Parse::parse_with_early_exit)
    #[inline(always)]
    fn parse_mit_frühen_beenden(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<Self, NonEmpty<Fehler<Self::Fehler>>>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_mit_frühen_beenden(args)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [parse_mit_frühen_beenden](Parse::parse_mit_frühen_beenden)
    #[inline(always)]
    fn parse_with_early_exit(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<Self, NonEmpty<Error<Self::Fehler>>>, Vec<OsString>) {
        Self::parse_mit_frühen_beenden(args)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [parse_complete](Parse::parse_complete)
    #[inline(always)]
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
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_vollständig(
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
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [parse_vollständig](Parse::parse_vollständig)
    #[inline(always)]
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
        Self::Fehler: Display,
    {
        Self::parse_vollständig(
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
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [parse_complete_with_language](Parse::parse_complete_with_language)
    #[inline(always)]
    fn parse_vollständig_mit_sprache(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        sprache: Sprache,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_vollständig_mit_sprache(args, fehler_code, sprache)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [parse_vollständig_mit_sprache](Parse::parse_vollständig_mit_sprache)
    #[inline(always)]
    fn parse_complete_with_language(
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        language: Language,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::parse_vollständig_mit_sprache(args, error_code, language)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English version
    /// [parse_with_error_message](Parse::parse_with_error_message)
    #[inline(always)]
    fn parse_mit_fehlermeldung(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_mit_fehlermeldung(args, fehler_code)
    }

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsche version
    /// [parse_mit_fehlermeldung](Parse::parse_mit_fehlermeldung)
    #[inline(always)]
    fn parse_with_error_message(
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_with_error_message(args, error_code)
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [parse_complete_from_env](Parse::parse_complete_from_env)
    #[inline(always)]
    fn parse_vollständig_aus_env(
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        arg_nicht_verwendet: &str,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_vollständig_aus_env(
            fehler_code,
            fehlende_flag,
            fehlender_wert,
            parse_fehler,
            invalider_string,
            arg_nicht_verwendet,
        )
    }

    /// Parse [args_os](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [parse_vollständig_aus_env](Parse::parse_vollständig_aus_env)
    #[inline(always)]
    fn parse_complete_from_env(
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::parse_vollständig_aus_env(
            error_code,
            missing_flag,
            missing_value,
            parse_error,
            invalid_string,
            unused_arg,
        )
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [parse_complete_with_language_from_env](Parse::parse_complete_with_language_from_env)
    #[inline(always)]
    fn parse_vollständig_mit_sprache_aus_env(fehler_code: NonZeroI32, sprache: Sprache) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente()
            .parse_vollständig_mit_sprache_aus_env(fehler_code, sprache)
    }

    /// Parse [args_os](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [parse_vollständig_mit_sprache_aus_env](Parse::parse_vollständig_mit_sprache_aus_env)
    #[inline(always)]
    fn parse_complete_with_language_from_env(error_code: NonZeroI32, language: Language) -> Self
    where
        Self::Fehler: Display,
    {
        Self::parse_vollständig_mit_sprache_aus_env(error_code, language)
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English version
    /// [parse_with_error_message_from_env](Parse::parse_with_error_message_from_env)
    #[inline(always)]
    fn parse_mit_fehlermeldung_aus_env(fehler_code: NonZeroI32) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_mit_fehlermeldung_aus_env(fehler_code)
    }

    /// Parse [args_os](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsche Version
    /// [parse_mit_fehlermeldung_aus_env](Parse::parse_mit_fehlermeldung_aus_env)
    #[inline(always)]
    fn parse_with_error_message_from_env(error_code: NonZeroI32) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_with_error_message_from_env(error_code)
    }
}
