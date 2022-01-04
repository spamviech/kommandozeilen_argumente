//! Tests zum Parsen von Kommandozeilen-Argumenten

use std::{ffi::OsString, iter};

use crate::*;

#[derive(Debug, PartialEq, Eq, ArgEnum)]
enum Bla {
    Meh,
    Muh,
}

#[derive(Debug)]
#[kommandozeilen_argumente(hilfe, version_deutsch)]
struct Test {
    /// bla
    bla: Bla,
}

#[test]
fn arg_enum_derive() {
    assert_eq!(Bla::varianten(), vec![Bla::Meh, Bla::Muh]);
    let os_string: OsString = "Meh".to_owned().into();
    let parse_res = Bla::parse_enum(os_string.as_os_str());
    assert_eq!(parse_res, Ok(Bla::Meh));
}

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
    let hilfe = OsString::from("--hilfe".to_owned());
    match arg.parse(iter::once(hilfe)) {
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
