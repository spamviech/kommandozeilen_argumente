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
    beschreibung::{Configuration, Konfiguration},
    ergebnis::{Ergebnis, Error, Fehler, Result},
    sprache::{Language, Sprache},
};

pub(crate) mod flag;
#[path = "argumente/frühes_beenden.rs"]
pub(crate) mod frühes_beenden;
pub(crate) mod kombiniere;
pub(crate) mod wert;

#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use self::wert::EnumArgument;

#[doc(inline)]
pub use crate::{combine, kombiniere};

// TODO Unterbefehle/subcommands
// TODO Positions-basierte Argumente
// TODO Standard-Wert, sofern nur der Name gegeben ist (unterschiedlich zu Name kommt nicht vor)
//      z.B. nichts: -O0, -O: -O1, -O=N für explizite Angabe
//      vgl. mit Flag-Argumenten, kann zu parse-Problemen wegen Mehrdeutigkeit führen
// TODO case sensitive Argumente/alles case sensitive
//  benötigt Unicode normalization (wird bisher von unicase übernommen?), auch optional?
// TODO Verwende unicode normalization, bevor das erste Grapheme für Kurznamen extrahiert wird?
//      sowohl für automatisch erzeugte, wie für überprüfte Kurznamen
//      https://crates.io/crates/unicode-normalization
// TODO Argument-Gruppen (nur eine dieser N Flags kann gleichzeitig aktiv sein)
// TODO Programm-Beschreibung in Hilfe-Text
// TODO Feature-gates für automatische Hilfe, verschmelzen von flag-kurzformen, ...
//      benötigen extra Felder in Argumente-Struktur, könnte Performance verbessern
// TODO Cow<'t/'static, str> anstelle von String verwenden
// TODO OneOf/Either für alternative Parse-Möglichkeiten

/// Kommandozeilen-Argumente und ihre Beschreibung.
pub struct Argumente<T, E> {
    pub(crate) konfigurationen: Vec<Konfiguration>,
    pub(crate) flag_kurzformen: Vec<String>,
    pub(crate) parse: Box<dyn Fn(Vec<Option<&OsStr>>) -> (Ergebnis<T, E>, Vec<Option<&OsStr>>)>,
}

/// Command line [Arguments] and their [crate::beschreibung::Description].
pub type Arguments<T, E> = Argumente<T, E>;

impl<T, E> Debug for Argumente<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Argumente")
            .field("konfigurationen", &self.konfigurationen)
            .field("parse", &"<function>")
            .finish()
    }
}

#[inline(always)]
fn args_aus_env() -> impl Iterator<Item = OsString> {
    env::args_os().skip(1)
}

impl<T, E: Display> Argumente<T, E> {
    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    #[inline(always)]
    pub fn parse_mit_fehlermeldung_aus_env(&self, fehler_code: NonZeroI32) -> T {
        self.parse_mit_fehlermeldung(args_aus_env(), fehler_code)
    }

    /// Parse [args_os](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    #[inline(always)]
    pub fn parse_with_error_message_from_env(&self, error_code: NonZeroI32) -> T {
        self.parse_with_error_message(args_aus_env(), error_code)
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    #[inline(always)]
    pub fn parse_vollständig_mit_sprache_aus_env(
        &self,
        fehler_code: NonZeroI32,
        sprache: Sprache,
    ) -> T {
        self.parse_vollständig_aus_env(
            fehler_code,
            sprache.fehlende_flag,
            sprache.fehlender_wert,
            sprache.parse_fehler,
            sprache.invalider_string,
            sprache.argument_nicht_verwendet,
        )
    }

    /// Parse [args_os](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    #[inline(always)]
    pub fn parse_complete_with_language_from_env(
        &self,
        error_code: NonZeroI32,
        language: Language,
    ) -> T {
        self.parse_vollständig_mit_sprache_aus_env(error_code, language)
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    #[inline(always)]
    pub fn parse_vollständig_aus_env(
        &self,
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        arg_nicht_verwendet: &str,
    ) -> T {
        self.parse_vollständig(
            args_aus_env(),
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
    #[inline(always)]
    pub fn parse_complete_from_env(
        &self,
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> T {
        self.parse_vollständig_aus_env(
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
    #[inline(always)]
    pub fn parse_mit_fehlermeldung(
        &self,
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
    ) -> T {
        self.parse_vollständig_mit_sprache(args, fehler_code, Sprache::DEUTSCH)
    }

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    #[inline(always)]
    pub fn parse_with_error_message(
        &self,
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
    ) -> T {
        self.parse_vollständig_mit_sprache(args, error_code, Sprache::ENGLISH)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    #[inline(always)]
    pub fn parse_vollständig_mit_sprache(
        &self,
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        sprache: Sprache,
    ) -> T {
        self.parse_vollständig(
            args,
            fehler_code,
            sprache.fehlende_flag,
            sprache.fehlender_wert,
            sprache.parse_fehler,
            sprache.invalider_string,
            sprache.argument_nicht_verwendet,
        )
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    #[inline(always)]
    pub fn parse_complete_with_language(
        &self,
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        language: Language,
    ) -> T {
        self.parse_vollständig_mit_sprache(args, error_code, language)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    pub fn parse_vollständig(
        &self,
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        arg_nicht_verwendet: &str,
    ) -> T {
        let (ergebnis, nicht_verwendet) = self.parse(args);
        match ergebnis {
            Ergebnis::Wert(wert) if nicht_verwendet.is_empty() => wert,
            Ergebnis::Wert(_wert) => {
                eprintln!("{}: {:?}", arg_nicht_verwendet, nicht_verwendet);
                process::exit(fehler_code.get())
            },
            Ergebnis::FrühesBeenden(nachrichten) => {
                for nachricht in nachrichten {
                    println!("{}", nachricht);
                }
                process::exit(0)
            },
            Ergebnis::Fehler(fehler_sammlung) => {
                for fehler in fehler_sammlung {
                    eprintln!(
                        "{}",
                        fehler.erstelle_fehlermeldung(
                            fehlende_flag,
                            fehlender_wert,
                            parse_fehler,
                            invalider_string
                        )
                    )
                }
                process::exit(fehler_code.get())
            },
        }
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    #[inline(always)]
    pub fn parse_complete(
        &self,
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> T {
        self.parse_vollständig(
            args,
            error_code,
            missing_flag,
            missing_value,
            parse_error,
            invalid_string,
            unused_arg,
        )
    }
}

impl<T, E> Argumente<T, E> {
    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    #[inline(always)]
    pub fn parse_aus_env(&self) -> (Ergebnis<T, E>, Vec<OsString>) {
        Argumente::parse(&self, args_aus_env())
    }

    /// Parse the given command line arguments to create the requested type.
    #[inline(always)]
    pub fn parse_from_env(&self) -> (Result<T, E>, Vec<OsString>) {
        self.parse_aus_env()
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    #[inline(always)]
    pub fn parse_aus_env_mit_frühen_beenden(
        &self,
    ) -> (std::result::Result<T, NonEmpty<Fehler<E>>>, Vec<OsString>) {
        self.parse_mit_frühen_beenden(args_aus_env())
    }

    /// Parse [args_os](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    #[inline(always)]
    pub fn parse_from_env_with_early_exit(
        &self,
    ) -> (std::result::Result<T, NonEmpty<Error<E>>>, Vec<OsString>) {
        self.parse_aus_env_mit_frühen_beenden()
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    pub fn parse_mit_frühen_beenden(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> (std::result::Result<T, NonEmpty<Fehler<E>>>, Vec<OsString>) {
        let (ergebnis, nicht_verwendet) = self.parse(args);
        let result = match ergebnis {
            Ergebnis::Wert(wert) => Ok(wert),
            Ergebnis::FrühesBeenden(nachrichten) => {
                for nachricht in nachrichten {
                    println!("{}", nachricht);
                }
                process::exit(0)
            },
            Ergebnis::Fehler(fehler) => Err(fehler),
        };
        (result, nicht_verwendet)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    #[inline(always)]
    pub fn parse_with_early_exit(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> (std::result::Result<T, NonEmpty<Error<E>>>, Vec<OsString>) {
        self.parse_mit_frühen_beenden(args)
    }

    // TODO english doc
    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    pub fn parse(&self, args: impl Iterator<Item = OsString>) -> (Ergebnis<T, E>, Vec<OsString>) {
        let Argumente { konfigurationen: _, flag_kurzformen, parse } = self;
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

    /// Alle konfigurierten Kommandozeilen-Argumente.
    #[inline(always)]
    pub fn konfigurationen(&self) -> impl Iterator<Item = &Konfiguration> {
        self.konfigurationen.iter()
    }

    /// All configured command line arguments.
    #[inline(always)]
    pub fn configurations(&self) -> impl Iterator<Item = &Configuration> {
        self.konfigurationen.iter()
    }
}
