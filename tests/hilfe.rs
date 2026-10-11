//! Tests zur automatisch erzeugen Hilfe.

// dependencies of the lib
#![allow(unused_crate_dependencies)]
#![allow(
    clippy::tests_outside_test_module,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit,
    clippy::use_debug
)]

use std::{
    ffi::OsString,
    fmt::{self, Debug, Formatter},
    iter,
};

use kommandozeilen_argumente::{
    Argumente, Arguments, Beschreibung, Description, Ergebnis, Language, Result, Sprache,
    arguments::{
        flag::Flag,
        help::{Default, Standard},
        single_argument::{EinzelArgument, SingleArgument},
    },
};

// Wrapper um String, mit Debug=Display Implementierung
struct DString(String);

impl Debug for DString {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[test]
fn help_test() -> std::result::Result<(), DString> {
    let language = Language::ENGLISH;
    let flag = Flag::new_with_language(
        Description::new_with_language("test", None::<&str>, Some("help"), Some(false), language),
        language,
    );
    let argument = Arguments::single_argument(SingleArgument::flag(flag));
    let arguments_with_help = argument.with_help_early_exit_with_language(
        &Default,
        "program",
        Some("My great program."),
        Some("0.test"),
        language,
    );

    match arguments_with_help.parse(iter::once(OsString::from("--help"))) {
        (Result::EarlyExit(messages), unused) if unused.is_empty() => {
            for message in messages {
                println!("{message}");
            }
            Ok(())
        },
        (_result, unused) if !unused.is_empty() => {
            Err(DString(format!("Unused arguments: {unused:?}")))
        },
        result => Err(DString(format!("Unexpected result: {result:?}"))),
    }
}

#[test]
fn hilfe_test() -> std::result::Result<(), DString> {
    let sprache = Sprache::DEUTSCH;
    let flag = Flag::neu_mit_sprache(
        Beschreibung::neu_mit_sprache("test", None::<&str>, Some("hilfe"), Some(false), sprache),
        sprache,
    );
    let argument = Argumente::einzel_argument(EinzelArgument::flag(flag));
    let argumente_mit_hilfe = argument.mit_hilfe_frühes_beenden_mit_sprache(
        &Standard,
        "programm",
        Some("Mein tolles Programm."),
        Some("0.test"),
        sprache,
    );

    match argumente_mit_hilfe.parse(iter::once(OsString::from("--hilfe"))) {
        (Ergebnis::FrühesBeenden(nachrichten), nicht_verwendet) if nicht_verwendet.is_empty() => {
            for nachricht in nachrichten {
                println!("{nachricht}");
            }
            Ok(())
        },
        (_ergebnis, nicht_verwendet) if !nicht_verwendet.is_empty() => {
            Err(DString(format!("Nicht verwendete Argumente: {nicht_verwendet:?}")))
        },
        ergebnis => Err(DString(format!("Unerwartetes Ergebnis: {ergebnis:?}"))),
    }
}
