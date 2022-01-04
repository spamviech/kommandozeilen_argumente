//! Tests zum Parsen von Kommandozeilen-Argumenten

use std::{
    ffi::{OsStr, OsString},
    fmt::Display,
    iter,
};

use crate::*;

#[test]
fn hilfe_test() {
    let arg: Arg<bool, Void> = Arg::hilfe_und_version(
        Arg::flag_deutsch(ArgBeschreibung {
            lang: "test".to_owned(),
            kurz: None,
            hilfe: Some("hilfe".to_owned()),
            standard: Some(false),
        }),
        "programm",
        "0.test",
        20,
    );
    match arg.parse(iter::once(OsString::from("--hilfe".to_owned()))) {
        (ParseErgebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
            let übrige = nicht_verwendet.iter().count();
            if übrige > 0 {
                eprintln!("Nicht verwendete Argumente: {:?}", nicht_verwendet);
                std::process::exit(1);
            } else {
                for nachricht in nachrichten {
                    println!("{}", nachricht);
                }
                std::process::exit(0);
            }
        }
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            std::process::exit(2);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ArgEnum)]
enum Bla {
    Meh,
    Muh,
}

impl Display for Bla {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[test]
fn arg_enum_derive() {
    assert_eq!(Bla::varianten(), vec![Bla::Meh, Bla::Muh]);
    let os_string: OsString = "Meh".to_owned().into();
    let parse_res = Bla::parse_enum(os_string.as_os_str());
    assert_eq!(parse_res, Ok(Bla::Meh));
}

#[derive(Debug, PartialEq, Eq)]
#[kommandozeilen_argumente(version_deutsch, hilfe)]
struct Test {
    /// bla
    bla: Bla,
    /// meh
    meh: Bla,
}

#[test]
fn derive_test() {
    let arg = Test::kommandozeilen_argumente();
    match arg.parse(iter::once(OsString::from("--hilfe".to_owned()))) {
        (ParseErgebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
            let übrige = nicht_verwendet.iter().count();
            if übrige > 0 {
                eprintln!("Nicht verwendete Argumente: {:?}", nicht_verwendet);
                std::process::exit(1);
            } else {
                for nachricht in nachrichten {
                    println!("{}", nachricht);
                }
                std::process::exit(0);
            }
        }
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            std::process::exit(2);
        }
    }
}
