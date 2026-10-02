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
use itertools::Itertools as _;
use nonempty::{nonempty, NonEmpty};
use void::Void;

use crate::{
    arguments::{
        combine::Combine,
        early_exit::FrühesBeenden,
        flag::Flag,
        help::{CreateHelpText, Hilfe},
        single_argument::{EinzelArgument, SingleArgument},
        value::{Value, Wert},
    },
    description::{ArgumentInput, Beschreibung},
    dyn_to_owned,
    language::{Language, Sprache},
    outcome::{Ergebnis, Error, Fehler},
    Description,
};

pub mod combine;
pub mod early_exit;
pub mod flag;
pub mod help;
pub mod single_argument;
pub mod value;

#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use self::value::EnumArgument;

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
    /// Die Kombination mehrerer Argumente, kodiert über den [`Combine`]-Trait.
    ///
    /// ## English
    /// The combination of multiple arguments, encoded via the [`Combine`]-trait.
    Kombiniere(Box<dyn 't + Combine<'t, T, Fehler>>),
    /// Alternative Kommandozeilen-Argumente. Beim parsen wird das erste [`Ergebnis`] verwendet,
    /// dass kein [`Ergebnis::Fehler`] ist.
    ///
    /// ## English
    /// Alternative command line arguments. Parsing takes the first non-[`Error`](Ergebnis::Fehler)
    /// [`Result`](crate::Result).
    Alternativen(Box<NonEmpty<Self>>),
}

/// Helper um [`Combine::debug_fmt`] mit [`fmt::Formatter::debug_tuple`] zu verwenden.
struct KombiniereDebug<'s, 't, T, Fehler>(&'s dyn Combine<'t, T, Fehler>);

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

/// Configuration of command-line arguments.
///
/// ## Deutsch
/// Konfiguration der Kommandozeilen-Argumente.
#[allow(clippy::large_enum_variant, clippy::module_name_repetitions)]
#[must_use]
pub enum Arguments<'t, T, Error> {
    /// A single argument.
    Single(SingleArgument<'t, T, Error>),
    /// A combination of multiple arguments.
    Combined(Box<dyn 't + Combine<'t, T, Error>>),
    /// Alternative command-line arguments.
    Alternatives(Box<NonEmpty<Self>>),
}

impl<T: Debug, Error: Debug> Debug for Arguments<'_, T, Error> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(argument) => formatter.debug_tuple("Single").field(argument).finish(),
            Self::Combined(combine) => {
                formatter.debug_tuple("Combined").field(&KombiniereDebug(&**combine)).finish()
            },
            Self::Alternatives(alternatives) => {
                formatter.debug_tuple("Alternatives").field(alternatives).finish()
            },
        }
    }
}

impl<'t, T, Error> From<Arguments<'t, T, Error>> for Argumente<'t, T, Error> {
    fn from(arguments: Arguments<'t, T, Error>) -> Self {
        match arguments {
            Arguments::Single(argument) => Self::EinzelArgument(argument.into()),
            Arguments::Combined(combine) => Self::Kombiniere(combine),
            Arguments::Alternatives(alternatives) => {
                Self::Alternativen(Box::new(alternatives.map(Into::into)))
            },
        }
    }
}

impl<'t, T, Fehler> From<Argumente<'t, T, Fehler>> for Arguments<'t, T, Fehler> {
    fn from(argumente: Argumente<'t, T, Fehler>) -> Self {
        match argumente {
            Argumente::EinzelArgument(argument) => Self::Single(argument.into()),
            Argumente::Kombiniere(combine) => Self::Combined(combine),
            Argumente::Alternativen(alternatives) => {
                Self::Alternatives(Box::new(alternatives.map(Into::into)))
            },
        }
    }
}

impl<'t, T, Error> From<SingleArgument<'t, T, Error>> for Arguments<'t, T, Error> {
    fn from(argument: SingleArgument<'t, T, Error>) -> Self {
        Self::Single(argument)
    }
}

impl<'t, T, Error> From<Flag<'t, T>> for Arguments<'t, T, Error> {
    fn from(flag: Flag<'t, T>) -> Self {
        Self::Single(flag.into())
    }
}

impl<'t, T, Error> From<Value<'t, T, Error>> for Arguments<'t, T, Error> {
    fn from(value: Value<'t, T, Error>) -> Self {
        Self::Single(value.into())
    }
}

impl<'t, T, Fehler> From<EinzelArgument<'t, T, Fehler>> for Arguments<'t, T, Fehler> {
    fn from(argument: EinzelArgument<'t, T, Fehler>) -> Self {
        Self::Single(argument.into())
    }
}

impl<'t, T, Fehler> From<Wert<'t, T, Fehler>> for Arguments<'t, T, Fehler> {
    fn from(value: Wert<'t, T, Fehler>) -> Self {
        Self::Single(Value::from(value).into())
    }
}

impl<T, Error> From<NonEmpty<Self>> for Arguments<'_, T, Error> {
    fn from(alternatives: NonEmpty<Self>) -> Self {
        Self::Alternatives(Box::new(alternatives))
    }
}

impl<T, Error> From<Box<NonEmpty<Self>>> for Arguments<'_, T, Error> {
    fn from(alternatives: Box<NonEmpty<Self>>) -> Self {
        Self::Alternatives(alternatives)
    }
}

impl<'t, T, Error> Arguments<'t, T, Error> {
    /// Creates a single-argument variant with suitable type parameters.
    pub fn single_argument(argument: SingleArgument<'t, T, Error>) -> Self {
        Self::Single(argument)
    }
    /// Creates a combined-arguments variant with suitable type parameters.
    pub fn combine(combine: impl 't + Combine<'t, T, Error>) -> Self {
        Self::Combined(Box::new(combine))
    }
    /// Creates an alternatives variant with suitable type parameters.
    pub fn alternatives(alternatives: NonEmpty<Self>) -> Self {
        Self::Alternatives(Box::new(alternatives))
    }
    /// Creates a boxed alternatives variant with suitable type parameters.
    pub fn alternatives_boxed(alternatives: Box<NonEmpty<Self>>) -> Self {
        Self::Alternatives(alternatives)
    }
}

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
        let anzeige_boxed: Box<dyn 't + dyn_to_owned::Show<'t, T>> = Box::new(anzeige);
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

impl<'t, T, Error> From<Value<'t, T, Error>> for Argumente<'t, T, Error> {
    #[inline]
    fn from(value: Value<'t, T, Error>) -> Self {
        Self::from(Wert::from(value))
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
        Arguments::single_argument(einzel_argument.into()).into()
    }
}

impl<'t, T, Fehler> Argumente<'t, T, Fehler> {
    /// Erzeuge eine [`Argumente::Kombiniere`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English synonym
    /// [`combine`](Self::combine)
    #[inline]
    pub fn kombiniere(kombiniere: impl 't + Combine<'t, T, Fehler>) -> Self {
        Arguments::combine(kombiniere).into()
    }

    /// Create a [`Argumente::Kombiniere`]-variant with sensible type parameters.
    ///
    /// ## Deutsches Synonym
    /// [`kombiniere`](Self::combine)
    #[inline]
    pub fn combine(combine: impl 't + Combine<'t, T, Fehler>) -> Self {
        Self::kombiniere(combine)
    }

    /// Erzeuge eine [`Argumente::Alternativen`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English synonym
    /// [`alternatives`](Self::alternatives)
    #[inline]
    pub fn alternativen(alternativen: NonEmpty<Self>) -> Self {
        Arguments::alternatives(alternativen.map(Into::into)).into()
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
        Arguments::alternatives_boxed(Box::new((*alternativen).map(Into::into))).into()
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
#[derive(Debug, Clone)]
pub struct ParsedEarlyExit<'s> {
    /// TODO
    pub name: Cow<'s, str>,
    /// TODO
    pub message: Cow<'s, str>,
    /// TODO
    pub input: Cow<'s, str>,
}
/// TODO
#[derive(Debug, Clone)]
pub struct ParsedShortFlag<'s> {
    /// TODO
    pub name: Cow<'s, str>,
    /// TODO
    pub input: Cow<'s, str>,
}
/// TODO
#[derive(Debug, Clone)]
pub struct ParsedValueName<'s> {
    /// TODO
    pub name: Cow<'s, str>,
}
/// TODO
#[derive(Debug, Clone)]
pub struct ParsedValue<'s> {
    /// TODO
    pub value: Cow<'s, str>,
    /// TODO
    pub input: Cow<'s, str>,
}

/// TODO
#[derive(Debug, Clone)]
pub struct ParseMergedShortFormsResult<'s, T, F> {
    /// Argument definition used to produce this parse result.
    pub definition: &'s Arguments<'s, T, F>,
    /// A vector of early\_exit arguments, containing name, message & original input.
    pub early_exits: Vec<ParsedEarlyExit<'s>>,
    /// A vector of flag-arguments with their name (all are true) & the original input.
    pub flags: Vec<ParsedShortFlag<'s>>,
    /// A map of value-arguments with name -> (value-string, original input).
    pub values: HashMap<ParsedValueName<'s>, ParsedValue<'s>>,
    /// Remaining arguments with the parsed merged short names and associated value-strings removed.
    pub remaining: Vec<Option<OsString>>,
}

impl<T, F> Arguments<'_, T, F>
where
    T: Clone,
    F: Clone,
{
    /// Parses merged short-form arguments.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> NonEmpty<ParseMergedShortFormsResult<'_, T, F>> {
        match self {
            Self::Single(argument) => NonEmpty::singleton(argument.parse_merged_short_forms(args)),
            Self::Combined(_combine) => todo!(),
            Self::Alternatives(alternatives) => {
                let args = args.collect_vec();
                NonEmpty::collect(
                    alternatives.iter().flat_map(|argument| {
                        argument.parse_merged_short_forms(args.iter().cloned())
                    }),
                )
                .expect("Iterator of NonEmpty<NonEmpty<_>>.")
            },
        }
    }
}

impl<T, F> Argumente<'_, T, F>
where
    T: Clone,
    F: Clone,
{
    /// Parse merged short form arguments.
    ///
    /// Rules to allow merging of short names:
    ///
    /// - All short names in the same string share the same (short) prefix.
    /// - Only short names consisting of a single [grapheme](https://docs.rs/unicode-segmentation/1.8.0/unicode_segmentation/trait.UnicodeSegmentation.html#tymethod.graphemes) participate.
    /// - At most one value argument per block.
    ///   It must be the last argument name in the string, optionally followed by \[a value-infix and\] the value sub-string.
    /// - Merging of short names must be allowed for this particular argument.
    ///
    /// ## Panics
    ///
    /// On programmer error only.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> NonEmpty<ParseMergedShortFormsResult<'_, T, F>> {
        use Argumente::{Alternativen, EinzelArgument, Kombiniere};
        match self {
            EinzelArgument(einzelargument) => {
                NonEmpty::singleton(einzelargument.parse_merged_short_forms(args))
            },
            Kombiniere(kombiniere) => todo!(),
            Alternativen(non_empty) => {
                let args = args.collect_vec();

                let nested =
                    non_empty.iter().map(|arg| arg.parse_merged_short_forms(args.iter().cloned()));
                NonEmpty::collect(nested.flatten()).expect("Iterator of NonEmpty<NonEmpty<_>>.")
            },
        }
    }
}

impl<'t, T, F> Arguments<'t, T, F> {
    /// Parses the supplied arguments and produces the corresponding value.
    #[inline]
    pub fn parse(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> (crate::outcome::Result<'t, T, F>, Vec<ArgumentInput>) {
        todo!("parse({:?})", args.collect::<Vec<_>>());
    }

    /// Parses arguments from the environment.
    #[inline]
    pub fn parse_from_env(self) -> (crate::outcome::Result<'t, T, F>, Vec<ArgumentInput>)
    where
        Self: 't,
        F: 't,
    {
        self.parse(env::args_os().skip(1))
    }

    /// Parses arguments and exits successfully after writing an early-exit message.
    #[inline]
    pub fn parse_with_early_exit(
        self,
        args: impl Iterator<Item = OsString>,
    ) -> (Result<T, NonEmpty<Error<'t, F>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        F: 't,
    {
        let (result, remaining) = self.parse(args);
        let result = match result {
            crate::outcome::Result::Value(value) => Ok(value),
            crate::outcome::Result::EarlyExit(messages) => {
                #[allow(clippy::print_stdout)]
                for message in messages {
                    println!("{message}");
                }
                process::exit(0);
            },
            crate::outcome::Result::Error(errors) => Err(errors),
        };
        (result, remaining)
    }

    /// Parses environment arguments and exits successfully after writing an early-exit message.
    #[inline]
    pub fn parse_from_env_with_early_exit(
        self,
    ) -> (Result<T, NonEmpty<Error<'t, F>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        F: 't,
    {
        self.parse_with_early_exit(env::args_os().skip(1))
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
        let (result, remaining) = Arguments::from(self).parse(args);
        (result.into(), remaining)
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
        let (result, remaining) = Arguments::from(self).parse_from_env();
        (result.into(), remaining)
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
        let (result, remaining) = self.parse_aus_env_mit_frühen_beenden();
        (result.map_err(|errors| errors.map(Into::into)), remaining)
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
        let (result, remaining) = Arguments::from(self).parse_with_early_exit(args);
        (result.map_err(|errors| errors.map(Into::into)), remaining)
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
        let (result, remaining) = self.parse_mit_frühen_beenden(args);
        (result.map_err(|errors| errors.map(Into::into)), remaining)
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
        self.parse_vollständig_mit_sprache(args, error_code, language.into())
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
        self.parse_vollständig_mit_sprache_aus_env(error_code, language.into())
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

/// Helper trait for cloneable error conversion functions.
trait ConvertError<Error, NewError>: Fn(Error) -> NewError + DynClone {}
impl<Error, NewError, F: Fn(Error) -> NewError + DynClone> ConvertError<Error, NewError> for F {}
clone_trait_object!(<Error, NewError> ConvertError<Error, NewError>);

/// Helper trait for cloneable error display functions.
trait DisplayError<Error>: Fn(&Error) -> String + DynClone {}
impl<Error, F: Fn(&Error) -> String + DynClone> DisplayError<Error> for F {}
clone_trait_object!(<Error> DisplayError<Error>);

impl<'t, T, Error> Arguments<'t, T, Error> {
    /// Converts errors with the supplied functions.
    #[inline]
    pub fn convert_error<NewError>(
        self,
        mapper: impl 't + Fn(Error) -> NewError + Clone,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> Arguments<'t, T, NewError> {
        todo!()
    }

    /// Converts errors using [`From::from`].
    #[inline]
    pub fn error_from<NewError: From<Error>>(
        self,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> Arguments<'t, T, NewError> {
        self.convert_error(NewError::from, display_new_error)
    }
}

impl<'t, T> Arguments<'t, T, Void> {
    /// Converts the infallible error type using [`void::unreachable`].
    #[inline]
    pub fn error_from_void<NewError>(
        self,
        display_new_error: impl 't + Fn(&NewError) -> String + Clone,
    ) -> Arguments<'t, T, NewError> {
        self.convert_error(|value| void::unreachable(value), display_new_error)
    }
}

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
        Arguments::from(self).convert_error(mapper, anzeige_neuer_fehler).into()
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

impl<T, Error> Arguments<'_, T, Error> {
    /// Creates syntax and help text for this argument.
    #[inline]
    pub fn create_help_text(
        &self,
        variant: &dyn CreateHelpText,
        meta_default: &str,
        meta_possible_values: &str,
    ) -> NonEmpty<help::Alternatives> {
        match self {
            Self::Single(argument) => nonempty![help::Alternatives::Single(
                variant
                    .create_help_text(
                        argument.as_string_value().into(),
                        meta_default,
                        meta_possible_values
                    )
                    .into()
            )],
            Self::Combined(combine) => combine
                .create_help_text(variant, meta_default, meta_possible_values)
                .map(Into::into),
            Self::Alternatives(alternatives) => {
                NonEmpty::collect(alternatives.iter().map(|argument| {
                    help::Alternatives::Alternatives(Box::new(argument.create_help_text(
                        variant,
                        meta_default,
                        meta_possible_values,
                    )))
                }))
                .expect("NonEmpty::map(...) has at least one argument!")
            },
        }
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
        variante: &dyn CreateHelpText,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> NonEmpty<help::Alternativen> {
        match self {
            Argumente::EinzelArgument(arg) => {
                nonempty![help::Alternativen::EinzelArgument({
                    let string_arg = arg.als_string_wert();
                    variante.create_help_text(string_arg, meta_standard, meta_erlaubte_werte)
                })]
            },
            Argumente::Kombiniere(kombiniere) => {
                kombiniere.create_help_text(variante, meta_standard, meta_erlaubte_werte)
            },
            Argumente::Alternativen(alternativen) => {
                // TODO use alternativen.as_ref().flat_map(...), coming in nonempty > 0.10.0
                NonEmpty::collect(alternativen.iter().map(|arg| {
                    help::Alternativen::Alternativen(Box::new(arg.erzeuge_hilfe_text(
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
        variante: &dyn CreateHelpText,
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
        hilfen.push(help::Alternativen::EinzelArgument(frühes_beenden.erzeuge_hilfe_text()));
        let hilfen = hilfen;
        let max_syntax_width = max_syntax_width(&hilfen, meta_alternative_präfix);
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
            write_argument_or_alternatives(
                &mut hilfe_text,
                Cow::Borrowed(meta_syntax_präfix),
                #[allow(clippy::arithmetic_side_effects)]
                {
                    meta_syntax_präfix.len() + max_syntax_width + 1
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
        variante: &dyn CreateHelpText,
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
        variante: &dyn CreateHelpText,
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
        variante: &dyn CreateHelpText,
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
    /// Add an [`EarlyExit`](crate::argumente::early_exit::EarlyExit`)-flag, showing the program version.
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
        Argumente::from(self)
            .mit_version_frühes_beenden(arg_description.into(), program_name, program_version)
            .into()
    }

    /// Variant of [`mit_version_frühes_beenden`](Self::mit_version_frühes_beenden),
    /// based on a [`Language`](crate::language::Language).
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
        Argumente::from(self)
            .mit_version_frühes_beenden_mit_sprache(program_name, program_version, language.into())
            .into()
    }
    /// Add an [`EarlyExit`](crate::argumente::early_exit::EarlyExit`)-Flag, showing the help text for all arguments.
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
        variant: &dyn CreateHelpText,
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
        Argumente::from(self)
            .mit_hilfe_frühes_beenden(
                variant,
                arg_description.into(),
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
            .into()
    }
    /// Add [`EarlyExit`](crate::argumente::early_exit::EarlyExit`)-Flags, showing the program version,
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
        variant: &dyn CreateHelpText,
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
        Argumente::from(self)
            .mit_hilfe_und_version_frühes_beenden(
                variant,
                version_description.into(),
                help_description.into(),
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
            .into()
    }

    /// Variant of [`with_help_early_exit`](Argumente::with_help_early_exit)
    /// based on a [`Language`](crate::language::Language).
    ///
    /// ## Deutsches Synonym
    /// [`mit_hilfe_frühes_beenden_mit_sprache`](Argumente::mit_hilfe_frühes_beenden_mit_sprache).
    #[inline]
    pub fn with_help_early_exit_with_language(
        self,
        variant: &dyn CreateHelpText,
        program_name: &str,
        program_beschreibung: Option<&str>,
        program_version: Option<&str>,
        language: Language,
    ) -> Self {
        Argumente::from(self)
            .mit_hilfe_frühes_beenden_mit_sprache(
                variant,
                program_name,
                program_beschreibung,
                program_version,
                language.into(),
            )
            .into()
    }

    /// Variant of [`with_help_early_exit`](Argumente::with_help_early_exit)
    /// and [`with_version_early_exit`](Argumente::with_version_early_exit),
    /// based on a [`Language`](crate::language::Language).
    ///
    /// ## Deutsches Synonym
    /// [`mit_hilfe_und_version_frühes_beenden_mit_sprache`](Argumente::mit_hilfe_und_version_frühes_beenden_mit_sprache).
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn with_help_and_version_early_exit_with_language(
        self,
        variant: &dyn CreateHelpText,
        program_name: &str,
        program_description: Option<&str>,
        program_version: &str,
        language: Language,
    ) -> Self {
        Argumente::from(self)
            .mit_hilfe_und_version_frühes_beenden_mit_sprache(
                variant,
                program_name,
                program_description,
                program_version,
                language.into(),
            )
            .into()
    }
}

/// Calculates the maximum width of an argument syntax.
///
/// Helper for `Arguments::with_help_early_exit`.
fn max_syntax_width(helps: &NonEmpty<help::Alternativen>, alternative_prefix: &str) -> usize {
    helps
        .iter()
        .filter_map(|argument| match argument {
            help::Alternativen::EinzelArgument(argument) => Some(argument.syntax.len()),
            help::Alternativen::Alternativen(alternatives) => {
                #[allow(clippy::arithmetic_side_effects)]
                let width =
                    alternative_prefix.len() + max_syntax_width(alternatives, alternative_prefix);
                Some(width)
            },
            help::Alternativen::Leer => None,
        })
        .max()
        .expect("NonEmpty")
}

/// Writes help text for an entry or all of its alternatives.
///
/// # Panics
/// Panics if `max_syntax_width` is too small for an entry's prefix and syntax.
fn write_argument_or_alternatives(
    output: &mut String,
    current_prefix: Cow<'_, str>,
    max_syntax_width: usize,
    syntax_padding: char,
    entry: &help::Alternativen,
    alternative_prefix: &str,
    alternative_separator: char,
) {
    match entry {
        help::Alternativen::EinzelArgument(argument) => {
            let Hilfe { syntax, hilfe: help } = argument;
            output.push_str(&current_prefix);
            output.push_str(syntax);
            #[allow(clippy::arithmetic_side_effects)]
            let padding = max_syntax_width - current_prefix.len() - syntax.len();
            let mut buffer: [u8; 4] = [0; 4];
            let padding_string = syntax_padding.encode_utf8(&mut buffer).repeat(padding);
            output.push_str(&padding_string);
            if let Some(help) = help {
                output.push_str(help);
            }
            output.push('\n');
        },
        help::Alternativen::Alternativen(alternatives) => {
            #[allow(clippy::arithmetic_side_effects)]
            let separator_width = max_syntax_width - current_prefix.len();
            let mut buffer: [u8; 4] = [0; 4];
            let separator = format!(
                "{current_prefix}{}",
                alternative_separator.encode_utf8(&mut buffer).repeat(separator_width)
            );
            let mut next_prefix = current_prefix.into_owned();
            next_prefix.push_str(alternative_prefix);
            let mut first = true;
            for alternative in alternatives.iter() {
                if first {
                    first = false;
                } else {
                    output.push_str(&separator);
                    output.push('\n');
                }
                write_argument_or_alternatives(
                    output,
                    Cow::Borrowed(&next_prefix),
                    max_syntax_width,
                    syntax_padding,
                    alternative,
                    alternative_prefix,
                    alternative_separator,
                );
            }
        },
        help::Alternativen::Leer => {},
    }
}
