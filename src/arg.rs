//! Definition von akzeptierten Kommandozeilen-Argumenten.

use std::{
    env,
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    num::NonZeroI32,
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

/// Interner Typ, wird für das [kombiniere]-Macro benötigt.
#[derive(Debug)]
pub(crate) enum ArgString {
    Flag {
        beschreibung: Beschreibung<String>,
        invertiere_präfix: Option<String>,
    },
    Wert {
        beschreibung: Beschreibung<String>,
        meta_var: String,
        mögliche_werte: Option<NonEmpty<String>>,
    },
}

// TODO Unterbefehle/subcommands
// TODO Positions-basierte Argumente
// TODO Standard-Wert, sofern nur der Name gegeben ist (unterschiedlich zu Name kommt nicht vor)
//      z.B. -O, -O2 bei compilern
// TODO case sensitive Argumente/alles case sensitive

/// Kommandozeilen-Argumente und ihre Beschreibung.
pub struct Arg<T, E> {
    pub(crate) beschreibungen: Vec<ArgString>,
    pub(crate) flag_kurzformen: Vec<String>,
    pub(crate) parse:
        Box<dyn Fn(Vec<Option<&OsStr>>) -> (ParseErgebnis<T, E>, Vec<Option<&OsStr>>)>,
}

impl<T, E> Debug for Arg<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Arg")
            .field("beschreibungen", &self.beschreibungen)
            .field("parse", &"<function>")
            .finish()
    }
}

impl<T, E: Display> Arg<T, E> {
    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [std::process::exit] mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [std::process::exit] mit exit code `fehler_code` beendet.
    #[inline(always)]
    pub fn parse_mit_fehlermeldung(
        &self,
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
    ) -> T {
        self.parse_vollständig(
            args,
            fehler_code,
            "Fehlende Flag",
            "Fehlender Wert",
            "Parse-Fehler",
            "Nicht alle Argumente verwendet",
        )
    }

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [std::process::exit] with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [std::process::exit] with exit code `error_code`.
    #[inline(always)]
    pub fn parse_with_error_message(
        &self,
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
    ) -> T {
        self.parse_vollständig(
            args,
            error_code,
            "Missing Flag",
            "Missing Value",
            "Parse Error",
            "Unused arguments",
        )
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [std::process::exit] mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [std::process::exit] mit exit code `fehler_code` beendet.
    pub fn parse_vollständig(
        &self,
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        arg_nicht_verwendet: &str,
    ) -> T {
        let (ergebnis, nicht_verwendet) = self.parse(args);
        match ergebnis {
            ParseErgebnis::Wert(wert) if nicht_verwendet.is_empty() => wert,
            ParseErgebnis::Wert(_wert) => {
                eprintln!("{}: {:?}", arg_nicht_verwendet, nicht_verwendet);
                process::exit(fehler_code.get())
            }
            ParseErgebnis::FrühesBeenden(nachrichten) => {
                for nachricht in nachrichten {
                    println!("{}", nachricht);
                }
                process::exit(0)
            }
            ParseErgebnis::Fehler(fehler_sammlung) => {
                for fehler in fehler_sammlung {
                    eprintln!(
                        "{}",
                        fehler.erstelle_fehlermeldung(fehlende_flag, fehlender_wert, parse_fehler)
                    )
                }
                process::exit(fehler_code.get())
            }
        }
    }
}

impl<T, E> Arg<T, E> {
    /// Parse [std::env::args_os] und versuche den gewünschten Typ zu erzeugen.
    #[inline(always)]
    pub fn parse_aus_env(&self) -> (ParseErgebnis<T, E>, Vec<OsString>) {
        Arg::parse(&self, env::args_os().skip(1))
    }

    /// Parse [std::env::args_os] und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [std::process::exit] mit exit code `0` beendet.
    #[inline(always)]
    pub fn parse_aus_env_mit_frühen_beenden(
        &self,
    ) -> (Result<T, NonEmpty<ParseFehler<E>>>, Vec<OsString>) {
        self.parse_mit_frühen_beenden(env::args_os().skip(1))
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [std::process::exit] mit exit code `0` beendet.
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

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
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
                                gefundene_kurzformen.push(format!("-{}", grapheme).into())
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
