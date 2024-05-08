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
    argumente::{einzelargument::EinzelArgument, flag::Flag, hilfe::Standard, new},
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
    let arg_mit_hilfe = arg.mit_hilfe_frühes_beenden::<Standard, Void>(
        Beschreibung::neu_mit_sprache(
            sprache.hilfe_lang,
            sprache.hilfe_kurz,
            Some(sprache.hilfe_beschreibung),
            None,
            sprache,
        ),
        "programm",
        Some("Mein Tolles Programm."),
        Some("0.test"),
        sprache.standard,
        sprache.erlaubte_werte,
        sprache.optionen,
        sprache.syntax_präfix,
        sprache.syntax_padding,
        sprache.alternative_präfix,
        sprache.alternative_trennzeichen,
    );

    match arg_mit_hilfe.parse(iter::once(Some(OsString::from("--hilfe".to_owned())))) {
        (Ergebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
            let nicht_verwendet: Vec<_> = nicht_verwendet.into_iter().flatten().collect();
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
