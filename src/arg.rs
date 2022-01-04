//! Definition von akzeptierten Kommandozeilen-Argumenten.

use std::{
    env,
    ffi::{OsStr, OsString},
    fmt::Debug,
    process,
};

use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    beschreibung::Beschreibung,
    ergebnis::{ParseErgebnis, ParseFehler},
};

pub mod flag;
#[path = "arg/frühes_beenden.rs"]
pub mod frühes_beenden;
pub mod kombiniere;
pub mod wert;

#[derive(Debug)]
pub enum ArgString {
    Flag {
        beschreibung: Beschreibung<String>,
        invertiere_prefix: Option<String>,
    },
    Wert {
        beschreibung: Beschreibung<String>,
        meta_var: String,
        mögliche_werte: Option<NonEmpty<String>>,
    },
}

// TODO derive-Macro zum automatischen erstellen aus Struktur-Definition?
// Parse-Trait, dass alle Methoden bis auf erstellen von Arg<Self, E> bereits implementiert
// derive-Macro muss sich nur noch darum kümmern
// TODO Unterbefehle/subcommands
// TODO Positions-basierte Argumente

/// Kommandozeilen-Argumente und ihre Beschreibung.
///
/// Felder sind public, damit das [kombiniere]-Macro funktioniert, ein Verwenden ist nicht vorgesehen.
/// Stattdessen werden die jeweiligen Methoden [flag], [wert], [frühes_beenden], [parse], etc. empfohlen.
pub struct Arg<T, E> {
    pub beschreibungen: Vec<ArgString>,
    pub flag_kurzformen: Vec<String>,
    pub parse: Box<dyn Fn(Vec<Option<&OsStr>>) -> (ParseErgebnis<T, E>, Vec<Option<&OsStr>>)>,
}

impl<T, E> Debug for Arg<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arg")
            .field("beschreibungen", &self.beschreibungen)
            .field("parse", &"<function>")
            .finish()
    }
}

impl<T, E> Arg<T, E> {
    #[inline(always)]
    pub fn parse_aus_env(&self) -> (ParseErgebnis<T, E>, Vec<OsString>) {
        Arg::parse(&self, env::args_os())
    }

    #[inline(always)]
    pub fn parse_aus_env_mit_frühen_beenden(
        &self,
    ) -> (Result<T, NonEmpty<ParseFehler<E>>>, Vec<OsString>) {
        self.parse_mit_frühen_beenden(env::args_os())
    }

    pub fn parse_mit_frühen_beenden(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> (Result<T, NonEmpty<ParseFehler<E>>>, Vec<OsString>) {
        let (ergebnis, nicht_verwendet) = self.parse(args);
        let result = match ergebnis {
            ParseErgebnis::Wert(wert) => Ok(wert),
            ParseErgebnis::FrühesBeenden(nachrichten) => {
                for nachricht in nachrichten {
                    println!("{}", nachricht);
                }
                process::exit(0)
            }
            ParseErgebnis::Fehler(fehler) => Err(fehler),
        };
        (result, nicht_verwendet)
    }

    pub fn parse(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> (ParseErgebnis<T, E>, Vec<OsString>) {
        let Arg { beschreibungen: _, flag_kurzformen, parse } = self;
        let angepasste_args: Vec<OsString> = args
            .flat_map(|arg| {
                if let Some(string) = arg.to_str() {
                    if let Some(kurz) = string.strip_prefix('-') {
                        let mut gefundene_kurzformen = Vec::new();
                        for grapheme in kurz.graphemes(true) {
                            if flag_kurzformen.iter().any(|string| string == grapheme) {
                                gefundene_kurzformen.push(grapheme.to_owned().into())
                            } else {
                                return vec![arg];
                            }
                        }
                        if !gefundene_kurzformen.is_empty() {
                            return gefundene_kurzformen;
                        }
                    }
                }
                vec![arg]
            })
            .collect();
        let args_os_str: Vec<_> =
            angepasste_args.iter().map(OsString::as_os_str).map(Some).collect();
        let (ergebnis, nicht_verwendet) = parse(args_os_str);
        (ergebnis, nicht_verwendet.into_iter().filter_map(|opt| opt.map(OsStr::to_owned)).collect())
    }
}
