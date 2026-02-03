//! Definition von akzeptierten Kommandozeilen-Argumenten.

use std::{
    borrow::Cow,
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
    num::NonZeroI32,
    path::Path,
    process,
};

use dyn_clone::{clone_trait_object, DynClone};
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
    beschreibung::{ArgumentInput, Beschreibung},
    dyn_to_owned,
    ergebnis::{Ergebnis, Error, Fehler, ParseFehler, ZwischenErgebnis},
    sprache::{Language, Sprache},
    Description,
};

pub mod einzelargument;
pub mod flag;
#[path = "argumente/frühes_beenden.rs"]
pub mod frühes_beenden;
pub mod hilfe;
pub mod kombiniere;
pub mod parser;
pub mod wert;

#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use self::wert::EnumArgument;

// TODO Name/Version für Hilfetext angeben, als alternative für macros (derive-Feature)
// TODO Unterbefehle/subcommands
// TODO Positions-basierte Argumente
// TODO tests mit Unicode-namen

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

/// Helper umd [`Kombiniere::debug_fmt`] mit [`fmt::Formatter::debug_tuple`] zu verwenden.
struct KombiniereDebug<'s, 't, T, Fehler>(&'s dyn Kombiniere<'t, T, Fehler>);

impl<T, Fehler> Debug for KombiniereDebug<'_, '_, T, Fehler> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.debug_fmt(formatter)
    }
}

impl<T: Debug, Fehler: Debug> Debug for Argumente<'_, T, Fehler> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Argumente::EinzelArgument(arg0) => {
                formatter.debug_tuple("EinzelArgument").field(arg0).finish()
            },
            Argumente::Kombiniere(arg0) => {
                formatter.debug_tuple("Kombiniere").field(&KombiniereDebug(&**arg0)).finish()
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

impl<'t, T, Fehler, Anzeige: 't + Fn(&T) -> String + Clone> From<(FrühesBeenden<'t>, T, Anzeige)>
    for Argumente<'t, T, Fehler>
{
    #[inline]
    fn from((frühes_beenden, wert, anzeige): (FrühesBeenden<'t>, T, Anzeige)) -> Self {
        let anzeige_boxed: Box<dyn 't + dyn_to_owned::Anzeige<'t, T>> = Box::new(anzeige);
        Argumente::EinzelArgument(EinzelArgument::FrühesBeenden {
            frühes_beenden,
            wert,
            anzeige: Cow::Owned(anzeige_boxed),
        })
    }
}

impl<'t, T: Display, Fehler> From<(FrühesBeenden<'t>, T)> for Argumente<'t, T, Fehler> {
    #[inline]
    fn from((frühes_beenden, wert): (FrühesBeenden<'t>, T)) -> Self {
        Argumente::EinzelArgument(EinzelArgument::FrühesBeenden {
            frühes_beenden,
            wert,
            anzeige: Cow::Borrowed(&ToString::to_string),
        })
    }
}

impl<'t, Fehler> From<FrühesBeenden<'t>> for Argumente<'t, (), Fehler> {
    #[inline]
    fn from(frühes_beenden: FrühesBeenden<'t>) -> Self {
        Argumente::EinzelArgument(EinzelArgument::FrühesBeenden {
            frühes_beenden,
            wert: (),
            anzeige: Cow::Borrowed(&|wert| format!("{wert:?}")),
        })
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
    /// ## English synonym
    /// [`combine`](Self::combine)
    #[inline]
    pub fn kombiniere(kombiniere: impl 't + Kombiniere<'t, T, Fehler>) -> Self {
        Argumente::Kombiniere(Box::new(kombiniere))
    }

    /// Create a [`Argumente::Kombiniere`]-variant with sensible type parameters.
    ///
    /// ## Deutsches Synonym
    /// [`kombiniere`](Self::combine)
    #[inline]
    pub fn combine(combine: impl 't + Kombiniere<'t, T, Fehler>) -> Self {
        Self::kombiniere(combine)
    }

    /// Erzeuge eine [`Argumente::Alternativen`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English synonym
    /// [`alternatives`](Self::alternatives)
    #[inline]
    pub fn alternativen(alternativen: NonEmpty<Self>) -> Self {
        Argumente::Alternativen(Box::new(alternativen))
    }

    /// Create a [`Argumente::Alternativen`]-variant with sensible type parameters.
    ///
    /// ## Deutsches Synonym
    /// [`alternativen`](Self::alternativen)
    #[inline]
    pub fn alternatives(alternatives: NonEmpty<Self>) -> Self {
        Self::alternativen(alternatives)
    }

    /// Erzeuge eine [`Argumente::Alternativen`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English synonym
    /// [`alternatives_boxed`](Self::alternatives_boxed)
    #[inline]
    pub fn alternativen_boxed(alternativen: Box<NonEmpty<Self>>) -> Self {
        Argumente::Alternativen(alternativen)
    }

    /// Create a [`Argumente::Alternativen`]-variant with sensible type parameters.
    ///
    /// ## Deutsches Synonym
    /// [`alternativen_boxed`](Self::alternativen_boxed)
    #[inline]
    pub fn alternatives_boxed(alternatives: Box<NonEmpty<Self>>) -> Self {
        Self::alternativen_boxed(alternatives)
    }
}

/// TODO
#[derive(Debug)]
pub struct ParsedEarlyExit<'s> {
    /// TODO
    pub name: Cow<'s, str>,
    /// TODO
    pub message: Cow<'s, str>,
    /// TODO
    pub input: Cow<'s, str>,
}
/// TODO
#[derive(Debug)]
pub struct ParsedShortFlag<'s> {
    /// TODO
    pub name: Cow<'s, str>,
    /// TODO
    pub input: Cow<'s, str>,
}
/// TODO
#[derive(Debug)]
pub struct ParsedValueName<'s> {
    /// TODO
    pub name: Cow<'s, str>,
}
/// TODO
#[derive(Debug)]
pub struct ParsedValue<'s> {
    /// TODO
    pub value: Cow<'s, str>,
    /// TODO
    pub input: Cow<'s, str>,
}

/// TODO
#[derive(Debug)]
pub struct ParseMergedShortFormsResult<'s> {
    /// A vector of early\_exit arguments, containing name, message & original input.
    pub early_exits: Vec<ParsedEarlyExit<'s>>,
    /// A vector of flag-arguments with their name (all are true) & the original input.
    pub flags: Vec<ParsedShortFlag<'s>>,
    /// A map of value-arguments with name -> (value-string, original input).
    pub values: HashMap<ParsedValueName<'s>, ParsedValue<'s>>,
    /// Remaining arguments with the parsed merged short names and associated value-strings removed.
    pub remaining: Vec<Option<OsString>>,
}

impl<T, F> Argumente<'_, T, F> {
    /// Parse merged short form arguments.
    ///
    /// Rules to allow merging of short names:
    ///
    /// - All short names in the same string share the same (short) prefix.
    /// - Only short names consisting of a single [grapheme](https://docs.rs/unicode-segmentation/1.8.0/unicode_segmentation/trait.UnicodeSegmentation.html#tymethod.graphemes) participate.
    /// - At most one value argument per block.
    ///   It must be the last argument name in the string, optionally followed by \[a value-infix and\] the value sub-string.
    /// - Merging of short names must be allowed for this particular argument.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'_> {
        use Argumente::{Alternativen, EinzelArgument, Kombiniere};
        match self {
            EinzelArgument(einzelargument) => einzelargument.parse_merged_short_forms(args),
            Kombiniere(kombiniere) => todo!(),
            Alternativen(non_empty) => todo!(),
        }
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
        args: impl Iterator<Item = OsString>,
    ) -> (Ergebnis<'t, T, F>, Vec<ArgumentInput>) {
        todo!("parse({:?})", args.collect::<Vec<_>>());
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    ///
    /// ## English synonym
    /// [`parse_from_env`](Self::parse_from_env)
    #[inline]
    pub fn parse_aus_env(self) -> (Ergebnis<'t, T, F>, Vec<ArgumentInput>)
    where
        Self: 't,
        F: 't,
    {
        self.parse(env::args_os().skip(1))
    }

    /// Parse [`args_os`](std::env::args_os) and try to create the requested type.
    ///
    /// ## Deutsches Synonym
    /// [`parse_aus_env`](Self::parse_aus_env)
    #[inline]
    pub fn parse_from_env(self) -> (Ergebnis<'t, T, F>, Vec<ArgumentInput>)
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
    /// [`parse_from_env_with_early_exit`](Self::parse_from_env_with_early_exit)
    #[inline]
    pub fn parse_aus_env_mit_frühen_beenden(
        self,
    ) -> (Result<T, NonEmpty<Fehler<'t, F>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        F: 't,
    {
        self.parse_mit_frühen_beenden(env::args_os().skip(1))
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_aus_env_mit_frühen_beenden`](Self::parse_aus_env_mit_frühen_beenden)
    #[inline]
    pub fn parse_from_env_with_early_exit(
        self,
    ) -> (Result<T, NonEmpty<Error<'t, F>>>, Vec<ArgumentInput>)
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
    ) -> (Result<T, NonEmpty<Fehler<'t, F>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        F: 't,
    {
        let (ergebnis, nicht_verwendet) = self.parse(args);
        let result = match ergebnis {
            Ergebnis::Wert(wert) => Ok(wert),
            Ergebnis::FrühesBeenden(nachrichten) => {
                #[allow(clippy::print_stdout)]
                for nachricht in nachrichten {
                    println!("{nachricht}");
                }
                process::exit(0);
            },
            Ergebnis::Fehler(fehler) => Err(fehler),
        };
        (result, nicht_verwendet)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_mit_frühen_beenden`](Self::parse_mit_frühen_beenden)
    #[inline]
    pub fn parse_with_early_exit(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> (Result<T, NonEmpty<Error<'t, F>>>, Vec<ArgumentInput>)
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
    #[allow(clippy::too_many_arguments)]
    pub fn parse_vollständig(
        self,
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        argument_nicht_verwendet: &str,
    ) -> T
    where
        F: Display,
    {
        let (ergebnis, nicht_verwendet) = self.parse(args);
        #[allow(clippy::print_stderr)]
        if !nicht_verwendet.is_empty() {
            eprintln!("{argument_nicht_verwendet}");
            process::exit(fehler_code.get());
        }
        match ergebnis {
            Ergebnis::Wert(wert) => wert,
            Ergebnis::FrühesBeenden(nachrichten) => {
                #[allow(clippy::print_stdout)]
                for nachricht in nachrichten {
                    println!("{nachricht}");
                }
                process::exit(0);
            },
            Ergebnis::Fehler(fehler_liste) => {
                #[allow(clippy::print_stderr)]
                for fehler in fehler_liste {
                    let fehlermeldung = fehler.erstelle_fehlermeldung(
                        fehlende_flag,
                        fehlender_wert,
                        parse_fehler,
                        invalider_string,
                    );
                    eprintln!("{fehlermeldung}");
                }
                process::exit(fehler_code.get());
            },
        }
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
    #[allow(clippy::too_many_arguments)]
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
    /// [`parse_vollständig_mit_sprache`](Self::parse_vollständig_mit_sprache) mit [`Sprache::DEUTSCH`].
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
        self.parse_vollständig_mit_sprache(args, fehler_code, Sprache::DEUTSCH)
    }

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// [`parse_complete_with_language`](Self::parse_complete_with_language) with [`Language::ENGLISH`].
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
        self.parse_complete_with_language(args, error_code, Language::ENGLISH)
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
        argument_nicht_verwendet: &str,
    ) -> T
    where
        F: Display,
    {
        self.parse_vollständig(
            env::args_os().skip(1),
            fehler_code,
            fehlende_flag,
            fehlender_wert,
            parse_fehler,
            invalider_string,
            argument_nicht_verwendet,
        )
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
        self.parse_vollständig_mit_sprache(env::args_os().skip(1), fehler_code, sprache)
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
    /// [`parse_vollständig_mit_sprache_aus_env`](Self::parse_vollständig_mit_sprache_aus_env)
    /// mit [`Sprache::DEUTSCH`].
    ///
    /// ## English version
    /// [`parse_with_error_message_from_env`](Self::parse_with_error_message_from_env)
    #[inline]
    #[must_use]
    pub fn parse_mit_fehlermeldung_aus_env(self, fehler_code: NonZeroI32) -> T
    where
        F: Display,
    {
        self.parse_vollständig_mit_sprache_aus_env(fehler_code, Sprache::DEUTSCH)
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// [`parse_complete_with_language_from_env`](Self::parse_complete_with_language_from_env)
    /// with [`Language::ENGLISH`].
    ///
    /// ## Deutsche Version
    /// [`parse_mit_fehlermeldung_aus_env`](Self::parse_mit_fehlermeldung_aus_env)
    #[inline]
    #[must_use]
    pub fn parse_with_error_message_from_env(self, error_code: NonZeroI32) -> T
    where
        F: Display,
    {
        self.parse_complete_with_language_from_env(error_code, Language::ENGLISH)
    }
}

/// Hilfs-Trait zum [Clone]-baren Konvertieren eines Fehlers.
trait KonvertiereFehler<Fehler, NeuerFehler>: Fn(Fehler) -> NeuerFehler + DynClone {}
impl<Fehler, NeuerFehler, F: Fn(Fehler) -> NeuerFehler + DynClone>
    KonvertiereFehler<Fehler, NeuerFehler> for F
{
}
clone_trait_object!(<Fehler, NeuerFehler> KonvertiereFehler<Fehler, NeuerFehler>);

/// Hilfs-Trait zur [Clone]-baren Anzeige eines Fehlers.
trait AnzeigeFehler<Fehler>: Fn(&Fehler) -> String + DynClone {}
impl<Fehler, F: Fn(&Fehler) -> String + DynClone> AnzeigeFehler<Fehler> for F {}
clone_trait_object!(<Fehler> AnzeigeFehler<Fehler>);

impl<'t, T, Fehler> Argumente<'t, T, Fehler> {
    /// Konvertiere den Fehler mit der spezifizierten Funktion.
    ///
    /// ## English synonym
    /// [`convert_error`](Self::convert_error)
    #[inline]
    pub fn konvertiere_fehler<NeuerFehler>(
        self,
        mapper: impl 't + Fn(Fehler) -> NeuerFehler + Clone,
        anzeige_neuer_fehler: impl 't + Fn(&NeuerFehler) -> String + Clone,
    ) -> Argumente<'t, T, NeuerFehler> {
        todo!()
    }

    /// [`konvertiere_fehler`](Self::konvertiere_fehler) mit [`From::from`].
    ///
    /// ## English synonym
    /// [`error_from`](Self::error_from)
    #[inline]
    pub fn fehler_from<NeuerFehler: From<Fehler>>(
        self,
        anzeige_neuer_fehler: impl 't + Fn(&NeuerFehler) -> String + Clone,
    ) -> Argumente<'t, T, NeuerFehler> {
        self.konvertiere_fehler(NeuerFehler::from, anzeige_neuer_fehler)
    }
}

impl<'t, T> Argumente<'t, T, Void> {
    /// [`konvertiere_fehler`](Self::konvertiere_fehler) mit [`void::unreachable`].
    ///
    /// ## English synonym
    /// [`error_from_void`](Self::error_from_void)
    #[inline]
    pub fn fehler_from_void<NeuerFehler>(
        self,
        anzeige_neuer_fehler: impl 't + Fn(&NeuerFehler) -> String + Clone,
    ) -> Argumente<'t, T, NeuerFehler> {
        self.konvertiere_fehler(|void| void::unreachable(void), anzeige_neuer_fehler)
    }
}

impl<'t, T, Fehler> Arguments<'t, T, Fehler> {
    /// Convert the error with with given function.
    ///
    /// ## Deutsches Synonym
    /// [`konvertiere_fehler`](Self::konvertiere_fehler)
    #[inline]
    pub fn convert_error<NewError>(
        self,
        mapper: impl 't + Fn(Fehler) -> NewError + Clone,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> Arguments<'t, T, NewError> {
        self.konvertiere_fehler(mapper, display_new_error)
    }

    /// [`convert_error`](Self::convert_error) with [`From::from`].
    ///
    /// ## Deutsches Synonym
    /// [`fehler_from`](Self::fehler_from)
    #[inline]
    pub fn error_from<NewError: From<Fehler>>(
        self,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> Argumente<'t, T, NewError> {
        self.fehler_from(display_new_error)
    }
}

impl<'t, T> Arguments<'t, T, Void> {
    /// [`convert_error`](Self::convert_error) with [`void::unreachable`].
    ///
    /// ## Deutsches Synonym
    /// [`fehler_from_void`](Self::fehler_from_void)
    #[inline]
    pub fn error_from_void<NewError>(
        self,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> Arguments<'t, T, NewError> {
        self.fehler_from_void(display_new_error)
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
    ) -> NonEmpty<hilfe::Alternativen> {
        match self {
            Argumente::EinzelArgument(arg) => {
                nonempty![hilfe::Alternativen::EinzelArgument({
                    let string_arg = arg.als_string_wert();
                    variante.erzeuge_hilfe_text(string_arg, meta_standard, meta_erlaubte_werte)
                })]
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

impl<'t, T: Debug, Fehler: Debug> Argumente<'t, T, Fehler> {
    /// Füge eine [`FrühesBeenden`]-Flag hinzu, wodurch die Programm-Version anzeigt wird.
    ///
    /// ## English synonym
    /// [`with_version_early_exit`](Self::with_version_early_exit)
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

    /// Variante von [`mit_version_frühes_beenden`](Self::mit_version_frühes_beenden),
    /// basierend auf einer [`Sprache`].
    ///
    /// ## English synonym
    /// [`with_version_early_exit_with_language`](Self::with_version_early_exit_with_language)
    #[inline]
    pub fn mit_version_frühes_beenden_mit_sprache(
        self,
        programm_name: &str,
        programm_version: &str,
        sprache: Sprache,
    ) -> Self {
        let eigene_beschreibung = Beschreibung::neu_mit_sprache(
            sprache.version_lang,
            sprache.version_kurz,
            Some(sprache.version_beschreibung),
            None,
            sprache,
        );
        self.mit_version_frühes_beenden(eigene_beschreibung, programm_name, programm_version)
    }

    /// Füge eine [`FrühesBeenden`]-Flag hinzu, wodurch der Hilfe-Text für alle Argumente anzeigt wird.
    ///
    /// ### Panics
    /// Wenn die Syntax-Beschreibung (inklusive normalem + Alternativen-Präfix) für ein Argument
    /// länger als [`usize::MAX`] ist.
    ///
    /// ## English synonym
    /// [`with_help_early_exit`](Self::with_help_early_exit)
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
    /// ## English synonym
    /// [`with_help_early_exit_with_language`](Self::with_help_early_exit_with_language)
    #[inline]
    pub fn mit_hilfe_frühes_beenden_mit_sprache(
        self,
        variante: &dyn ErzeugeHilfeText,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: Option<&str>,
        sprache: Sprache,
    ) -> Self {
        let eigene_beschreibung = Beschreibung::neu_mit_sprache(
            sprache.hilfe_lang,
            sprache.hilfe_kurz,
            Some(sprache.hilfe_beschreibung),
            None,
            sprache,
        );
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
    /// ## English synonym
    /// [`with_help_and_version_early_exit`](Self::with_help_and_version_early_exit)
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
    /// ## English synonym
    /// [`with_help_and_version_early_exit_with_language`](Self::with_help_and_version_early_exit_with_language)
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn mit_hilfe_und_version_frühes_beenden_mit_sprache(
        self,
        variante: &dyn ErzeugeHilfeText,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: &str,
        sprache: Sprache,
    ) -> Self {
        let version_beschreibung = Beschreibung::neu_mit_sprache(
            sprache.version_lang,
            sprache.version_kurz,
            Some(sprache.version_beschreibung),
            None,
            sprache,
        );
        let hilfe_beschreibung = Beschreibung::neu_mit_sprache(
            sprache.hilfe_lang,
            sprache.hilfe_kurz,
            Some(sprache.hilfe_beschreibung),
            None,
            sprache,
        );
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

impl<'t, T: Debug, Error: Debug> Arguments<'t, T, Error> {
    /// Add an [`EarlyExit`](crate::argumente::frühes_beenden::EarlyExit`)-flag, showing the program version.
    ///
    /// ## Deutsches Synonym
    /// [`mit_version_frühes_beenden`](Self::mit_version_frühes_beenden)
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn with_version_early_exit(
        self,
        arg_description: Beschreibung<'t, Void>,
        program_name: &str,
        program_version: &str,
    ) -> Self {
        self.mit_version_frühes_beenden(arg_description, program_name, program_version)
    }

    /// Variant of [`mit_version_frühes_beenden`](Self::mit_version_frühes_beenden),
    /// based on a [`Language`](crate::sprache::Language).
    ///
    /// ## Deutsches Synonym
    /// [`mit_version_frühes_beenden_mit_sprache`](Self::mit_version_frühes_beenden_mit_sprache)
    #[inline]
    pub fn with_version_early_exit_with_language(
        self,
        program_name: &str,
        program_version: &str,
        language: Language,
    ) -> Self {
        self.mit_version_frühes_beenden_mit_sprache(program_name, program_version, language)
    }
    /// Add an [`EarlyExit`](crate::argumente::frühes_beenden::EarlyExit`)-Flag, showing the help text for all arguments.
    ///
    /// ### Panics
    /// If the syntax-description (including normal + alternativ prefixes) for an argument exceeds [`usize::MAX`].
    ///
    /// ## Deutsches Synonym
    /// [`mit_hilfe_frühes_beenden`](Self::mit_hilfe_frühes_beenden)
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn with_help_early_exit(
        self,
        variant: &dyn ErzeugeHilfeText,
        arg_description: Beschreibung<'t, Void>,
        program_name: &str,
        program_description: Option<&str>,
        program_version: Option<&str>,
        meta_standard: &str,
        meta_possible_values: &str,
        meta_options: &str,
        meta_syntax_prefix: &str,
        meta_syntax_padding: char,
        meta_alternative_prefix: &str,
        meta_alternative_separator: char,
    ) -> Self {
        self.mit_hilfe_frühes_beenden(
            variant,
            arg_description,
            program_name,
            program_description,
            program_version,
            meta_standard,
            meta_possible_values,
            meta_options,
            meta_syntax_prefix,
            meta_syntax_padding,
            meta_alternative_prefix,
            meta_alternative_separator,
        )
    }
    /// Add [`EarlyExit`](crate::argumente::frühes_beenden::EarlyExit`)-Flags, showing the program version,
    /// or the help text for all arguments.
    ///
    /// ### Panics
    /// If the syntax-description (including normal + alternativ prefixes) for an argument exceeds [`usize::MAX`].
    ///
    /// ## Deutsches Synonym
    /// [`mit_hilfe_und_version_frühes_beenden`](Self::mit_hilfe_und_version_frühes_beenden)
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn with_help_and_version_early_exit(
        self,
        variant: &dyn ErzeugeHilfeText,
        version_description: Description<'t, Void>,
        help_description: Description<'t, Void>,
        program_name: &str,
        program_description: Option<&str>,
        program_version: &str,
        meta_standard: &str,
        meta_possible_values: &str,
        meta_options: &str,
        meta_syntax_prefix: &str,
        meta_syntax_padding: char,
        meta_alternative_prefix: &str,
        meta_alternative_separator: char,
    ) -> Self {
        self.mit_hilfe_und_version_frühes_beenden(
            variant,
            version_description,
            help_description,
            program_name,
            program_description,
            program_version,
            meta_standard,
            meta_possible_values,
            meta_options,
            meta_syntax_prefix,
            meta_syntax_padding,
            meta_alternative_prefix,
            meta_alternative_separator,
        )
    }

    /// Variant of [`with_help_early_exit`](Argumente::with_help_early_exit)
    /// based on a [`Language`](crate::sprache::Language).
    ///
    /// ## Deutsches Synonym
    /// [`mit_hilfe_frühes_beenden_mit_sprache`](Argumente::mit_hilfe_frühes_beenden_mit_sprache).
    #[inline]
    pub fn with_help_early_exit_with_language(
        self,
        variant: &dyn ErzeugeHilfeText,
        program_name: &str,
        program_beschreibung: Option<&str>,
        program_version: Option<&str>,
        language: Language,
    ) -> Self {
        self.mit_hilfe_frühes_beenden_mit_sprache(
            variant,
            program_name,
            program_beschreibung,
            program_version,
            language,
        )
    }

    /// Variant of [`with_help_early_exit`](Argumente::with_help_early_exit)
    /// and [`with_version_early_exit`](Argumente::with_version_early_exit),
    /// based on a [`Language`](crate::sprache::Language).
    ///
    /// ## Deutsches Synonym
    /// [`mit_hilfe_und_version_frühes_beenden_mit_sprache`](Argumente::mit_hilfe_und_version_frühes_beenden_mit_sprache).
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn with_help_and_version_early_exit_with_language(
        self,
        variant: &dyn ErzeugeHilfeText,
        program_name: &str,
        program_description: Option<&str>,
        program_version: &str,
        language: Language,
    ) -> Self {
        self.mit_hilfe_und_version_frühes_beenden_mit_sprache(
            variant,
            program_name,
            program_description,
            program_version,
            language,
        )
    }
}

/// Berechne die maximale Breite für die Syntax eines Argumentes.
///
/// Hilfsfunktion für [`Argumente::mit_hilfe_frühes_beenden`]
///
/// ## Panics
/// Programmierfehler, wenn `NonEmpty::iter().map(...)` kein Element hat.
fn max_syntax_breite(hilfen: &NonEmpty<hilfe::Alternativen>, alternative_präfix: &str) -> usize {
    hilfen
        .iter()
        .filter_map(|arg| match arg {
            hilfe::Alternativen::EinzelArgument(arg) => Some(arg.syntax.len()),
            hilfe::Alternativen::Alternativen(alternativen) => {
                #[allow(clippy::arithmetic_side_effects)]
                let breite =
                    alternative_präfix.len() + max_syntax_breite(alternativen, alternative_präfix);
                Some(breite)
            },
            hilfe::Alternativen::Leer => None,
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
    eintrag: &hilfe::Alternativen,
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
        hilfe::Alternativen::Leer => {},
    }
}
