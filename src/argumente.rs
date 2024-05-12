//! Definition von akzeptierten Kommandozeilen-Argumenten.

use std::{
    borrow::Cow,
    env,
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
    num::NonZeroI32,
    path::Path,
};

use nonempty::{nonempty, NonEmpty};
use void::Void;

use crate::{
    argumente::{
        einzelargument::EinzelArgument,
        flag::Flag,
        frühes_beenden::FrühesBeenden,
        hilfe::{ErzeugeHilfeText, Hilfe},
        kombiniere::Kombiniere,
        wert::Wert,
    },
    beschreibung::Beschreibung,
    ergebnis::{Ergebnis, Error, Fehler},
    sprache::{Language, Sprache},
};

pub mod einzelargument;
pub mod flag;
#[path = "argumente/frühes_beenden.rs"]
pub mod frühes_beenden;
pub mod hilfe;
pub mod kombiniere;
pub mod wert;

#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use self::wert::EnumArgument;

// TODO Name/Version für Hilfetext angeben, als alternative für macros (derive-Feature)
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

/// Konfiguration der Kommandozeilen-Argumente.
///
/// ## English synonym
/// [`Arguments`]
#[allow(clippy::large_enum_variant, clippy::module_name_repetitions)]
#[must_use]
pub enum Argumente<'t, T, Fehler> {
    /// Ein einzelnes Argument.
    ///
    /// ## English
    /// A single argument.
    EinzelArgument(EinzelArgument<'t, T, Fehler>),
    /// Die Kombination mehrerer Argumente, kodiert über den [`Kombiniere`]-trait.
    ///
    /// ## English
    /// The combination of multiple arguments, encoded via the [`Kombiniere`]-trait.
    Kombiniere(Box<dyn 't + Kombiniere<'t, T, Fehler>>),
    /// Alternative Kommandozeilen-Argumente. Beim parsen wird das erste [`Ergebnis`] verwendet,
    /// dass kein [`Ergebnis::Fehler`] ist.
    ///
    /// ## English
    /// Alternative command line arguments. Parsing takes the first non-[`Error`](Ergebnis::Fehler)
    /// [`Result`](crate::Result).
    Alternativen(Box<NonEmpty<Self>>),
}

impl<T: Debug, Fehler: Debug> Debug for Argumente<'_, T, Fehler> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Argumente::EinzelArgument(arg0) => {
                formatter.debug_tuple("EinzelArgument").field(arg0).finish()
            },
            Argumente::Kombiniere(_arg0) => {
                formatter.debug_tuple("Kombiniere").field(&"<kombiniere>").finish()
            },
            Argumente::Alternativen(arg0) => {
                formatter.debug_tuple("Alternativen").field(arg0).finish()
            },
        }
    }
}

/// Configuration of command line arguments.
///
/// ## Deutsches Synonym
/// [`Argumente`]
pub type Arguments<'t, T, Fehler> = Argumente<'t, T, Fehler>;

impl<'t, T, Fehler> From<EinzelArgument<'t, T, Fehler>> for Argumente<'t, T, Fehler> {
    #[inline]
    fn from(argument: EinzelArgument<'t, T, Fehler>) -> Self {
        Argumente::EinzelArgument(argument)
    }
}

impl<'t, T, Fehler> From<Flag<'t, T>> for Argumente<'t, T, Fehler> {
    #[inline]
    fn from(flag: Flag<'t, T>) -> Self {
        Argumente::EinzelArgument(EinzelArgument::from(flag))
    }
}

impl<'t, T, Fehler> From<(FrühesBeenden<'t>, T)> for Argumente<'t, T, Fehler> {
    #[inline]
    fn from((frühes_beenden, wert): (FrühesBeenden<'t>, T)) -> Self {
        Argumente::EinzelArgument(EinzelArgument::FrühesBeenden { frühes_beenden, wert })
    }
}

impl<'t, Fehler> From<FrühesBeenden<'t>> for Argumente<'t, (), Fehler> {
    #[inline]
    fn from(frühes_beenden: FrühesBeenden<'t>) -> Self {
        Argumente::EinzelArgument(EinzelArgument::FrühesBeenden { frühes_beenden, wert: () })
    }
}

impl<'t, T, Fehler> From<Wert<'t, T, Fehler>> for Argumente<'t, T, Fehler> {
    #[inline]
    fn from(wert: Wert<'t, T, Fehler>) -> Self {
        Argumente::EinzelArgument(EinzelArgument::Wert(wert))
    }
}

impl<T, Fehler> From<NonEmpty<Self>> for Argumente<'_, T, Fehler> {
    #[inline]
    fn from(alternativen: NonEmpty<Self>) -> Self {
        Argumente::Alternativen(Box::new(alternativen))
    }
}

impl<T, Fehler> From<Box<NonEmpty<Self>>> for Argumente<'_, T, Fehler> {
    #[inline]
    fn from(alternativen: Box<NonEmpty<Self>>) -> Self {
        Argumente::Alternativen(alternativen)
    }
}

impl<'t, T, Fehler> Argumente<'t, T, Fehler> {
    /// Erzeuge eine [`Argumente::EinzelArgument`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`Argumente::EinzelArgument`]-variant with sensible type parameters.
    #[inline]
    pub fn einzel_argument(einzel_argument: EinzelArgument<'t, T, Fehler>) -> Self {
        Argumente::EinzelArgument(einzel_argument)
    }
}

impl<'t, T, Fehler> Argumente<'t, T, Fehler> {
    /// Erzeuge eine [`Argumente::Kombiniere`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`Argumente::Kombiniere`]-variant with sensible type parameters.
    #[inline]
    pub fn kombiniere(kombiniere: impl 't + Kombiniere<'t, T, Fehler>) -> Self {
        Argumente::Kombiniere(Box::new(kombiniere))
    }

    /// Erzeuge eine [`Argumente::Alternativen`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`Argumente::Alternativen`]-variant with sensible type parameters.
    #[inline]
    pub fn alternativen(alternativen: NonEmpty<Self>) -> Self {
        Argumente::Alternativen(Box::new(alternativen))
    }

    /// Erzeuge eine [`Argumente::Alternativen`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`Argumente::Alternativen`]-variant with sensible type parameters.
    #[inline]
    pub fn alternativen_boxed(alternativen: Box<NonEmpty<Self>>) -> Self {
        Argumente::Alternativen(alternativen)
    }
}

impl<'t, T, F> Argumente<'t, T, F> {
    /// Parse die übergebenen Argumente und erzeuge den zugehörigen Wert.
    ///
    /// ## English
    /// Parse the given arguments and return the corresponding value.
    #[inline]
    pub fn parse(
        self,
        args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, T, F>, Vec<Option<OsString>>) {
        use Argumente::{Alternativen, EinzelArgument, Kombiniere};
        use Ergebnis::{Fehler, FrühesBeenden, Wert};
        match self {
            EinzelArgument(arg) => arg.parse(args),
            Kombiniere(kombiniere) => kombiniere.parse(Box::new(args)),
            Alternativen(alternativen) => {
                // TODO only accept parsing without leftover args?
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
                        Wert(_) | FrühesBeenden(_) => (ergebnis, nicht_verwendet),
                    },
                )
            },
        }
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    ///
    /// ## English synonym
    /// [`parse_from_env`](Parse::parse_from_env)
    #[inline]
    pub fn parse_aus_env(self) -> (Ergebnis<'t, T, F>, Vec<OsString>)
    where
        Self: 't,
        F: 't,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_aus_env()
    }

    /// Parse [`args_os`](std::env::args_os) and try to create the requested type.
    ///
    /// ## Deutsches Synonym
    /// [`parse_aus_env`](Parse::parse_aus_env)
    #[inline]
    pub fn parse_from_env(self) -> (Ergebnis<'t, T, F>, Vec<OsString>)
    where
        Self: 't,
        F: 't,
    {
        self.parse_aus_env()
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [`parse_from_env_with_early_exit`](Parse::parse_from_env_with_early_exit)
    #[inline]
    pub fn parse_aus_env_mit_frühen_beenden(
        self,
    ) -> (Result<T, NonEmpty<Fehler<'t, F>>>, Vec<OsString>)
    where
        Self: 't,
        F: 't,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_aus_env_mit_frühen_beenden()
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_aus_env_mit_frühen_beenden`](Argumente::parse_aus_env_mit_frühen_beenden)
    #[inline]
    pub fn parse_from_env_with_early_exit(
        self,
    ) -> (Result<T, NonEmpty<Error<'t, F>>>, Vec<OsString>)
    where
        Self: 't,
        F: 't,
    {
        self.parse_aus_env_mit_frühen_beenden()
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [`parse_with_early_exit`](Self::parse_with_early_exit)
    #[inline]
    pub fn parse_mit_frühen_beenden(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> (Result<T, NonEmpty<Fehler<'t, F>>>, Vec<OsString>)
    where
        Self: 't,
        F: 't,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_mit_frühen_beenden(args)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_mit_frühen_beenden`](Self::parse_mit_frühen_beenden)
    #[inline]
    fn parse_with_early_exit(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> (Result<T, NonEmpty<Error<'t, F>>>, Vec<OsString>)
    where
        Self: 't,
        F: 't,
    {
        self.parse_mit_frühen_beenden(args)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [`parse_complete`](Self::parse_complete)
    #[inline]
    #[must_use]
    pub fn parse_vollständig(
        self,
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        arg_nicht_verwendet: &str,
    ) -> T
    where
        F: Display,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_vollständig(
        //     args,
        //     fehler_code,
        //     fehlende_flag,
        //     fehlender_wert,
        //     parse_fehler,
        //     invalider_string,
        //     arg_nicht_verwendet,
        // )
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig`](Self::parse_vollständig)
    #[inline]
    #[must_use]
    pub fn parse_complete(
        self,
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> T
    where
        F: Display,
    {
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

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [`parse_complete_with_language`](Self::parse_complete_with_language)
    #[inline]
    #[must_use]
    pub fn parse_vollständig_mit_sprache(
        self,
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        sprache: Sprache,
    ) -> T
    where
        F: Display,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_vollständig_mit_sprache(args, fehler_code, sprache)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig_mit_sprache`](Self::parse_vollständig_mit_sprache)
    #[inline]
    #[must_use]
    pub fn parse_complete_with_language(
        self,
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        language: Language,
    ) -> T
    where
        F: Display,
    {
        self.parse_vollständig_mit_sprache(args, error_code, language)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English version
    /// [`parse_with_error_message`](Self::parse_with_error_message)
    #[inline]
    #[must_use]
    pub fn parse_mit_fehlermeldung(
        self,
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
    ) -> T
    where
        F: Display,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_mit_fehlermeldung(args, fehler_code)
    }

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsche version
    /// [`parse_mit_fehlermeldung`](Self::parse_mit_fehlermeldung)
    #[inline]
    #[must_use]
    pub fn parse_with_error_message(
        self,
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
    ) -> T
    where
        F: Display,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_with_error_message(args, error_code)
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [`parse_complete_from_env`](Self::parse_complete_from_env)
    #[inline]
    #[must_use]
    pub fn parse_vollständig_aus_env(
        self,
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        arg_nicht_verwendet: &str,
    ) -> T
    where
        F: Display,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_vollständig_aus_env(
        //     fehler_code,
        //     fehlende_flag,
        //     fehlender_wert,
        //     parse_fehler,
        //     invalider_string,
        //     arg_nicht_verwendet,
        // )
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig_aus_env`](Self::parse_vollständig_aus_env)
    #[inline]
    #[must_use]
    pub fn parse_complete_from_env(
        self,
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> T
    where
        F: Display,
    {
        self.parse_vollständig_aus_env(
            error_code,
            missing_flag,
            missing_value,
            parse_error,
            invalid_string,
            unused_arg,
        )
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [`parse_complete_with_language_from_env`](Self::parse_complete_with_language_from_env)
    #[inline]
    #[must_use]
    pub fn parse_vollständig_mit_sprache_aus_env(
        self,
        fehler_code: NonZeroI32,
        sprache: Sprache,
    ) -> T
    where
        F: Display,
    {
        todo!()
        // Self::kommandozeilen_argumente()
        //     .parse_vollständig_mit_sprache_aus_env(fehler_code, sprache)
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig_mit_sprache_aus_env`](Self::parse_vollständig_mit_sprache_aus_env)
    #[inline]
    #[must_use]
    pub fn parse_complete_with_language_from_env(
        self,
        error_code: NonZeroI32,
        language: Language,
    ) -> T
    where
        F: Display,
    {
        self.parse_vollständig_mit_sprache_aus_env(error_code, language)
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English version
    /// [`parse_with_error_message_from_env`](Self::parse_with_error_message_from_env)
    #[inline]
    #[must_use]
    pub fn parse_mit_fehlermeldung_aus_env(self, fehler_code: NonZeroI32) -> T
    where
        F: Display,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_mit_fehlermeldung_aus_env(fehler_code)
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsche Version
    /// [`parse_mit_fehlermeldung_aus_env`](Self::parse_mit_fehlermeldung_aus_env)
    #[inline]
    #[must_use]
    pub fn parse_with_error_message_from_env(self, error_code: NonZeroI32) -> T
    where
        F: Display,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_with_error_message_from_env(error_code)
    }
}

impl<T, Fehler> Argumente<'_, T, Fehler> {
    /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
    ///
    /// ## English
    /// Create the Message for the syntax of the arguments and the corresponding help text.
    #[inline]
    #[must_use]
    // panic when a programming error occurs
    #[allow(clippy::missing_panics_doc)]
    pub fn erzeuge_hilfe_text(
        &self,
        variante: &dyn ErzeugeHilfeText,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> NonEmpty<hilfe::Alternativen<'_>> {
        match self {
            Argumente::EinzelArgument(arg) => {
                nonempty![hilfe::Alternativen::EinzelArgument(
                    arg.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
                )]
            },
            Argumente::Kombiniere(kombiniere) => {
                kombiniere.erzeuge_hilfe_text(variante, meta_standard, meta_erlaubte_werte)
            },
            Argumente::Alternativen(alternativen) => {
                // TODO use alternativen.as_ref().flat_map(...), coming in nonempty > 0.10.0
                NonEmpty::collect(alternativen.iter().map(|arg| {
                    hilfe::Alternativen::Alternativen(Box::new(arg.erzeuge_hilfe_text(
                        variante,
                        meta_standard,
                        meta_erlaubte_werte,
                    )))
                }))
                .expect("NonEmpty::map(...) hat mindestens ein Argument!")
            },
        }
    }
}

impl<'t, T, Fehler> Argumente<'t, T, Fehler> {
    /// Füge eine [`FrühesBeenden`]-Flag hinzu, wodurch die Programm-Version anzeigt wird.
    ///
    /// ## English
    /// Add an [`EarlyExit`](crate::frühes_beenden::EarlyExit`)-flag, showing the program version.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn mit_version_frühes_beenden(
        self,
        eigene_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_version: &str,
    ) -> Self {
        let name_und_version = format!("{programm_name} {programm_version}");
        let frühes_beenden = FrühesBeenden {
            beschreibung: eigene_beschreibung,
            nachricht: Cow::Owned(name_und_version),
        };
        let kombiniere =
            (|wert: T, ()| wert, self, Argumente::from(EinzelArgument::from(frühes_beenden)));
        Argumente::kombiniere(kombiniere)
    }

    /// Füge eine [`FrühesBeenden`]-Flag hinzu, wodurch der Hilfe-Text für alle Argumente anzeigt wird.
    ///
    /// ### Panics
    /// Wenn die Syntax-Beschreibung (inklusive normalem + Alternativen-Präfix) für ein Argument
    /// länger als [`usize::MAX`] ist.
    ///
    /// ## English
    /// Add an [`EarlyExit`](crate::frühes_beenden::EarlyExit`)-Flag, showing the help text for all arguments.
    ///
    /// ### Panics
    /// If the syntax-description (including normal + alternativ prefixes) for an argument exceeds [`usize::MAX`].
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn mit_hilfe_frühes_beenden(
        self,
        variante: &dyn ErzeugeHilfeText,
        eigene_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: Option<&str>,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
        meta_optionen: &str,
        meta_syntax_präfix: &str,
        meta_syntax_padding: char,
        meta_alternative_präfix: &str,
        meta_alternative_trennzeichen: char,
    ) -> Self {
        let mut hilfen = self.erzeuge_hilfe_text(variante, meta_standard, meta_erlaubte_werte);
        let dummy = Cow::Borrowed("");
        let mut frühes_beenden =
            FrühesBeenden { beschreibung: eigene_beschreibung, nachricht: dummy };
        hilfen.push(hilfe::Alternativen::EinzelArgument(frühes_beenden.erzeuge_hilfe_text()));
        let hilfen = hilfen;
        let max_syntax_breite = max_syntax_breite(&hilfen, meta_alternative_präfix);
        let current_exe = env::current_exe().ok();
        let exe_name = current_exe
            .as_deref()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .unwrap_or(programm_name);
        let mut name = String::from(programm_name);
        if let Some(version) = programm_version {
            name.push(' ');
            name.push_str(version);
        }
        let programm_beschreibung = programm_beschreibung
            .map(|beschreibung| format!("\n{beschreibung}"))
            .unwrap_or_default();
        let mut hilfe_text = format!(
            "{name}{programm_beschreibung}\n\n{exe_name} [{meta_optionen}]\n\n{meta_optionen}:\n"
        );
        for hilfe in hilfen {
            schreibe_argument_oder_alternativen(
                &mut hilfe_text,
                Cow::Borrowed(meta_syntax_präfix),
                #[allow(clippy::arithmetic_side_effects)]
                {
                    meta_syntax_präfix.len() + max_syntax_breite + 1
                },
                meta_syntax_padding,
                &hilfe,
                meta_alternative_präfix,
                meta_alternative_trennzeichen,
            );
        }
        frühes_beenden.nachricht = Cow::Owned(hilfe_text);
        let kombiniere =
            (|wert: T, ()| wert, self, Argumente::from(EinzelArgument::from(frühes_beenden)));
        Argumente::kombiniere(kombiniere)
    }

    /// Variante von [`mit_hilfe_frühes_beenden`](Self::mit_hilfe_frühes_beenden),
    /// basierend auf einer [`Sprache`].
    ///
    /// ## English
    /// Variant of [`mit_hilfe_frühes_beenden`](Self::mit_hilfe_frühes_beenden),
    /// based on a [`Language`](crate::sprache::Language).
    #[inline]
    pub fn mit_hilfe_frühes_beenden_mit_sprache(
        self,
        variante: &dyn ErzeugeHilfeText,
        eigene_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: Option<&str>,
        sprache: Sprache,
    ) -> Self {
        self.mit_hilfe_frühes_beenden(
            variante,
            eigene_beschreibung,
            programm_name,
            programm_beschreibung,
            programm_version,
            sprache.standard,
            sprache.erlaubte_werte,
            sprache.optionen,
            sprache.syntax_präfix,
            sprache.syntax_padding,
            sprache.alternative_präfix,
            sprache.alternative_trennzeichen,
        )
    }

    /// Füge [`FrühesBeenden`]-Flags hinzu, wodurch die Programm-Version,
    /// bzw. der Hilfe-Text für alle Argumente anzeigt wird.
    ///
    /// ### Panics
    /// Wenn die Syntax-Beschreibung (inklusive normalem + Alternativen-Präfix) für ein Argument
    /// länger als [`usize::MAX`] ist.
    ///
    /// ## English
    /// Add [`EarlyExit`](crate::frühes_beenden::EarlyExit`)-Flags, showing the program version,
    /// or the help text for all arguments.
    ///
    /// ### Panics
    /// If the syntax-description (including normal + alternativ prefixes) for an argument exceeds [`usize::MAX`].
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn mit_hilfe_und_version_frühes_beenden(
        self,
        variante: &dyn ErzeugeHilfeText,
        version_beschreibung: Beschreibung<'t, Void>,
        hilfe_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: &str,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
        meta_optionen: &str,
        meta_syntax_präfix: &str,
        meta_syntax_padding: char,
        meta_alternative_präfix: &str,
        meta_alternative_trennzeichen: char,
    ) -> Self {
        self.mit_version_frühes_beenden(version_beschreibung, programm_name, programm_version)
            .mit_hilfe_frühes_beenden(
                variante,
                hilfe_beschreibung,
                programm_name,
                programm_beschreibung,
                Some(programm_version),
                meta_standard,
                meta_erlaubte_werte,
                meta_optionen,
                meta_syntax_präfix,
                meta_syntax_padding,
                meta_alternative_präfix,
                meta_alternative_trennzeichen,
            )
    }

    /// Variante von [`mit_hilfe_und_version_frühes_beenden`](Argumente::mit_hilfe_und_version_frühes_beenden).
    ///
    /// ## English
    /// Variant of [`mit_hilfe_und_version_frühes_beenden`](Argumente::mit_hilfe_und_version_frühes_beenden).
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn mit_hilfe_und_version_frühes_beenden_mit_sprache(
        self,
        variante: &dyn ErzeugeHilfeText,
        version_beschreibung: Beschreibung<'t, Void>,
        hilfe_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: &str,
        sprache: Sprache,
    ) -> Self {
        self.mit_hilfe_und_version_frühes_beenden(
            variante,
            version_beschreibung,
            hilfe_beschreibung,
            programm_name,
            programm_beschreibung,
            programm_version,
            sprache.standard,
            sprache.erlaubte_werte,
            sprache.optionen,
            sprache.syntax_präfix,
            sprache.syntax_padding,
            sprache.alternative_präfix,
            sprache.alternative_trennzeichen,
        )
    }
}

/// Berechne die maximale Breite für die Syntax eines Argumentes.
///
/// Hilfsfunktion für [`Argumente::mit_hilfe_frühes_beenden`]
///
/// ## Panics
/// Programmierfehler, wenn `NonEmpty::iter().map(...)` kein Element hat.
fn max_syntax_breite(
    hilfen: &NonEmpty<hilfe::Alternativen<'_>>,
    alternative_präfix: &str,
) -> usize {
    hilfen
        .iter()
        .map(|arg| match arg {
            hilfe::Alternativen::EinzelArgument(arg) => arg.syntax.len(),
            hilfe::Alternativen::Alternativen(alternativen) => {
                #[allow(clippy::arithmetic_side_effects)]
                {
                    alternative_präfix.len() + max_syntax_breite(alternativen, alternative_präfix)
                }
            },
        })
        .max()
        .expect("NonEmpty")
}

/// Schreibe den Hilfetext für den aktuellen Eintrag oder alle Alternativen.
///
/// Hilfsfunktion für [`Argumente::mit_hilfe_frühes_beenden`]
///
/// ## Panics
/// If `max_syntax_breite < aktueller_präfix.len() + syntax.len()` for any entry.
/// If `max_syntax_breite < aktueller_präfix.len()` for any entry.
fn schreibe_argument_oder_alternativen(
    string: &mut String,
    aktueller_präfix: Cow<'_, str>,
    max_syntax_breite: usize,
    syntax_padding: char,
    eintrag: &hilfe::Alternativen<'_>,
    alternative_präfix: &str,
    alternative_trennzeichen: char,
) {
    match eintrag {
        hilfe::Alternativen::EinzelArgument(arg) => {
            let Hilfe { syntax, hilfe } = arg;
            string.push_str(&aktueller_präfix);
            string.push_str(syntax);
            #[allow(clippy::arithmetic_side_effects)]
            let padding = max_syntax_breite - aktueller_präfix.len() - syntax.len();
            let mut buffer: [u8; 4] = [0; 4];
            let padding_string = syntax_padding.encode_utf8(&mut buffer).repeat(padding);
            string.push_str(&padding_string);
            if let Some(hilfe) = hilfe {
                string.push_str(hilfe);
            }
            string.push('\n');
        },
        hilfe::Alternativen::Alternativen(alternativen) => {
            #[allow(clippy::arithmetic_side_effects)]
            let trennzeile_breite = max_syntax_breite - aktueller_präfix.len();
            let mut buffer: [u8; 4] = [0; 4];
            let trennzeile = format!(
                "{aktueller_präfix}{}",
                alternative_trennzeichen.encode_utf8(&mut buffer).repeat(trennzeile_breite)
            );
            let mut neuer_präfix = aktueller_präfix.into_owned();
            neuer_präfix.push_str(alternative_präfix);
            let mut first = true;
            for alternative in alternativen.iter() {
                if first {
                    first = false;
                } else {
                    string.push_str(&trennzeile);
                    string.push('\n');
                }
                schreibe_argument_oder_alternativen(
                    string,
                    Cow::Borrowed(&neuer_präfix),
                    max_syntax_breite,
                    syntax_padding,
                    alternative,
                    alternative_präfix,
                    alternative_trennzeichen,
                );
            }
        },
    }
}
