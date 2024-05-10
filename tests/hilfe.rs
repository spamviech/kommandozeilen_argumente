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

use std::{ffi::OsString, iter, process};

use void::Void;

use kommandozeilen_argumente::{
    argumente::{einzelargument::EinzelArgument, flag::Flag, hilfe::Standard},
    Argumente, Beschreibung, Ergebnis, Sprache,
};

#[test]
fn hilfe_test() {
    let sprache = Sprache::DEUTSCH;
    let flag = Flag::neu_mit_sprache(
        Beschreibung::neu_mit_sprache(
            "test".to_owned(),
            None::<&str>,
            Some("hilfe"),
            Some(false),
            sprache,
        ),
        sprache,
    );
    let einzelargument = EinzelArgument::flag(flag);
    let arg = Argumente::einzel_argument(einzelargument);
    let arg_mit_hilfe = arg.mit_hilfe_frühes_beenden_mit_sprache::<Standard>(
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
        sprache,
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
