//! Trait für Typen, die aus Kommandozeilen-Argumenten geparst werden können.

use std::{ffi::OsString, fmt::Display, num::NonZeroI32, str::FromStr};

use nonempty::NonEmpty;

use crate::{
    argumente::{wert::ArgEnum, Argumente},
    beschreibung::Beschreibung,
    ergebnis::{Ergebnis, Fehler, ParseFehler},
};

#[cfg(feature = "derive")]
pub use kommandozeilen_argumente_derive::Parse;

/// Trait für Typen, die direkt mit dem derive-Macro für [Parse] verwendet werden können.
pub trait ParseArgument: Sized {
    /// Erstelle ein [Argumente] mit den konfigurierten Eigenschaften.
    ///
    /// `invertiere_präfix` ist für Flag-Argumente gedacht,
    /// `meta_var` für Wert-Argumente.
    fn erstelle_arg(
        beschreibung: Beschreibung<Self>,
        invertiere_präfix: &'static str,
        meta_var: &str,
    ) -> Argumente<Self, String>;

    /// Sollen Argumente dieses Typs normalerweise einen Standard-Wert haben?
    fn standard() -> Option<Self>;

    /// Erstelle ein [Argumente] für die übergebene [Beschreibung].
    fn neu(beschreibung: Beschreibung<Self>) -> Argumente<Self, String> {
        Self::erstelle_arg(beschreibung, "kein", "WERT")
    }

    /// Create an [Argumente] for the [Beschreibung].
    fn new(beschreibung: Beschreibung<Self>) -> Argumente<Self, String> {
        Self::erstelle_arg(beschreibung, "no", "VALUE")
    }
}

impl ParseArgument for bool {
    fn erstelle_arg(
        beschreibung: Beschreibung<Self>,
        invertiere_präfix: &'static str,
        _meta_var: &str,
    ) -> Argumente<Self, String> {
        Argumente::flag(beschreibung, invertiere_präfix)
    }

    fn standard() -> Option<Self> {
        Some(false)
    }
}

impl ParseArgument for String {
    fn erstelle_arg(
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
            fn erstelle_arg(
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
    fn erstelle_arg(
        beschreibung: Beschreibung<Self>,
        invertiere_präfix: &'static str,
        meta_var: &str,
    ) -> Argumente<Self, String> {
        let Beschreibung { lang, kurz, .. } = &beschreibung;
        let Argumente { parse, .. } = T::erstelle_arg(
            Beschreibung { lang: lang.clone(), kurz: kurz.clone(), hilfe: None, standard: None },
            invertiere_präfix,
            meta_var,
        );
        let name: OsString = format!("--{}", lang).into();
        Argumente::wert(
            beschreibung,
            meta_var.to_owned(),
            None,
            move |arg| {
                let (ergebnis, _nicht_verwendet) = parse(vec![Some(name.as_os_str()), Some(arg)]);
                match ergebnis {
                    Ergebnis::Wert(wert) => Ok(Some(wert)),
                    _ergebnis => Err(ParseFehler::InvaliderString(arg.to_owned())),
                }
            },
            |opt| {
                if let Some(t) = opt {
                    t.to_string()
                } else {
                    "None".to_owned()
                }
            },
        )
    }

    fn standard() -> Option<Self> {
        Some(None)
    }
}

impl<T: 'static + ArgEnum + Display + Clone> ParseArgument for T {
    fn erstelle_arg(
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
/// Mit aktiviertem `derive`-Feature kann diese automatisch erzeugt werden.
pub trait Parse: Sized {
    /// Möglicher Parse-Fehler, die automatisch erzeugte Implementierung verwendet [String].
    type Fehler;

    /// Erzeuge eine Beschreibung, wie Kommandozeilen-Argumente geparst werden sollen.
    fn kommandozeilen_argumente() -> Argumente<Self, Self::Fehler>;

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    fn parse(
        args: impl Iterator<Item = OsString>,
    ) -> (Ergebnis<Self, Self::Fehler>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse(args)
    }

    /// Parse [std::env::args_os] und versuche den gewünschten Typ zu erzeugen.
    fn parse_aus_env() -> (Ergebnis<Self, Self::Fehler>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_aus_env()
    }

    /// Parse [std::env::args_os] und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [std::process::exit] mit exit code `0` beendet.
    fn parse_aus_env_mit_frühen_beenden(
    ) -> (Result<Self, NonEmpty<Fehler<Self::Fehler>>>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_aus_env_mit_frühen_beenden()
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [std::process::exit] mit exit code `0` beendet.
    fn parse_mit_frühen_beenden(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<Self, NonEmpty<Fehler<Self::Fehler>>>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_mit_frühen_beenden(args)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [std::process::exit] mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [std::process::exit] mit exit code `fehler_code` beendet.
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

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [std::process::exit] with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [std::process::exit] with exit code `error_code`.
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
    /// [std::process::exit] mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [std::process::exit] mit exit code `fehler_code` beendet.
    fn parse_mit_fehlermeldung(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_mit_fehlermeldung(args, fehler_code)
    }

    /// Parse [std::env::args_os] und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [std::process::exit] mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [std::process::exit] mit exit code `fehler_code` beendet.
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

    /// Parse [std::env::args_os] und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [std::process::exit] mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [std::process::exit] mit exit code `fehler_code` beendet.
    fn parse_mit_fehlermeldung_aus_env(fehler_code: NonZeroI32) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_mit_fehlermeldung_aus_env(fehler_code)
    }

    /// Parse [std::env::args_os] to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [std::process::exit] with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [std::process::exit] with exit code `error_code`.
    fn parse_with_error_message_from_env(error_code: NonZeroI32) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_with_error_message_from_env(error_code)
    }
}
