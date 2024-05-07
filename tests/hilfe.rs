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
    convert::identity,
    ffi::{OsStr, OsString},
    iter, process,
};

use void::Void;

use kommandozeilen_argumente::{
    argumente::{einzelargument::EinzelArgument, flag::Flag, new},
    Argumente, Beschreibung, Ergebnis, ParseFehler, Sprache,
};

#[test]
fn hilfe_test() {
    let arg: Argumente<'_, bool, Void> = Argumente::hilfe_und_version(
        Argumente::flag_bool_deutsch(Beschreibung::neu_mit_sprache(
            "test".to_owned(),
            None::<&str>,
            Some("hilfe"),
            Some(false),
            Sprache::DEUTSCH,
        )),
        "programm",
        Some("Mein Tolles Programm."),
        "0.test",
    );
    match arg.parse(iter::once(OsString::from("--hilfe".to_owned()))) {
        (Ergebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
            let übrige = nicht_verwendet.len();
            if übrige > 0 {
                eprintln!("Nicht verwendete Argumente: {nicht_verwendet:?}");
                process::exit(1);
            } else {
                for nachricht in nachrichten {
                    println!("{nachricht}");
                }
            }
        },
        res => {
            eprintln!("Unerwartetes Ergebnis: {res:?}");
            process::exit(2);
        },
    }
}

#[test]
fn new_hilfe_test() {
    let sprache = Sprache::DEUTSCH;
    let flag: Flag<'_, _, fn(_) -> _, fn(&bool) -> String> = Flag {
        beschreibung: Beschreibung::neu_mit_sprache(
            "test".to_owned(),
            None::<&str>,
            Some("hilfe"),
            Some(false),
            sprache,
        ),
        invertiere_präfix: sprache.invertiere_präfix.into(),
        invertiere_infix: sprache.invertiere_infix.into(),
        konvertiere: identity,
        anzeige: <bool as ToString>::to_string,
    };
    let einzelargument = EinzelArgument::Flag(flag);
    let arg: new::Argumente<'_, _, _, fn(&OsStr) -> Result<bool, ParseFehler<Void>>, _, Void> =
        new::Argumente::EinzelArgument(einzelargument);
    // TODO erzeuge hilfe flag
    match arg.parse(iter::once(Some(OsString::from("--hilfe".to_owned())))) {
        (Ergebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
            let übrige = nicht_verwendet.len();
            if übrige > 0 {
                eprintln!("Nicht verwendete Argumente: {nicht_verwendet:?}");
                process::exit(1);
            } else {
                for nachricht in nachrichten {
                    println!("{nachricht}");
                }
            }
        },
        res => {
            eprintln!("Unerwartetes Ergebnis: {res:?}");
            process::exit(2);
        },
    }
}
