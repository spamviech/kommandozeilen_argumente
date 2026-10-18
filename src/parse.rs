//! Traits for types that can be parsed from command-line arguments.

mod argument;
mod option;

use std::{ffi::OsString, fmt::Display, num::NonZeroI32, result};

use nonempty::NonEmpty;

use crate::{
    arguments::Arguments,
    description::ArgumentInput,
    language::Language,
    outcome::{Error, Result},
};
#[cfg(feature = "german")]
use crate::{
    language::Sprache,
    outcome::{Ergebnis, Fehler},
};

#[rustfmt::skip]
#[cfg(feature = "derive")]
#[cfg_attr(all(doc, feature = "derive", not(doctest)), doc(cfg(feature = "derive")))]
pub use kommandozeilen_argumente_derive::Parse;
pub use argument::ParseArgument;

/// Result of [`Parse::parse_with_early_exit`] and similar methods.
pub type ParseEarlyExitResult<'t, T, E> = result::Result<T, NonEmpty<Error<'t, E>>>;

/// Result of [`Parse::parse_mit_frühen_beenden`] and similar methods.
#[cfg(feature = "german")]
pub type ParseFrühesBeendenResult<'t, T, E> = result::Result<T, NonEmpty<Fehler<'t, E>>>;

/// Allows parsing from command-line arguments based on a default configuration.
///
/// With the `derive` feature, an implementation can be generated automatically with
/// `#[derive(Parse)]`.
///
/// ## Deutsch
/// Erlaubt das Parsen von Kommandozeilen-Argumenten anhand einer Standardkonfiguration.
///
/// Mit dem `derive`-Feature kann eine Implementierung automatisch mit `#[derive(Parse)]` erzeugt
/// werden.
pub trait Parse: Sized {
    /// Possible parse error, the automatically created implementation uses [`String`].
    ///
    /// ## Deutsch
    /// Möglicher Parse-Fehler, die automatisch erzeugte Implementierung verwendet [`String`].
    type Error;

    /// Create a description, how command line arguments should be parsed.
    ///
    /// ## Deutsch
    /// Erzeuge eine Beschreibung, wie Kommandozeilen-Argumente geparst werden sollen.
    fn arguments<'t>() -> Arguments<'t, Self, Self::Error>;

    /// Parse the given command line arguments to create the requested type.
    ///
    /// ## Deutsch
    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    #[inline]
    fn parse<'t>(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<'t, Self, Self::Error>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        Self::arguments().parse(args)
    }

    /// Parse [`args_os`](std::env::args_os) and try to create the requested type.
    ///
    /// ## Deutsches Synonym
    /// `parse_aus_env`
    #[inline]
    fn parse_from_env<'t>() -> (Result<'t, Self, Self::Error>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        Self::arguments().parse_from_env()
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    ///
    /// ## English synonym
    /// [`parse_from_env`](Parse::parse_from_env)
    #[inline]
    #[cfg(feature = "german")]
    fn parse_aus_env<'t>() -> (Ergebnis<'t, Self, Self::Error>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        let (result, remaining) = Self::parse_from_env();
        (result.into(), remaining)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// `parse_mit_frühen_beenden`
    #[inline]
    #[expect(
        clippy::type_complexity,
        reason = "The public trait signature accurately exposes the parser result without an \
                  obscuring alias."
    )]
    fn parse_with_early_exit<'t>(
        args: impl Iterator<Item = OsString>,
    ) -> (result::Result<Self, NonEmpty<Error<'t, Self::Error>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        Self::arguments().parse_with_early_exit(args)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [`parse_with_early_exit`](Parse::parse_with_early_exit)
    #[inline]
    #[cfg(feature = "german")]
    fn parse_mit_frühen_beenden<'t>(
        args: impl Iterator<Item = OsString>,
    ) -> (ParseFrühesBeendenResult<'t, Self, Self::Error>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        let (result, remaining) = Self::parse_with_early_exit(args);
        (result.map_err(|errors| errors.map(Into::into)), remaining)
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// `parse_aus_env_mit_frühen_beenden`
    #[inline]
    fn parse_from_env_with_early_exit<'t>()
    -> (ParseEarlyExitResult<'t, Self, Self::Error>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        Self::arguments().parse_from_env_with_early_exit()
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [`parse_from_env_with_early_exit`](Parse::parse_from_env_with_early_exit)
    #[inline]
    #[cfg(feature = "german")]
    fn parse_aus_env_mit_frühen_beenden<'t>()
    -> (ParseFrühesBeendenResult<'t, Self, Self::Error>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Error: 't,
    {
        let (result, remaining) = Self::parse_from_env_with_early_exit();
        (result.map_err(|errors| errors.map(Into::into)), remaining)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// `parse_vollständig`
    #[inline]
    #[must_use]
    fn parse_complete(
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_complete(
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
    /// [`parse_complete`](Parse::parse_complete)
    #[inline]
    #[must_use]
    #[cfg(feature = "german")]
    fn parse_vollständig(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        arg_nicht_verwendet: &str,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_complete(
            args,
            fehler_code,
            fehlende_flag,
            fehlender_wert,
            parse_fehler,
            invalider_string,
            arg_nicht_verwendet,
        )
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// `parse_vollständig_mit_sprache`
    #[inline]
    #[must_use]
    fn parse_complete_with_language(
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        language: Language,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_complete_with_language(args, error_code, language)
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
    /// [`parse_complete_with_language`](Parse::parse_complete_with_language)
    #[inline]
    #[must_use]
    #[cfg(feature = "german")]
    fn parse_vollständig_mit_sprache(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        sprache: Sprache,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_complete_with_language(args, fehler_code, sprache.into())
    }

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// [`parse_complete_with_language`](Parse::parse_complete_with_language) with
    /// [`Language::ENGLISH`].
    ///
    /// ## Deutsche version
    /// `parse_mit_fehlermeldung`
    #[inline]
    #[must_use]
    fn parse_with_error_message(
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_with_error_message(args, error_code)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
    ///
    /// `parse_vollständig_mit_sprache` mit
    /// [`Sprache::DEUTSCH`].
    ///
    /// ## English version
    /// [`parse_with_error_message`](Parse::parse_with_error_message)
    #[inline]
    #[must_use]
    #[cfg(feature = "german")]
    fn parse_mit_fehlermeldung(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_vollständig_mit_sprache(args, fehler_code, Sprache::DEUTSCH)
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// `parse_vollständig_aus_env`
    #[inline]
    #[must_use]
    fn parse_complete_from_env(
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_complete_from_env(
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
    /// [`parse_complete_from_env`](Parse::parse_complete_from_env)
    #[inline]
    #[must_use]
    #[cfg(feature = "german")]
    fn parse_vollständig_aus_env(
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        arg_nicht_verwendet: &str,
    ) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_complete_from_env(
            fehler_code,
            fehlende_flag,
            fehlender_wert,
            parse_fehler,
            invalider_string,
            arg_nicht_verwendet,
        )
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// `parse_vollständig_mit_sprache_aus_env`
    #[inline]
    #[must_use]
    fn parse_complete_with_language_from_env(error_code: NonZeroI32, language: Language) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_complete_with_language_from_env(error_code, language)
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
    /// [`parse_complete_with_language_from_env`](Parse::parse_complete_with_language_from_env)
    #[inline]
    #[must_use]
    #[cfg(feature = "german")]
    fn parse_vollständig_mit_sprache_aus_env(fehler_code: NonZeroI32, sprache: Sprache) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_complete_with_language_from_env(fehler_code, sprache.into())
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// [`parse_complete_with_language_from_env`](Parse::parse_complete_with_language_from_env)
    /// with [`Language::ENGLISH`].
    ///
    /// ## Deutsche Version
    /// `parse_mit_fehlermeldung_aus_env`
    #[inline]
    #[must_use]
    fn parse_with_error_message_from_env(error_code: NonZeroI32) -> Self
    where
        Self::Error: Display,
    {
        Self::arguments().parse_with_error_message_from_env(error_code)
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code`
    /// beendet.
    ///
    /// `parse_vollständig_mit_sprache_aus_env`
    /// mit [`Sprache::DEUTSCH`].
    ///
    /// ## English version
    /// [`parse_with_error_message_from_env`](Parse::parse_with_error_message_from_env)
    #[inline]
    #[must_use]
    #[cfg(feature = "german")]
    fn parse_mit_fehlermeldung_aus_env(fehler_code: NonZeroI32) -> Self
    where
        Self::Error: Display,
    {
        Self::parse_vollständig_mit_sprache_aus_env(fehler_code, Sprache::DEUTSCH)
    }
}
