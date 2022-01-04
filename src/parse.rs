//! Trait für Typen, die aus Kommandozeilen-Argumenten geparsed werden können.

use std::ffi::OsString;

use nonempty::NonEmpty;

use crate::{
    arg::Arg,
    ergebnis::{ParseErgebnis, ParseFehler},
};

pub use kommandozeilen_argumente_derive::Parse;

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
