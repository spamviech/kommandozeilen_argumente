//! Trait für Typen, die aus Kommandozeilen-Argumenten geparst werden können.

#[cfg(feature = "derive")]
use std::{ffi::OsString, fmt::Display};

#[cfg(feature = "derive")]
use nonempty::NonEmpty;

#[cfg(feature = "derive")]
use crate::{
    arg::{wert::ArgEnum, Arg},
    beschreibung::Beschreibung,
    ergebnis::{ParseErgebnis, ParseFehler},
};

#[cfg(feature = "derive")]
pub use kommandozeilen_argumente_derive::Parse;

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
