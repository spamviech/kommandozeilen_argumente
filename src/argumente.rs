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

pub mod einzelargument;
pub mod flag;
#[path = "argumente/frühes_beenden.rs"]
pub mod frühes_beenden;
pub mod kombiniere;
pub mod wert;

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

    use std::{
        borrow::Cow,
        ffi::{OsStr, OsString},
    };

    use nonempty::NonEmpty;
    use void::Void;

    use crate::{
        argumente::einzelargument::EinzelArgument,
        ergebnis::{Ergebnis, ParseFehler},
    };

    // TODO standard-implementierung, basierend auf Sprache?
    /// Trait zum simulieren einer Rank-2 Funktion.
    ///
    /// # English
    /// Trait to simulate a rank-2 function.
    pub trait HilfeText {
        fn erzeuge_hilfe_text<'t, S, Bool, Parse, Anzeige>(
            arg: &'t EinzelArgument<'t, S, Bool, Parse, Anzeige>,
            meta_standard: &'t str,
            meta_erlaubte_werte: &'t str,
        ) -> (String, Option<Cow<'t, str>>)
        where
            Anzeige: Fn(&S) -> String;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Standard;

    impl HilfeText for Standard {
        #[inline(always)]
        fn erzeuge_hilfe_text<'t, S, Bool, Parse, Anzeige>(
            arg: &'t EinzelArgument<'t, S, Bool, Parse, Anzeige>,
            meta_standard: &'t str,
            meta_erlaubte_werte: &'t str,
        ) -> (String, Option<Cow<'t, str>>)
        where
            Anzeige: Fn(&S) -> String,
        {
            arg.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
        }
    }

    /// Erlaube kombinieren mehrerer Argumente.
    ///
    /// ## English
    /// Allow combining multiple arguments.
    pub trait Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige> {
        fn parse(
            self,
            args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>);

        /// Erzeuge den Hilfetext für die enthaltenen [Einzelargumente](EinzelArgument).
        fn erzeuge_hilfe_text<H: HilfeText>(
            &self,
            meta_standard: &str,
            meta_erlaubte_werte: &str,
        ) -> Vec<(String, Option<Cow<'_, str>>)>;
    }

    impl<'t, T, Bool, Parse, Fehler, Anzeige> Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige> for Void {
        fn parse(
            self,
            _args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>) {
            void::unreachable(self)
        }

        fn erzeuge_hilfe_text<H: HilfeText>(
            &self,
            _meta_standard: &str,
            _meta_erlaubte_werte: &str,
        ) -> Vec<(String, Option<Cow<'_, str>>)> {
            void::unreachable(*self)
        }
    }

    impl<'t, F, T0, T1, B0, B1, P0, P1, Fehler, A0, A1, K0> Kombiniere<'t, T1, B1, P1, Fehler, A1>
        for (F, ArgTest<'t, T0, B0, P0, A0, K0>)
    where
        F: Fn(T0) -> T1,
        B0: Fn(bool) -> T0,
        A0: Fn(&T0) -> String,
        P0: Fn(&OsStr) -> Result<T0, ParseFehler<Fehler>>,
        K0: Kombiniere<'t, T0, B0, P0, Fehler, A0>,
    {
        fn parse(
            self,
            args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'t, T1, Fehler>, Vec<Option<OsString>>) {
            let (f, argument) = self;
            let (ergebnis, nicht_verwendet) = argument.parse(args);
            (ergebnis.konvertiere(f), nicht_verwendet)
        }

        fn erzeuge_hilfe_text<H: HilfeText>(
            &self,
            meta_standard: &str,
            meta_erlaubte_werte: &str,
        ) -> Vec<(String, Option<Cow<'_, str>>)> {
            self.1.erzeuge_hilfe_text::<H, Fehler>(meta_standard, meta_erlaubte_werte)
        }
    }

    impl<'t, 't0, 't1, K, T, B, P, F, A, T0, B0, P0, F0, A0, K0, T1, B1, P1, F1, A1, K1>
        Kombiniere<'t, T, B, P, F, A>
        for (K, ArgTest<'t0, T0, B0, P0, A0, K0>, ArgTest<'t1, T1, B1, P1, A1, K1>)
    where
        't0: 't,
        't1: 't,
        K: Fn(T0, T1) -> T,
        B0: Fn(bool) -> T0,
        P0: Fn(&OsStr) -> Result<T0, ParseFehler<F0>>,
        F0: Into<F>,
        A0: Fn(&T0) -> String,
        K0: Kombiniere<'t0, T0, B0, P0, F0, A0>,
        B1: Fn(bool) -> T1,
        P1: Fn(&OsStr) -> Result<T1, ParseFehler<F1>>,
        F1: Into<F>,
        A1: Fn(&T1) -> String,
        K1: Kombiniere<'t1, T1, B1, P1, F1, A1>,
    {
        fn parse(
            self,
            args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'t, T, F>, Vec<Option<OsString>>) {
            use Ergebnis::*;

            let (f, a0, a1) = self;
            let (e0, nicht_verwendet0) = a0.parse(args);
            let (e1, nicht_verwendet1) = a1.parse(nicht_verwendet0.into_iter());
            let ergebnis = match (e0, e1) {
                (Wert(w0), Wert(w1)) => Wert(f(w0, w1)),
                (Wert(_w0), FrühesBeenden(n1)) => FrühesBeenden(n1),
                (Wert(_w0), Fehler(f1)) => Fehler(f1.map(|fehler| fehler.konvertiere(F1::into))),
                (FrühesBeenden(n0), Wert(_w1)) => FrühesBeenden(n0),
                (FrühesBeenden(n0), FrühesBeenden(_n1)) => FrühesBeenden(n0),
                (FrühesBeenden(_n0), Fehler(f1)) => {
                    Fehler(f1.map(|fehler| fehler.konvertiere(F1::into)))
                },
                (Fehler(f0), Wert(_w1)) => Fehler(f0.map(|fehler| fehler.konvertiere(F0::into))),
                (Fehler(f0), FrühesBeenden(_n1)) => {
                    Fehler(f0.map(|fehler| fehler.konvertiere(F0::into)))
                },
                (Fehler(f0), Fehler(f1)) => {
                    let mut f = f0.map(|fehler| fehler.konvertiere(F0::into));
                    f.extend(f1.into_iter().map(|fehler| fehler.konvertiere(F1::into)));
                    Fehler(f)
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
            let mut hilfe_texte =
                a0.erzeuge_hilfe_text::<H, F0>(meta_standard, meta_erlaubte_werte);
            hilfe_texte.extend(a1.erzeuge_hilfe_text::<H, F1>(meta_standard, meta_erlaubte_werte));
            hilfe_texte
        }
    }

    /// Konfiguration eines Kommandozeilen-Arguments.
    ///
    /// ## English synonym
    /// [Configuration]
    #[derive(Debug)]
    pub enum ArgTest<'t, T, Bool, Parse, Anzeige, K> {
        /// Ein einzelnes Argument.
        ///
        /// ## English
        /// A single argument.
        EinzelArgument(EinzelArgument<'t, T, Bool, Parse, Anzeige>),

        Kombiniere {
            kombiniere: K,
        },

        Alternativ {
            alternativen: Box<NonEmpty<ArgTest<'t, T, Bool, Parse, Anzeige, K>>>,
        },
    }

    impl<'t, T, Bool, Parse, Fehler, Anzeige, K> ArgTest<'t, T, Bool, Parse, Anzeige, K>
    where
        Bool: Fn(bool) -> T,
        Parse: Fn(&OsStr) -> Result<T, ParseFehler<Fehler>>,
        K: Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige>,
    {
        pub fn parse(
            self,
            args: impl Iterator<Item = Option<OsString>>,
        ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>) {
            use ArgTest::*;
            use Ergebnis::*;
            match self {
                EinzelArgument(arg) => arg.parse(args),
                Kombiniere { kombiniere } => kombiniere.parse(args),
                Alternativ { alternativen } => {
                    let NonEmpty { head, tail } = *alternativen;
                    let args_vec: Vec<_> = args.into_iter().collect();
                    tail.into_iter().fold(
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

    impl<'t, T, Bool, Parse, Anzeige, K> ArgTest<'t, T, Bool, Parse, Anzeige, K> {
        // [Sprache::standard] kann als meta_standard verwendet werden.
        /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
        pub fn erzeuge_hilfe_text<H: HilfeText, Fehler>(
            &self,
            meta_standard: &str,
            meta_erlaubte_werte: &str,
        ) -> Vec<(String, Option<Cow<'_, str>>)>
        where
            Anzeige: Fn(&T) -> String,
            K: Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige>,
        {
            match self {
                ArgTest::EinzelArgument(arg) => {
                    vec![arg.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)]
                },
                ArgTest::Kombiniere { kombiniere } => {
                    kombiniere.erzeuge_hilfe_text::<H>(meta_standard, meta_erlaubte_werte)
                },
                ArgTest::Alternativ { alternativen } => alternativen
                    .iter()
                    .flat_map(|arg| {
                        arg.erzeuge_hilfe_text::<H, Fehler>(meta_standard, meta_erlaubte_werte)
                    })
                    .collect(),
            }
        }
    }
}
