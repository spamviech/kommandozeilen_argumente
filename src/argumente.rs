//! Definition von akzeptierten Kommandozeilen-Argumenten.

use std::{
    collections::HashMap,
    convert::identity,
    env,
    ffi::OsString,
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
    unicode::{Normalisiert, Vergleich},
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
// TODO Argument-Gruppen (nur eine dieser N Flags kann gleichzeitig aktiv sein)
// TODO Feature-gates für automatische Hilfe, verschmelzen von flag-kurzformen, ...
//      benötigen extra Felder in Argumente-Struktur, könnte Performance verbessern
// TODO tests mit Unicode-namen
// TODO OneOf/Either für alternative Parse-Möglichkeiten
//      alternativ-Methode (analog kombinierte2), besondere Methode für Either-Typen?
// TODO Standard-Wert, sofern nur der Name gegeben ist (unterschiedlich zu Name kommt nicht vor)
//      z.B. nichts: -O0, -O: -O1, -O=N für explizite Angabe
//      vgl. mit Flag-Argumenten, kann zu parse-Problemen wegen Mehrdeutigkeit führen
//      kann durch alternativ-Methode erzeugt werden (erst Wert, dann Flag)
//          dazu spezialisierte Methode bereitstellen

/// Kommandozeilen-Argumente und ihre Beschreibung.
pub struct Argumente<'t, T, E> {
    pub(crate) konfigurationen: Vec<Konfiguration<'t>>,
    pub(crate) flag_kurzformen: HashMap<Vergleich<'t>, Vec<Vergleich<'t>>>,
    pub(crate) parse:
        Box<dyn 't + Fn(Vec<Option<OsString>>) -> (Ergebnis<'t, T, E>, Vec<Option<OsString>>)>,
}

/// Command line [Arguments] and their [crate::beschreibung::Description].
pub type Arguments<'t, T, E> = Argumente<'t, T, E>;

impl<T, E> Debug for Argumente<'_, T, E> {
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

impl<T, E: Display> Argumente<'_, T, E> {
    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [exit](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [parse_with_error_message_from_env](Arguments::parse_with_error_message_from_env)
    #[inline(always)]
    pub fn parse_mit_fehlermeldung_aus_env(&self, fehler_code: NonZeroI32) -> T {
        self.parse_mit_fehlermeldung(args_aus_env(), fehler_code)
    }

    /// Parse [args_os](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [exit](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [parse_mit_fehlermeldung_aus_env](Argumente::parse_mit_fehlermeldung_aus_env)
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
    ///
    /// ## English synonym
    /// [parse_complete_with_language_from_env](Arguments::parse_complete_with_language_from_env)
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
    ///
    /// ## Deutsches Synonym
    /// [parse_vollständig_mit_sprache_aus_env](Argumente::parse_vollständig_mit_sprache_aus_env)
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
    ///
    /// ## English synonym
    /// [parse_complete_from_env](Arguments::parse_complete_from_env)
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
    ///
    /// ## Deutsches Synonym
    /// [parse_vollständig_aus_env](Argumente::parse_vollständig_aus_env)
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
    ///
    /// ## English synonym
    /// [parse_with_error_message](Arguments::parse_with_error_message)
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
    ///
    /// ## Deutsches Synonym
    /// [parse_mit_fehlermeldung](Argumente::parse_mit_fehlermeldung)
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
    ///
    /// ## English synonym
    /// [parse_complete_with_language](Arguments::parse_complete_with_language)
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
    ///
    /// ## Deutsches Synonym
    /// [parse_vollständig_mit_sprache](Argumente::parse_vollständig_mit_sprache)
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
    ///
    /// ## English synonym
    /// [parse_complete](Arguments::parse_complete)
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
    ///
    /// ## Deutsches Synonym
    /// [parse_vollständig](Argumente::parse_vollständig)
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

impl<'t, T, E> Argumente<'t, T, E> {
    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    ///
    /// ## English synonym
    /// [parse_from_env](Arguments::parse_from_env)
    #[inline(always)]
    pub fn parse_aus_env(&self) -> (Ergebnis<'t, T, E>, Vec<OsString>) {
        Argumente::parse(&self, args_aus_env())
    }

    /// Parse [args_os](std::env::args_os) to create the requested type.
    ///
    /// ## Deutsches Synonym
    /// [parse_aus_env](Argumente::parse_aus_env)
    #[inline(always)]
    pub fn parse_from_env(&self) -> (Result<'t, T, E>, Vec<OsString>) {
        self.parse_aus_env()
    }

    /// Parse [args_os](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [parse_from_env_with_early_exit](Arguments::parse_from_env_with_early_exit)
    #[inline(always)]
    pub fn parse_aus_env_mit_frühen_beenden(
        &self,
    ) -> (std::result::Result<T, NonEmpty<Fehler<'t, E>>>, Vec<OsString>) {
        self.parse_mit_frühen_beenden(args_aus_env())
    }

    /// Parse [args_os](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [exit](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [parse_aus_env_mit_frühen_beenden](Argumente::parse_aus_env_mit_frühen_beenden)
    #[inline(always)]
    pub fn parse_from_env_with_early_exit(
        &self,
    ) -> (std::result::Result<T, NonEmpty<Error<'t, E>>>, Vec<OsString>) {
        self.parse_aus_env_mit_frühen_beenden()
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [exit](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [parse_with_early_exit](Arguments::parse_with_early_exit)
    pub fn parse_mit_frühen_beenden(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> (std::result::Result<T, NonEmpty<Fehler<'t, E>>>, Vec<OsString>) {
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
    ///
    /// ## Deutsches Synonym
    /// [parse_mit_frühen_beenden](Argumente::parse_mit_frühen_beenden)
    #[inline(always)]
    pub fn parse_with_early_exit(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> (std::result::Result<T, NonEmpty<Error<'t, E>>>, Vec<OsString>) {
        self.parse_mit_frühen_beenden(args)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    ///
    /// ## English
    /// Parse the given command line arguments to create the requested type
    pub fn parse(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> (Ergebnis<'t, T, E>, Vec<OsString>) {
        let Argumente { konfigurationen: _, flag_kurzformen, parse } = self;
        let ersetze_verschmolzene_kurzformen = |arg: OsString| -> Vec<OsString> {
            if let Some(string) = arg.to_str() {
                for (prefix, kurzformen) in flag_kurzformen.iter() {
                    if let Some(kurz_str) = prefix.strip_als_präfix(&Normalisiert::neu(string)) {
                        let präfix_str = prefix.string.as_ref();
                        let mut gefundene_kurzformen = Vec::new();
                        for grapheme in kurz_str.graphemes(true) {
                            if kurzformen.iter().any(|vergleich| vergleich.eq(grapheme)) {
                                gefundene_kurzformen.push(format!("{präfix_str}{grapheme}").into())
                            } else {
                                return vec![arg];
                            }
                        }
                        if !gefundene_kurzformen.is_empty() {
                            return gefundene_kurzformen;
                        }
                    }
                }
            }
            vec![arg]
        };
        let angepasste_args: Vec<_> =
            args.flat_map(ersetze_verschmolzene_kurzformen).map(Some).collect();
        let (ergebnis, nicht_verwendet) = parse(angepasste_args);
        (ergebnis, nicht_verwendet.into_iter().filter_map(identity).collect())
    }

    /// Alle konfigurierten Kommandozeilen-Argumente.
    /// Hiermit ist es möglich einen eigenen,
    /// auf den konfigurierten Argumenten basierenden Hilfetext zu erzeugen.
    ///
    /// ## Deutsches Synonym
    /// [configurations](Argumente::configurations)
    #[inline(always)]
    pub fn konfigurationen(&self) -> impl Iterator<Item = &Konfiguration<'_>> {
        self.konfigurationen.iter()
    }

    /// All configured command line arguments.
    /// This function allows creating your own help text based on the configured arguments.
    ///
    /// ## English synonym
    /// [konfigurationen](Arguments::konfigurationen)
    #[inline(always)]
    pub fn configurations(&self) -> impl Iterator<Item = &Configuration<'_>> {
        self.konfigurationen.iter()
    }
}

// ----------------------------------TEST------------------------------------------------------

/// test
pub mod test {
    #![allow(missing_docs)]

    use std::{borrow::Cow, ffi::OsString, iter};

    use itertools::Itertools;
    use nonempty::NonEmpty;
    use unicode_segmentation::UnicodeSegmentation;
    use void::Void;

    use crate::{
        beschreibung::{contains_prefix, contains_str, Beschreibung, Name},
        ergebnis::{Ergebnis, Fehler, ParseFehler},
        unicode::{Normalisiert, Vergleich},
    };

    impl Name<'_> {
        fn parse_flag_aux<E>(
            &self,
            name_gefunden: impl FnOnce() -> E,
            parse_invertiert: impl FnOnce(&NonEmpty<Vergleich<'_>>, &Normalisiert<'_>) -> Option<E>,
            arg: OsString,
        ) -> Result<E, OsString> {
            let Name { lang_präfix, lang, kurz_präfix, kurz } = self;
            let name_kurz_existiert = !kurz.is_empty();
            if let Some(string) = arg.to_str() {
                let normalisiert = Normalisiert::neu(string);
                if let Some(lang_str) = &lang_präfix.strip_als_präfix_n(&normalisiert) {
                    if contains_str(lang, lang_str.as_str()) {
                        return Ok(name_gefunden());
                    } else if let Some(e) = parse_invertiert(lang, lang_str) {
                        return Ok(e);
                    }
                } else if name_kurz_existiert {
                    if let Some(kurz_graphemes) = kurz_präfix.strip_als_präfix_n(&normalisiert) {
                        if kurz_graphemes
                            .as_str()
                            .graphemes(true)
                            .exactly_one()
                            .map(|name| contains_str(kurz, name))
                            .unwrap_or(false)
                        {
                            return Ok(name_gefunden());
                        }
                    }
                }
            }
            Err(arg)
        }

        fn parse_flag(
            &self,
            invertiere_präfix: &Vergleich<'_>,
            invertiere_infix: &Vergleich<'_>,
            arg: OsString,
        ) -> Result<bool, OsString> {
            let parse_invertiert =
                |lang: &NonEmpty<Vergleich<'_>>, lang_str: &Normalisiert<'_>| -> Option<bool> {
                    if let Some(infix_name) = invertiere_präfix.strip_als_präfix_n(&lang_str) {
                        let infix_name_normalisiert = infix_name;
                        if let Some(negiert) =
                            invertiere_infix.strip_als_präfix_n(&infix_name_normalisiert)
                        {
                            if contains_str(lang, negiert.as_str()) {
                                return Some(false);
                            }
                        }
                    }
                    None
                };
            self.parse_flag_aux(|| true, parse_invertiert, arg)
        }

        #[inline(always)]
        fn parse_frühes_beenden(&self, arg: OsString) -> Result<(), OsString> {
            self.parse_flag_aux(|| (), |_, _| None, arg)
        }

        fn parse_mit_wert(
            &self,
            wert_infix: &Vergleich<'_>,
            arg: OsString,
        ) -> Result<Option<OsString>, OsString> {
            let Name { lang_präfix, lang, kurz_präfix, kurz } = self;
            let kurz_existiert = !kurz.is_empty();
            if let Some(string) = arg.to_str() {
                let normalisiert = Normalisiert::neu(string);
                if let Some(lang_str) = lang_präfix.strip_als_präfix_n(&normalisiert) {
                    let suffixe = contains_prefix(lang, &lang_str);
                    for suffix in suffixe {
                        let suffix_normalisiert = Normalisiert::neu_borrowed_unchecked(suffix);
                        if suffix.is_empty() {
                            return Ok(None);
                        } else if let Some(wert_graphemes) =
                            wert_infix.strip_als_präfix_n(&suffix_normalisiert)
                        {
                            return Ok(Some(wert_graphemes.as_str().to_owned().into()));
                        }
                    }
                } else if kurz_existiert {
                    if let Some(kurz_str) = kurz_präfix.strip_als_präfix_n(&normalisiert) {
                        let mut kurz_graphemes = kurz_str.as_str().graphemes(true);
                        if kurz_graphemes
                            .next()
                            .map(|name| contains_str(kurz, name))
                            .unwrap_or(false)
                        {
                            let rest =
                                Normalisiert::neu_borrowed_unchecked(kurz_graphemes.as_str());
                            let wert_str = if rest.as_str().is_empty() {
                                None
                            } else {
                                let wert_str = wert_infix
                                    .strip_als_präfix_n(&rest)
                                    .unwrap_or_else(|| rest.clone());
                                Some(wert_str.as_str().to_owned().into())
                            };
                            return Ok(wert_str);
                        }
                    }
                }
            }
            Err(arg)
        }
    }

    /// Es handelt sich um ein Flag-Argument.
    ///
    /// ## English
    /// It is a flag argument.
    #[derive(Debug)]
    pub struct Flag<'t, T, Bool, Anzeige>
    where
        Bool: Fn(bool) -> T,
        Anzeige: Fn(&T) -> String,
    {
        /// Allgemeine Beschreibung des Arguments.
        ///
        /// ## English
        /// General description of the argument.
        pub beschreibung: Beschreibung<'t, T>,

        /// Präfix invertieren des Flag-Arguments.
        ///
        /// ## English
        /// Prefix to invert the flag argument.
        pub invertiere_präfix: Vergleich<'t>,

        /// Infix zum invertieren des Flag-Arguments.
        ///
        /// ## English
        /// Infix to invert the flag argument.
        pub invertiere_infix: Vergleich<'t>,

        /// Erzeuge einen Wert aus einer [bool].
        ///
        /// ## English
        /// Create a value from a [bool].
        pub konvertiere: Bool,

        /// Anzeige eines Wertes (default value).
        ///
        /// ## English
        /// Display a value (default value).
        pub anzeige: Anzeige,
    }

    impl<T, Bool, Anzeige> Flag<'_, T, Bool, Anzeige>
    where
        T: Clone,
        Bool: Fn(bool) -> T,
        Anzeige: Fn(&T) -> String,
    {
        fn parse<F>(
            &self,
            args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'_, T, F>, Vec<Option<OsString>>) {
            let Flag {
                beschreibung, invertiere_präfix, invertiere_infix, konvertiere, anzeige: _
            } = self;
            let Beschreibung { name, hilfe: _, standard } = beschreibung;
            let mut nicht_verwendet = Vec::new();
            let mut iter = args.into_iter();
            while let Some(arg) = iter.next() {
                if let Some(arg) = arg {
                    match name.parse_flag(invertiere_präfix, invertiere_infix, arg) {
                        Ok(b) => {
                            nicht_verwendet.push(None);
                            nicht_verwendet.extend(iter);
                            return (Ergebnis::Wert(konvertiere(b)), nicht_verwendet);
                        },
                        Err(arg) => nicht_verwendet.push(Some(arg)),
                    }
                } else {
                    nicht_verwendet.push(None)
                }
            }
            let ergebnis = if let Some(wert) = standard {
                Ergebnis::Wert(wert.clone())
            } else {
                // FIXME Fehler-Typ anpassen
                let fehler = Fehler::FehlendeFlag {
                    namen: crate::ergebnis::Namen {
                        lang_präfix: name.lang_präfix.string.clone(),
                        lang: name.lang.clone().map(|Vergleich { string, case: _ }| string),
                        kurz_präfix: name.kurz_präfix.string.clone(),
                        kurz: name
                            .kurz
                            .iter()
                            .map(|Vergleich { string, case: _ }| string.clone())
                            .collect(),
                    },
                    invertiere_präfix: invertiere_präfix.string.clone(),
                    invertiere_infix: invertiere_infix.string.clone(),
                };
                Ergebnis::Fehler(NonEmpty::singleton(fehler))
            };
            (ergebnis, nicht_verwendet)
        }
    }

    impl<T, Bool, Anzeige> Flag<'_, T, Bool, Anzeige>
    where
        Bool: Fn(bool) -> T,
        Anzeige: Fn(&T) -> String,
    {
        pub fn erzeuge_hilfe_text(&self, meta_standard: &str) -> (String, Option<Cow<'_, str>>) {
            let Flag {
                beschreibung, invertiere_präfix, invertiere_infix, konvertiere: _, anzeige
            } = self;
            let Beschreibung { name, hilfe, standard } = beschreibung;
            let Name { lang_präfix, lang, kurz_präfix, kurz } = name;
            let mut hilfe_text = String::new();
            hilfe_text.push_str(lang_präfix.as_str());
            hilfe_text.push('[');
            hilfe_text.push_str(invertiere_präfix.as_str());
            hilfe_text.push_str(invertiere_infix.as_str());
            hilfe_text.push(']');
            let NonEmpty { head, tail } = lang;
            möglichkeiten_als_regex(head, tail.as_slice(), &mut hilfe_text);
            if let Some((h, t)) = kurz.split_first() {
                hilfe_text.push_str(" | ");
                hilfe_text.push_str(kurz_präfix.as_str());
                möglichkeiten_als_regex(h, t, &mut hilfe_text);
            }
            let cow: Option<Cow<'_, str>> = match (hilfe, standard) {
                (None, None) => None,
                (None, Some(standard)) => {
                    Some(Cow::Owned(format!("{meta_standard}: {}", anzeige(standard))))
                },
                (Some(hilfe), None) => Some(Cow::Borrowed(hilfe)),
                (Some(hilfe), Some(standard)) => {
                    let mut hilfe_text = (*hilfe).to_owned();
                    hilfe_text.push(' ');
                    hilfe_text.push_str(meta_standard);
                    hilfe_text.push_str(": ");
                    hilfe_text.push_str(&anzeige(standard));
                    Some(Cow::Owned(hilfe_text))
                },
            };
            (hilfe_text, cow)
        }
    }

    /// Es handelt sich um ein Flag-Argument, das zu frühem beenden führt.
    ///
    /// ## English
    /// It is a flag argument, causing an early exit.
    #[derive(Debug)]
    pub struct FrühesBeenden<'t> {
        /// Allgemeine Beschreibung des Arguments.
        ///
        /// ## English
        /// General description of the argument.
        pub beschreibung: Beschreibung<'t, Void>,

        /// Die angezeigte Nachricht.
        ///
        /// ## English
        /// The message.
        pub nachricht: Cow<'t, str>,
    }

    impl FrühesBeenden<'_> {
        fn parse<T, Fehler>(
            &self,
            args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'_, T, Fehler>, Vec<Option<OsString>>) {
            todo!()
        }

        pub fn erzeuge_hilfe_text(&self) -> (String, Option<Cow<'_, str>>) {
            let FrühesBeenden { beschreibung, nachricht: _ } = self;
            let Beschreibung { name, hilfe, standard } = beschreibung;
            let Name { lang_präfix, lang, kurz_präfix, kurz } = name;
            let mut hilfe_text = String::new();
            hilfe_text.push_str(lang_präfix.as_str());
            let NonEmpty { head, tail } = lang;
            möglichkeiten_als_regex(head, tail.as_slice(), &mut hilfe_text);
            if let Some((h, t)) = kurz.split_first() {
                hilfe_text.push_str(" | ");
                hilfe_text.push_str(kurz_präfix.as_str());
                möglichkeiten_als_regex(h, t, &mut hilfe_text);
            }
            if let Some(v) = standard {
                void::unreachable(*v)
            }
            let cow = hilfe.map(Cow::Borrowed);
            (hilfe_text, cow)
        }
    }

    /// Es handelt sich um ein Wert-Argument.
    ///
    /// ## English
    /// It is a value argument.
    #[derive(Debug)]
    pub struct Wert<'t, T, Parse, Fehler, Anzeige>
    where
        Parse: Fn(OsString) -> Result<T, ParseFehler<Fehler>>,
        Anzeige: Fn(&T) -> String,
    {
        /// Allgemeine Beschreibung des Arguments.
        ///
        /// ## English
        /// General description of the argument.
        pub beschreibung: Beschreibung<'t, T>,

        /// Infix um einen Wert im selben Argument wie den Namen anzugeben.
        ///
        /// ## English
        /// Infix to give a value in the same argument as the name.
        pub wert_infix: Vergleich<'t>,

        /// Meta-Variable im Hilfe-Text.
        ///
        /// ## English
        /// Meta-variable used in the help-text.
        pub meta_var: &'t str,

        /// String-Darstellung der erlaubten Werte.
        ///
        /// ## English
        /// String-representation of the allowed values.
        pub mögliche_werte: Option<NonEmpty<T>>,

        /// Parse einen Wert aus einem [OsString].
        ///
        /// ## English
        /// Parse a value from an [OsString].
        pub parse: Parse,

        /// Anzeige eines Wertes (standard/mögliche Werte).
        ///
        /// ## English
        /// Display a value (default/possible values).
        pub anzeige: Anzeige,
    }

    fn zeige_elemente<'t, T: 't, Anzeige: Fn(&T) -> String>(
        s: &mut String,
        anzeige: &Anzeige,
        elemente: impl IntoIterator<Item = &'t T>,
    ) {
        let mut erstes = true;
        for element in elemente {
            if erstes {
                erstes = false;
            } else {
                s.push_str(", ");
            }
            s.push_str(&anzeige(element));
        }
    }

    impl<T, Parse, Fehler, Anzeige> Wert<'_, T, Parse, Fehler, Anzeige>
    where
        Parse: Fn(OsString) -> Result<T, ParseFehler<Fehler>>,
        Anzeige: Fn(&T) -> String,
    {
        fn parse(
            &self,
            args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'_, T, Fehler>, Vec<Option<OsString>>) {
            todo!()
        }

        pub fn erzeuge_hilfe_text(
            &self,
            meta_standard: &str,
            meta_erlaubte_werte: &str,
        ) -> (String, Option<Cow<'_, str>>) {
            let Wert { beschreibung, wert_infix, meta_var, mögliche_werte, parse: _, anzeige } =
                self;
            let Beschreibung { name, hilfe, standard } = beschreibung;
            let Name { lang_präfix, lang, kurz_präfix, kurz } = name;
            let mut hilfe_text = String::new();
            hilfe_text.push_str(lang_präfix.as_str());
            let NonEmpty { head, tail } = lang;
            möglichkeiten_als_regex(head, tail.as_slice(), &mut hilfe_text);
            hilfe_text.push_str("( |");
            hilfe_text.push_str(wert_infix.as_str());
            hilfe_text.push(')');
            hilfe_text.push_str(meta_var);
            if let Some((h, t)) = kurz.split_first() {
                hilfe_text.push_str(" | ");
                hilfe_text.push_str(kurz_präfix.as_str());
                möglichkeiten_als_regex(h, t, &mut hilfe_text);
                hilfe_text.push_str("[ |");
                hilfe_text.push_str(wert_infix.as_str());
                hilfe_text.push(']');
                hilfe_text.push_str(meta_var);
            }
            // TODO a lot of code duplication...
            let cow: Option<Cow<'_, str>> = match (hilfe, standard, mögliche_werte) {
                (None, None, None) => None,
                (None, None, Some(mögliche_werte)) => {
                    let mut s = format!("[{meta_erlaubte_werte}: ");
                    zeige_elemente(&mut s, &self.anzeige, mögliche_werte);
                    s.push(']');
                    Some(Cow::Owned(s))
                },
                (None, Some(standard), None) => {
                    Some(Cow::Owned(format!("[{meta_standard}: {}]", anzeige(standard))))
                },
                (None, Some(standard), Some(mögliche_werte)) => {
                    let mut s =
                        format!("[{meta_standard}: {}, {meta_erlaubte_werte}: ", anzeige(standard));
                    zeige_elemente(&mut s, &self.anzeige, mögliche_werte);
                    s.push(']');
                    Some(Cow::Owned(s))
                },
                (Some(hilfe), None, None) => Some(Cow::Borrowed(hilfe)),
                (Some(hilfe), None, Some(mögliche_werte)) => {
                    let mut s = format!("{hilfe} [{meta_erlaubte_werte}: ");
                    zeige_elemente(&mut s, &self.anzeige, mögliche_werte);
                    s.push(']');
                    Some(Cow::Owned(s))
                },
                (Some(hilfe), Some(standard), None) => {
                    Some(Cow::Owned(format!("{hilfe} [{meta_standard}: {}]", anzeige(standard))))
                },
                (Some(hilfe), Some(standard), Some(mögliche_werte)) => {
                    let mut s = format!(
                        "{hilfe} [{meta_standard}: {}, {meta_erlaubte_werte}: ",
                        anzeige(standard)
                    );
                    zeige_elemente(&mut s, &self.anzeige, mögliche_werte);
                    s.push(']');
                    Some(Cow::Owned(s))
                },
            };
            (hilfe_text, cow)
        }
    }

    /// Konfiguration eines einzelnen Kommandozeilen-Arguments.
    ///
    /// ## English
    /// TODO
    #[derive(Debug)]
    pub enum EinzelArgument<'t, T, Bool, Parse, Fehler, Anzeige>
    where
        Bool: Fn(bool) -> T,
        Parse: Fn(OsString) -> Result<T, ParseFehler<Fehler>>,
        Anzeige: Fn(&T) -> String,
    {
        /// Es handelt sich um ein Flag-Argument.
        ///
        /// ## English
        /// It is a flag argument.
        Flag(Flag<'t, T, Bool, Anzeige>),

        /// Es handelt sich um ein Flag-Argument, das zu frühem beenden führt.
        ///
        /// ## English
        /// It is a flag argument, causing an early exit.
        FrühesBeenden(FrühesBeenden<'t>),

        /// Es handelt sich um ein Wert-Argument.
        ///
        /// ## English
        /// It is a value argument.
        Wert(Wert<'t, T, Parse, Fehler, Anzeige>),
    }

    fn möglichkeiten_als_regex(head: &Vergleich<'_>, tail: &[Vergleich<'_>], s: &mut String) {
        if !tail.is_empty() {
            s.push('(')
        }
        s.push_str(head.as_str());
        for l in tail {
            s.push('|');
            s.push_str(l.as_str());
        }
        if !tail.is_empty() {
            s.push(')')
        }
    }

    impl<T, Bool, Parse, Fehler, Anzeige> EinzelArgument<'_, T, Bool, Parse, Fehler, Anzeige>
    where
        T: Clone,
        Bool: Fn(bool) -> T,
        Parse: Fn(OsString) -> Result<T, ParseFehler<Fehler>>,
        Anzeige: Fn(&T) -> String,
    {
        fn parse(
            &self,
            args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'_, T, Fehler>, Vec<Option<OsString>>) {
            match self {
                EinzelArgument::Flag(flag) => flag.parse(args),
                EinzelArgument::FrühesBeenden(frühes_beenden) => frühes_beenden.parse(args),
                EinzelArgument::Wert(wert) => wert.parse(args),
            }
        }
    }

    impl<T, Bool, Parse, Fehler, Anzeige> EinzelArgument<'_, T, Bool, Parse, Fehler, Anzeige>
    where
        Bool: Fn(bool) -> T,
        Parse: Fn(OsString) -> Result<T, ParseFehler<Fehler>>,
        Anzeige: Fn(&T) -> String,
    {
        // [Sprache::standard] kann als meta_standard verwendet werden.
        /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
        pub fn erzeuge_hilfe_text(
            &self,
            meta_standard: &str,
            meta_erlaubte_werte: &str,
        ) -> (String, Option<Cow<'_, str>>) {
            match self {
                EinzelArgument::Flag(flag) => flag.erzeuge_hilfe_text(meta_standard),
                EinzelArgument::FrühesBeenden(frühes_beenden) => {
                    frühes_beenden.erzeuge_hilfe_text()
                },
                EinzelArgument::Wert(wert) => {
                    wert.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
                },
            }
        }
    }

    // TODO standard-implementierung, basierend auf Sprache?
    /// Trait zum simulieren einer Rank-2 Funktion.
    ///
    /// # English
    /// TODO
    pub trait HilfeText {
        fn erzeuge_hilfe_text<'t, S, Bool, Parse, Fehler, Anzeige>(
            arg: &'t EinzelArgument<'t, S, Bool, Parse, Fehler, Anzeige>,
            meta_standard: &'t str,
            meta_erlaubte_werte: &'t str,
        ) -> (String, Option<Cow<'t, str>>)
        where
            Bool: Fn(bool) -> S,
            Parse: Fn(OsString) -> Result<S, ParseFehler<Fehler>>,
            Anzeige: Fn(&S) -> String;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Standard;

    impl HilfeText for Standard {
        #[inline(always)]
        fn erzeuge_hilfe_text<'t, S, Bool, Parse, Fehler, Anzeige>(
            arg: &'t EinzelArgument<'t, S, Bool, Parse, Fehler, Anzeige>,
            meta_standard: &'t str,
            meta_erlaubte_werte: &'t str,
        ) -> (String, Option<Cow<'t, str>>)
        where
            Bool: Fn(bool) -> S,
            Parse: Fn(OsString) -> Result<S, ParseFehler<Fehler>>,
            Anzeige: Fn(&S) -> String,
        {
            arg.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
        }
    }

    /// Erlaube kombinieren mehrerer Argumente.
    ///
    /// ## English
    /// TODO
    pub trait Kombiniere<T, Bool, Parse, Fehler, Anzeige> {
        fn parse(
            &self,
            args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'_, T, Fehler>, Vec<Option<OsString>>);

        /// Erzeuge den Hilfetext für die enthaltenen [Einzelargumente](EinzelArgument).
        fn erzeuge_hilfe_text<H: HilfeText>(
            &self,
            meta_standard: &str,
            meta_erlaubte_werte: &str,
        ) -> Vec<(String, Option<Cow<'_, str>>)>;
    }

    impl<T, Bool, Parse, Fehler, Anzeige> Kombiniere<T, Bool, Parse, Fehler, Anzeige> for Void {
        fn parse(
            &self,
            _args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'_, T, Fehler>, Vec<Option<OsString>>) {
            void::unreachable(*self)
        }

        fn erzeuge_hilfe_text<H: HilfeText>(
            &self,
            _meta_standard: &str,
            _meta_erlaubte_werte: &str,
        ) -> Vec<(String, Option<Cow<'_, str>>)> {
            void::unreachable(*self)
        }
    }

    impl<F, T, B, P, Fehler, A, T0, B0, P0, A0, K0, T1, B1, P1, A1, K1>
        Kombiniere<T, B, P, Fehler, A>
        for (F, ArgTest<'_, T0, B0, P0, Fehler, A0, K0>, ArgTest<'_, T1, B1, P1, Fehler, A1, K1>)
    where
        T0: Clone,
        T1: Clone,
        F: Fn(T0, T1) -> T,
        B0: Fn(bool) -> T0,
        P0: Fn(OsString) -> Result<T0, ParseFehler<Fehler>>,
        A0: Fn(&T0) -> String,
        K0: Kombiniere<T0, B0, P0, Fehler, A0>,
        B1: Fn(bool) -> T1,
        P1: Fn(OsString) -> Result<T1, ParseFehler<Fehler>>,
        A1: Fn(&T1) -> String,
        K1: Kombiniere<T1, B1, P1, Fehler, A1>,
    {
        fn parse(
            &self,
            args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'_, T, Fehler>, Vec<Option<OsString>>) {
            use Ergebnis::*;

            let (f, a0, a1) = self;
            let (e0, nicht_verwendet0) = a0.parse(args);
            let (e1, nicht_verwendet1) = a1.parse(nicht_verwendet0.into_iter());
            let ergebnis = match (e0, e1) {
                (Wert(w0), Wert(w1)) => Wert(f(w0, w1)),
                (Wert(_w0), FrühesBeenden(n1)) => FrühesBeenden(n1),
                (Wert(_w0), Fehler(f1)) => Fehler(f1),
                (FrühesBeenden(n0), Wert(_w1)) => FrühesBeenden(n0),
                (FrühesBeenden(n0), FrühesBeenden(_n1)) => FrühesBeenden(n0),
                (FrühesBeenden(_n0), Fehler(f1)) => Fehler(f1),
                (Fehler(f0), Wert(_w1)) => Fehler(f0),
                (Fehler(f0), FrühesBeenden(_n1)) => Fehler(f0),
                (Fehler(mut f0), Fehler(f1)) => {
                    f0.extend(f1);
                    Fehler(f0)
                },
            };
            (ergebnis, nicht_verwendet1)
        }

        fn erzeuge_hilfe_text<H: HilfeText>(
            &self,
            meta_standard: &str,
            meta_erlaubte_werte: &str,
        ) -> Vec<(String, Option<Cow<'_, str>>)> {
            let (_f, a0, a1) = self;
            let mut hilfe_texte = a0.erzeuge_hilfe_text::<H>(meta_standard, meta_erlaubte_werte);
            hilfe_texte.extend(a1.erzeuge_hilfe_text::<H>(meta_standard, meta_erlaubte_werte));
            hilfe_texte
        }
    }

    /// Konfiguration eines Kommandozeilen-Arguments.
    ///
    /// ## English synonym
    /// [Configuration]
    #[derive(Debug)]
    pub enum ArgTest<'t, T, Bool, Parse, Fehler, Anzeige, K>
    where
        Bool: Fn(bool) -> T,
        Parse: Fn(OsString) -> Result<T, ParseFehler<Fehler>>,
        Anzeige: Fn(&T) -> String,
        K: Kombiniere<T, Bool, Parse, Fehler, Anzeige>,
    {
        /// Ein einzelnes Argument.
        ///
        /// ## English
        /// A single argument.
        EinzelArgument(EinzelArgument<'t, T, Bool, Parse, Fehler, Anzeige>),

        // TODO Kombiniere ähnlich wie aktuell versteckt;
        // erlaube Hilfe-Text erstellen durch Iteration über alle Einzelargumente (rekursiv)
        // mithilfe z.B. des IteriereArg-Traits.
        Kombiniere {
            kombiniere: K,
        },

        Alternativ {
            alternativen: Box<NonEmpty<ArgTest<'t, T, Bool, Parse, Fehler, Anzeige, K>>>,
        },
    }

    impl<T, Bool, Parse, Fehler, Anzeige, K> ArgTest<'_, T, Bool, Parse, Fehler, Anzeige, K>
    where
        T: Clone,
        Bool: Fn(bool) -> T,
        Parse: Fn(OsString) -> Result<T, ParseFehler<Fehler>>,
        Anzeige: Fn(&T) -> String,
        K: Kombiniere<T, Bool, Parse, Fehler, Anzeige>,
    {
        fn parse(
            &self,
            args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'_, T, Fehler>, Vec<Option<OsString>>) {
            use ArgTest::*;
            use Ergebnis::*;
            match self {
                EinzelArgument(arg) => arg.parse(args),
                Kombiniere { kombiniere } => kombiniere.parse(args),
                Alternativ { alternativen } => {
                    let NonEmpty { head, tail } = alternativen.as_ref();
                    let args_vec: Vec<_> = args.into_iter().collect();
                    tail.iter().fold(
                        head.parse(args_vec.clone().into_iter()),
                        |(ergebnis, nicht_verwendet), arg| match ergebnis {
                            Fehler(mut fehler0) => match arg.parse(args_vec.clone().into_iter()) {
                                (Fehler(fehler1), nicht_verwendet1) => {
                                    fehler0.extend(fehler1);
                                    let von_keinem_verwendet = nicht_verwendet
                                        .into_iter()
                                        .filter(|os_string| nicht_verwendet1.contains(os_string))
                                        .collect();
                                    (Fehler(fehler0), von_keinem_verwendet)
                                },
                                end_ergebnis => end_ergebnis,
                            },
                            ergebnis => (ergebnis, nicht_verwendet),
                        },
                    )
                },
            }
        }
    }

    impl<T, Bool, Parse, Fehler, Anzeige, K> ArgTest<'_, T, Bool, Parse, Fehler, Anzeige, K>
    where
        Bool: Fn(bool) -> T,
        Parse: Fn(OsString) -> Result<T, ParseFehler<Fehler>>,
        Anzeige: Fn(&T) -> String,
        K: Kombiniere<T, Bool, Parse, Fehler, Anzeige>,
    {
        // [Sprache::standard] kann als meta_standard verwendet werden.
        /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
        pub fn erzeuge_hilfe_text<H: HilfeText>(
            &self,
            meta_standard: &str,
            meta_erlaubte_werte: &str,
        ) -> Vec<(String, Option<Cow<'_, str>>)> {
            match self {
                ArgTest::EinzelArgument(arg) => {
                    vec![arg.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)]
                },
                ArgTest::Kombiniere { kombiniere } => {
                    kombiniere.erzeuge_hilfe_text::<H>(meta_standard, meta_erlaubte_werte)
                },
                ArgTest::Alternativ { alternativen } => alternativen
                    .iter()
                    .flat_map(|arg| arg.erzeuge_hilfe_text::<H>(meta_standard, meta_erlaubte_werte))
                    .collect(),
            }
        }
    }
}
