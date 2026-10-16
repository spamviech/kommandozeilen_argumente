//! Parsing convenience methods for [`Argumente`].

use std::{ffi::OsString, fmt::Display, num::NonZeroI32};

use itertools::Itertools as _;
use nonempty::NonEmpty;

use crate::{
    arguments::{Arguments, argumente::Argumente, parsing::ParsedMergedShortForms},
    description::ArgumentInput,
    language::{Language, Sprache},
    outcome::{Ergebnis, Error, Fehler},
};

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
    ) -> NonEmpty<ParsedMergedShortForms<'t>> {
        use crate::arguments::argumente::Argumente::{Alternativen, EinzelArgument, Kombiniere};
        match self {
            EinzelArgument(einzelargument) => {
                NonEmpty::singleton(einzelargument.parse_merged_short_forms(args))
            },
            Kombiniere(kombiniere) => kombiniere.parse_merged_short_forms(Box::new(args)),
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
    #[expect(
        clippy::too_many_arguments,
        reason = "This public localization API accepts each independently configurable message \
                  explicitly."
    )]
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
    #[expect(
        clippy::too_many_arguments,
        reason = "This public localization API accepts each independently configurable message \
                  explicitly."
    )]
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
