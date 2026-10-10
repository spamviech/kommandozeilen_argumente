//! Deutsche Spiegeltypen für Kommandozeilen-Argumente.

use std::{
    borrow::Cow,
    ffi::OsString,
    fmt::{self, Debug, Display},
    num::NonZeroI32,
};

use itertools::Itertools as _;
use nonempty::{NonEmpty, nonempty};
use void::Void;

pub use crate::arguments::ParseResult;
/// Compatibility re-exports for established German module paths.
pub use crate::arguments::{combine, early_exit, flag, help, single_argument, value};
use crate::{
    arguments::{
        Arguments,
        combine::Combine,
        early_exit::FrühesBeenden,
        flag::Flag,
        help::CreateHelpText,
        single_argument::EinzelArgument,
        value::{Value, Wert},
    },
    description::{ArgumentInput, Beschreibung},
    dyn_to_owned,
    language::{Language, Sprache},
    outcome::{Ergebnis, Error, Fehler},
};

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

impl<'t, T, F> Argumente<'t, T, F>
where
    T: Clone,
    F: Clone,
{
    /// Parse merged short form arguments.
    ///
    /// Rules to allow merging of short names:
    ///
    /// - All short names in the same string share the same (short) prefix.
    /// - Only short names consisting of a single [grapheme](https://docs.rs/unicode-segmentation/1.8.0/unicode_segmentation/trait.UnicodeSegmentation.html#tymethod.graphemes)
    ///   participate.
    /// - At most one value argument per block. It must be the last argument name in the string,
    ///   optionally followed by \[a value-infix and\] the value sub-string.
    /// - Merging of short names must be allowed for this particular argument.
    ///
    /// ## Panics
    ///
    /// On programmer error only.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> NonEmpty<ParseResult<'_, 't, T, F>> {
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
        let (result, remaining) = Arguments::from(self).parse_from_env();
        (result.into(), remaining)
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
        let (result, remaining) = Arguments::from(self).parse_from_env_with_early_exit();
        (result.map_err(|errors| errors.map(Into::into)), remaining)
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
        let (result, remaining) = Arguments::from(self).parse_from_env_with_early_exit();
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
        let (result, remaining) = Arguments::from(self).parse_with_early_exit(args);
        (result.map_err(|errors| errors.map(Into::into)), remaining)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
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
        Arguments::from(self).parse_complete(
            args,
            fehler_code,
            fehlende_flag,
            fehlender_wert,
            parse_fehler,
            invalider_string,
            argument_nicht_verwendet,
        )
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
        Arguments::from(self).parse_complete(
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
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
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
        Arguments::from(self).parse_complete_with_language(args, error_code, language)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
    ///
    /// [`parse_vollständig_mit_sprache`](Self::parse_vollständig_mit_sprache) mit
    /// [`Sprache::DEUTSCH`].
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
        Arguments::from(self).parse_complete_with_language(args, fehler_code, Language::GERMAN)
    }

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// [`parse_complete_with_language`](Self::parse_complete_with_language) with
    /// [`Language::ENGLISH`].
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
        Arguments::from(self).parse_with_error_message(args, error_code)
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
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
        Arguments::from(self).parse_complete_from_env(
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
        Arguments::from(self).parse_complete_from_env(
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
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
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
        Arguments::from(self).parse_complete_with_language_from_env(fehler_code, sprache.into())
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
        Arguments::from(self).parse_complete_with_language_from_env(error_code, language)
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
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
        Arguments::from(self).parse_complete_with_language_from_env(fehler_code, Language::GERMAN)
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
        Arguments::from(self).parse_with_error_message_from_env(error_code)
    }
}

impl<'t, T, Fehler> Argumente<'t, T, Fehler> {
    /// Konvertiere den Fehler mit der spezifizierten Funktion.
    ///
    /// ## English synonym
    /// [`Arguments::convert_error`]
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
    /// [`Arguments::error_from`]
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
    /// [`Arguments::error_from_void`]
    #[inline]
    pub fn fehler_from_void<NeuerFehler>(
        self,
        anzeige_neuer_fehler: impl 't + Fn(&NeuerFehler) -> String + Clone,
    ) -> Argumente<'t, T, NeuerFehler> {
        self.konvertiere_fehler(|void| void::unreachable(void), anzeige_neuer_fehler)
    }
}
impl<T, Fehler> Argumente<'_, T, Fehler> {
    /// Erzeugt die Syntax und den zugehörigen Hilfetext dieses Arguments.
    #[inline]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn erzeuge_hilfe_text(
        &self,
        variante: &dyn CreateHelpText,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> NonEmpty<help::Alternativen> {
        match self {
            Self::EinzelArgument(argument) => {
                nonempty![help::Alternativen::EinzelArgument(variante.create_help_text(
                    argument.als_string_wert(),
                    meta_standard,
                    meta_erlaubte_werte,
                ))]
            },
            Self::Kombiniere(kombiniere) => {
                kombiniere.create_help_text(variante, meta_standard, meta_erlaubte_werte)
            },
            Self::Alternativen(alternativen) => {
                // TODO use alternativen.as_ref().flat_map(...), coming in nonempty > 0.10.0
                NonEmpty::collect(alternativen.iter().map(|argument| {
                    help::Alternativen::Alternativen(Box::new(argument.erzeuge_hilfe_text(
                        variante,
                        meta_standard,
                        meta_erlaubte_werte,
                    )))
                }))
                .expect("NonEmpty::map(...) has at least one argument!")
            },
        }
    }
}
impl<'t, T: Debug, Fehler: Debug> Argumente<'t, T, Fehler> {
    /// Füge eine [`FrühesBeenden`]-Flag hinzu, wodurch die Programm-Version anzeigt wird.
    ///
    /// ## English synonym
    /// [`Arguments::with_version_early_exit`]
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn mit_version_frühes_beenden(
        self,
        eigene_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_version: &str,
    ) -> Self {
        Arguments::from(self)
            .with_version_early_exit(eigene_beschreibung.into(), programm_name, programm_version)
            .into()
    }

    /// Variante von [`mit_version_frühes_beenden`](Self::mit_version_frühes_beenden),
    /// basierend auf einer [`Sprache`].
    ///
    /// ## English synonym
    /// [`Arguments::with_version_early_exit_with_language`]
    #[inline]
    pub fn mit_version_frühes_beenden_mit_sprache(
        self,
        programm_name: &str,
        programm_version: &str,
        sprache: Sprache,
    ) -> Self {
        Arguments::from(self)
            .with_version_early_exit_with_language(programm_name, programm_version, sprache.into())
            .into()
    }

    /// Füge eine [`FrühesBeenden`]-Flag hinzu, wodurch der Hilfe-Text für alle Argumente anzeigt
    /// wird.
    ///
    /// ### Panics
    /// Wenn die Syntax-Beschreibung (inklusive normalem + Alternativen-Präfix) für ein Argument
    /// länger als [`usize::MAX`] ist.
    ///
    /// ## English synonym
    /// [`Arguments::with_help_early_exit`]
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
        Arguments::from(self)
            .with_help_early_exit(
                variante,
                eigene_beschreibung.into(),
                programm_name,
                programm_beschreibung,
                programm_version,
                meta_standard,
                meta_erlaubte_werte,
                meta_optionen,
                meta_syntax_präfix,
                meta_syntax_padding,
                meta_alternative_präfix,
                meta_alternative_trennzeichen,
            )
            .into()
    }

    /// Variante von [`mit_hilfe_frühes_beenden`](Self::mit_hilfe_frühes_beenden),
    /// basierend auf einer [`Sprache`].
    ///
    /// ## English synonym
    /// [`Arguments::with_help_early_exit_with_language`]
    #[inline]
    pub fn mit_hilfe_frühes_beenden_mit_sprache(
        self,
        variante: &dyn CreateHelpText,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: Option<&str>,
        sprache: Sprache,
    ) -> Self {
        Arguments::from(self)
            .with_help_early_exit_with_language(
                variante,
                programm_name,
                programm_beschreibung,
                programm_version,
                sprache.into(),
            )
            .into()
    }

    /// Füge [`FrühesBeenden`]-Flags hinzu, wodurch die Programm-Version,
    /// bzw. der Hilfe-Text für alle Argumente anzeigt wird.
    ///
    /// ### Panics
    /// Wenn die Syntax-Beschreibung (inklusive normalem + Alternativen-Präfix) für ein Argument
    /// länger als [`usize::MAX`] ist.
    ///
    /// ## English synonym
    /// [`Arguments::with_help_and_version_early_exit`]
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
        Arguments::from(self)
            .with_help_and_version_early_exit(
                variante,
                version_beschreibung.into(),
                hilfe_beschreibung.into(),
                programm_name,
                programm_beschreibung,
                programm_version,
                meta_standard,
                meta_erlaubte_werte,
                meta_optionen,
                meta_syntax_präfix,
                meta_syntax_padding,
                meta_alternative_präfix,
                meta_alternative_trennzeichen,
            )
            .into()
    }

    /// Variante von
    /// [`mit_hilfe_und_version_frühes_beenden`](Argumente::mit_hilfe_und_version_frühes_beenden).
    ///
    /// ## English synonym
    /// [`Arguments::with_help_and_version_early_exit_with_language`]
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
        Arguments::from(self)
            .with_help_and_version_early_exit_with_language(
                variante,
                programm_name,
                programm_beschreibung,
                programm_version,
                sprache.into(),
            )
            .into()
    }
}
