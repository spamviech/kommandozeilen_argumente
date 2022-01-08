//! Tests zur automatisch erzeugen Hilfe.

use std::{ffi::OsString, iter, process};

use void::Void;

use kommandozeilen_argumente::{Argumente, Beschreibung, Ergebnis};

#[allow(unused_imports)]
// Derive-Macro kommt mit integration test nicht zurecht, daher muss crate::kombiniere existieren.
use kommandozeilen_argumente::kombiniere;

#[test]
fn hilfe_test() {
    let arg: Argumente<bool, Void> = Argumente::hilfe_und_version(
        Argumente::flag_deutsch(Beschreibung::neu(
            "test".to_owned(),
            None,
            Some("hilfe".to_owned()),
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
        }
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            process::exit(2);
        }
    }
}
