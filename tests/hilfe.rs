//! Tests zur automatisch erzeugen Hilfe.

use std::{ffi::OsString, iter, process};

use void::Void;

use kommandozeilen_argumente::{Arg, Beschreibung, Ergebnis};

#[allow(unused_imports)]
// Derive-Macro kommt mit integration test nicht zurecht, daher muss crate::kombiniere existieren.
use kommandozeilen_argumente::kombiniere;

#[test]
fn hilfe_test() {
    let arg: Arg<bool, Void> = Arg::hilfe_und_version(
        Arg::flag_deutsch(Beschreibung {
            lang: "test".to_owned(),
            kurz: None,
            hilfe: Some("hilfe".to_owned()),
            standard: Some(false),
        }),
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
