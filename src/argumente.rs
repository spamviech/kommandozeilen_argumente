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

    use std::{borrow::Cow, ffi::OsString};

    use nonempty::NonEmpty;

    use crate::{beschreibung::Beschreibung, ergebnis::ParseFehler, unicode::Vergleich};

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
        pub invertiere_präfix_infix: Vergleich<'t>,

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

    /// Es handelt sich um ein Flag-Argument, das zu frühem beenden führt.
    ///
    /// ## English
    /// It is a flag argument, causing an early exit.
    #[derive(Debug)]
    pub struct FrühesBeenden<'t, T> {
        /// Allgemeine Beschreibung des Arguments.
        ///
        /// ## English
        /// General description of the argument.
        pub beschreibung: Beschreibung<'t, T>,

        /// Die angezeigte Nachricht.
        ///
        /// ## English
        /// The message.
        pub nachricht: Cow<'t, str>,
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
        pub beschreibung: Beschreibung<'t, String>,

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
        FrühesBeenden(FrühesBeenden<'t, T>),

        /// Es handelt sich um ein Wert-Argument.
        ///
        /// ## English
        /// It is a value argument.
        Wert(Wert<'t, T, Parse, Fehler, Anzeige>),
    }

    // TODO standard-implementierung, basierend auf Sprache?
    /// Trait zum simulieren einer Rank-2 Funktion.
    ///
    /// # English
    /// TODO
    pub trait HilfeText<Bool, Parse, Fehler, Anzeige> {
        fn erzeuge_hilfe_text<S>(
            arg: &EinzelArgument<'_, S, Bool, Parse, Fehler, Anzeige>,
        ) -> String
        where
            Bool: Fn(bool) -> S,
            Parse: Fn(OsString) -> Result<S, ParseFehler<Fehler>>,
            Anzeige: Fn(&S) -> String;
    }

    /// Erlaube kombinieren mehrerer Argumente.
    ///
    /// ## English
    /// TODO
    pub trait Kombiniere<T, Bool, Parse, Fehler, Anzeige> {
        fn parse(
            &self,
            args: impl Iterator<Item = OsString>,
        ) -> (crate::Ergebnis<'_, T, Fehler>, Vec<OsString>);

        /// Erzeuge den Hilfetext für die enthaltenen [Einzelargumente](EinzelArgument).
        fn erzeuge_hilfe_text<F: HilfeText<Bool, Parse, Fehler, Anzeige>>(
            &self,
            f: F,
        ) -> Vec<String>;
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
}
