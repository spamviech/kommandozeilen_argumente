//! Tests zur automatisch erzeugen Hilfe.

// dependencies of the lib
#![allow(unused_crate_dependencies)]

use std::{ffi::OsString, iter, process};

use void::Void;

use kommandozeilen_argumente::{Argumente, Beschreibung, Ergebnis, Sprache};

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
            let übrige = nicht_verwendet.iter().count();
            if übrige > 0 {
                eprintln!("Nicht verwendete Argumente: {:?}", nicht_verwendet);
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
