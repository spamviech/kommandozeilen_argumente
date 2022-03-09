//! Trait für Typen, die aus Kommandozeilen-Argumenten geparst werden können.

use std::{ffi::OsString, fmt::Display, num::NonZeroI32, str::FromStr};

use nonempty::NonEmpty;

use crate::{
    argumente::{wert::EnumArgument, ArgString, Argumente},
    beschreibung::Beschreibung,
    ergebnis::{Ergebnis, Fehler, ParseFehler},
    sprache::Sprache,
};

#[cfg(any(feature = "derive", doc))]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use kommandozeilen_argumente_derive::Parse;

/// Trait für Typen, die direkt mit dem derive-Macro für [Parse] verwendet werden können.
pub trait ParseArgument: Sized {
    /// Erstelle ein [Argumente] mit den konfigurierten Eigenschaften.
    ///
    /// `invertiere_präfix` ist für Flag-Argumente gedacht,
    /// `meta_var` für Wert-Argumente.
    fn argumente(
        beschreibung: Beschreibung<Self>,
        invertiere_präfix: &'static str,
        meta_var: &str,
    ) -> Argumente<Self, String>;

    /// Sollen Argumente dieses Typs normalerweise einen Standard-Wert haben?
    fn standard() -> Option<Self>;

    /// Erstelle ein [Argumente] für die übergebene [Beschreibung].
    fn argumente_mit_sprache(
        beschreibung: Beschreibung<Self>,
        sprache: Sprache,
    ) -> Argumente<Self, String> {
        Self::argumente(beschreibung, sprache.invertiere_präfix, sprache.meta_var)
    }

    /// Erstelle ein [Argumente] für die übergebene [Beschreibung].
    fn neu(beschreibung: Beschreibung<Self>) -> Argumente<Self, String> {
        Self::argumente_mit_sprache(beschreibung, Sprache::DEUTSCH)
    }

    /// Create an [Argumente] for the [Beschreibung].
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
            beschreibungen: vec![ArgString::Wert {
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
pub trait Parse: Sized {
    /// Möglicher Parse-Fehler, die automatisch erzeugte Implementierung verwendet [String].
    type Fehler;

    /// Erzeuge eine Beschreibung, wie Kommandozeilen-Argumente geparst werden sollen.
    fn kommandozeilen_argumente() -> Argumente<Self, Self::Fehler>;

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    #[inline(always)]
    fn parse(
        args: impl Iterator<Item = OsString>,
    ) -> (Ergebnis<Self, Self::Fehler>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse(args)
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    #[inline(always)]
    fn parse_aus_env() -> (Ergebnis<Self, Self::Fehler>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_aus_env()
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    #[inline(always)]
    fn parse_aus_env_mit_frühen_beenden(
    ) -> (Result<Self, NonEmpty<Fehler<Self::Fehler>>>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_aus_env_mit_frühen_beenden()
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    #[inline(always)]
    fn parse_mit_frühen_beenden(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<Self, NonEmpty<Fehler<Self::Fehler>>>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_mit_frühen_beenden(args)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
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

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    #[inline(always)]
    fn parse_vollständig_mit_sprache(
        &self,
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        sprache: Sprache,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_vollständig_mit_sprache(args, fehler_code, sprache)
    }

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
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

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
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

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
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

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    #[inline(always)]
    fn parse_vollständig_mit_sprache_aus_env(
        &self,
        fehler_code: NonZeroI32,
        sprache: Sprache,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente()
            .parse_vollständig_mit_sprache_aus_env(fehler_code, sprache)
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
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
    #[inline(always)]
    fn parse_with_error_message_from_env(error_code: NonZeroI32) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_with_error_message_from_env(error_code)
    }
}
