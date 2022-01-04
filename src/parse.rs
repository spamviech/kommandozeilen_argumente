//! Trait für Typen, die aus Kommandozeilen-Argumenten geparsed werden können.

use std::{ffi::OsString, fmt::Display};

use nonempty::NonEmpty;

use crate::{
    arg::{wert::ArgEnum, Arg},
    beschreibung::Beschreibung,
    ergebnis::{ParseErgebnis, ParseFehler},
};

pub use kommandozeilen_argumente_derive::Parse;

pub trait ArgumentArt: Sized {
    fn erstelle_arg(
        beschreibung: Beschreibung<Self>,
        invertiere_prefix: &'static str,
        meta_var: &str,
    ) -> Arg<Self, OsString>;

    fn standard() -> Option<Self>;
}

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

pub trait Parse: Sized {
    type Fehler;

    fn kommandozeilen_argumente() -> Arg<Self, Self::Fehler>;

    fn parse(
        args: impl Iterator<Item = OsString>,
    ) -> (ParseErgebnis<Self, Self::Fehler>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse(args)
    }

    fn parse_aus_env() -> (ParseErgebnis<Self, Self::Fehler>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_aus_env()
    }

    fn parse_aus_env_mit_frühen_beenden(
    ) -> (Result<Self, NonEmpty<ParseFehler<Self::Fehler>>>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_aus_env_mit_frühen_beenden()
    }

    fn parse_mit_frühen_beenden(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<Self, NonEmpty<ParseFehler<Self::Fehler>>>, Vec<OsString>) {
        Self::kommandozeilen_argumente().parse_mit_frühen_beenden(args)
    }
}
