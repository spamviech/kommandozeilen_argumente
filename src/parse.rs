//! Trait für Typen, die aus Kommandozeilen-Argumenten geparst werden können.

use std::ffi::OsString;
#[cfg(feature = "derive")]
use std::fmt::Display;

use nonempty::NonEmpty;

#[cfg(feature = "derive")]
use crate::{arg::wert::ArgEnum, beschreibung::Beschreibung};
use crate::{
    arg::Arg,
    ergebnis::{ParseErgebnis, ParseFehler},
};

#[cfg(feature = "derive")]
pub use kommandozeilen_argumente_derive::Parse;

// TODO besseren Namen finden
#[cfg(feature = "derive")]
/// Trait für Typen, die direkt mit dem derive-Macro für [Parse] verwendet werden können.
pub trait ArgumentArt: Sized {
    /// Erstelle ein [Arg] mit den konfigurierten Eigenschaften.
    ///
    /// `invertiere_prefix` ist für Flag-Argumente gedacht,
    /// `meta_var` für Wert-Argumente.
    fn erstelle_arg(
        beschreibung: Beschreibung<Self>,
        invertiere_prefix: &'static str,
        meta_var: &str,
    ) -> Arg<Self, OsString>;

    /// Sollen Argumente dieses Typs normalerweise einen Standard-Wert haben?
    fn standard() -> Option<Self>;
}

#[cfg(feature = "derive")]
impl ArgumentArt for bool {
    fn erstelle_arg(
        beschreibung: Beschreibung<Self>,
        invertiere_prefix: &'static str,
        _meta_var: &str,
    ) -> Arg<Self, OsString> {
        Arg::flag(beschreibung, invertiere_prefix)
    }

    fn standard() -> Option<Self> {
        Some(false)
    }
}

#[cfg(feature = "derive")]
impl ArgumentArt for String {
    fn erstelle_arg(
        beschreibung: Beschreibung<Self>,
        _invertiere_prefix: &'static str,
        meta_var: &str,
    ) -> Arg<Self, OsString> {
        Arg::wert(beschreibung, meta_var.to_owned(), None, |os_str| {
            if let Some(string) = os_str.to_str() {
                Ok(string.to_owned())
            } else {
                Err(os_str.to_owned())
            }
        })
    }

    fn standard() -> Option<Self> {
        None
    }
}

macro_rules! impl_parse_types {
    ($($type:ty),*$(,)?) => {$(
        #[cfg(feature = "derive")]
        impl ArgumentArt for $type {
            fn erstelle_arg(
                beschreibung: Beschreibung<Self>,
                _invertiere_prefix: &'static str,
                meta_var: &str,
            ) -> Arg<Self, OsString> {
                Arg::wert(beschreibung, meta_var.to_owned(), None, |os_str| {
                    if let Some(u) = os_str.to_str().and_then(|s| s.parse().ok()) {
                        Ok(u)
                    } else {
                        Err(os_str.to_owned())
                    }
                })
            }

            fn standard() -> Option<Self> {
                None
            }
        }
    )*};
}
impl_parse_types! {i8,u8,i16,u16,i32,u32,i64,u64,i128,u128,isize,usize,f32,f64}

#[cfg(feature = "derive")]
impl<T: 'static + ArgumentArt + Clone + Display> ArgumentArt for Option<T> {
    fn erstelle_arg(
        beschreibung: Beschreibung<Self>,
        invertiere_prefix: &'static str,
        meta_var: &str,
    ) -> Arg<Self, OsString> {
        let Beschreibung { lang, kurz, .. } = &beschreibung;
        let Arg { parse, .. } = T::erstelle_arg(
            Beschreibung { lang: lang.clone(), kurz: kurz.clone(), hilfe: None, standard: None },
            invertiere_prefix,
            meta_var,
        );
        let name: OsString = format!("--{}", lang).into();
        Arg::wert_allgemein(
            beschreibung,
            meta_var.to_owned(),
            None,
            move |arg| {
                let (ergebnis, _nicht_verwendet) = parse(vec![Some(name.as_os_str()), Some(arg)]);
                match ergebnis {
                    ParseErgebnis::Wert(wert) => Ok(Some(wert)),
                    _ergebnis => Err(arg.to_owned()),
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

#[cfg(feature = "derive")]
impl<T: 'static + ArgEnum + Display + Clone> ArgumentArt for T {
    fn erstelle_arg(
        beschreibung: Beschreibung<Self>,
        _invertiere_prefix: &'static str,
        meta_var: &str,
    ) -> Arg<Self, OsString> {
        Arg::wert_enum(beschreibung, meta_var.to_owned())
    }

    fn standard() -> Option<Self> {
        None
    }
}

/// Erlaube parsen aus Kommandozeilen-Argumenten ausgehend einer Standard-Konfiguration.
///
/// Mit aktiviertem `derive`-Feature kann diese automatisch erzeugt werden.
pub trait Parse: Sized {
    /// Möglicher Parse-Fehler, die automatisch erzeugte Implementierung verwendet [OsString].
    type Fehler;

    /// Erzeuge eine Beschreibung, wie Kommandozeilen-Argumente geparst werden sollen.
    fn kommandozeilen_argumente() -> Arg<Self, Self::Fehler>;

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    fn parse(
        args: impl Iterator<Item = OsString>,
    ) -> (ParseErgebnis<Self, Self::Fehler>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse(args)
    }

    /// Parse [std::env::args_os] und versuche den gewünschten Typ zu erzeugen.
    fn parse_aus_env() -> (ParseErgebnis<Self, Self::Fehler>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_aus_env()
    }

    /// Parse [std::env::args_os] und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [std::process::exit] mit exit code `0` beendet.
    fn parse_aus_env_mit_frühen_beenden(
    ) -> (Result<Self, NonEmpty<ParseFehler<Self::Fehler>>>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_aus_env_mit_frühen_beenden()
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [std::process::exit] mit exit code `0` beendet.
    fn parse_mit_frühen_beenden(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<Self, NonEmpty<ParseFehler<Self::Fehler>>>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_mit_frühen_beenden(args)
    }
}
