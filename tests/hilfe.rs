//! Tests zur automatisch erzeugen Hilfe.

use std::{ffi::OsString, iter, process};

use void::Void;

use kommandozeilen_argumente::{Argumente, Beschreibung, Ergebnis};

#[test]
fn hilfe_test() {
    let arg: Argumente<bool, Void> = Argumente::hilfe_und_version(
        Argumente::flag_bool_deutsch(Beschreibung::neu(
            "test".to_owned(),
            None::<&str>,
            Some("hilfe"),
            Some(false),
        )),
        "programm",
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
                    println!("{}", nachricht);
                }
            }
        },
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            process::exit(2);
        },
    }
}
