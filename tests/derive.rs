//! Tests zum Parsen von Kommandozeilen-Argumenten, erzeugt über das derive-Feature.

use std::{ffi::OsString, fmt::Display, iter, process};

use kommandozeilen_argumente::{Arg, ArgEnum, Parse, ParseErgebnis};

#[allow(unused_imports)]
// Derive-Macro kommt mit integration test nicht zurecht, daher muss crate::kombiniere existieren.
use kommandozeilen_argumente::kombiniere;

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

#[derive(Debug, PartialEq, Eq, Parse)]
#[kommandozeilen_argumente(version, hilfe)]
struct Test {
    /// bla
    bla: Bla,
    /// opt
    opt: Option<Bla>,
    /// flag
    #[kommandozeilen_argumente(standard(true))]
    flag: bool,
}

#[derive(Debug, PartialEq, Eq, Parse)]
#[kommandozeilen_argumente(english)]
struct Empty {}

#[derive(Debug, PartialEq, Eq, Parse)]
#[kommandozeilen_argumente(english)]
struct Inner {
    #[kommandozeilen_argumente(default(false), short)]
    empty: bool,
}

#[derive(Debug, PartialEq, Eq, Parse)]
#[kommandozeilen_argumente(english, version, help)]
struct Test2 {
    #[kommandozeilen_argumente(default(Bla::Meh))]
    /// bla
    bla: Bla,
    /// flag
    #[kommandozeilen_argumente(required, short)]
    flag: bool,
    #[kommandozeilen_argumente(flatten)]
    inner: Inner,
}

#[test]
fn derive_test() {
    let arg = Test::kommandozeilen_argumente();
    match arg.parse(iter::once(OsString::from("--hilfe".to_owned()))) {
        (ParseErgebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
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
        (ParseErgebnis::Fehler(fehler), nicht_verwendet) => {
            for f in fehler {
                match f.als_string() {
                    Ok(f_str) => eprintln!("{}", f_str.fehlermeldung()),
                    Err(f_os_str) => eprintln!("{:?}", f_os_str),
                }
            }
            eprintln!("{:?}", nicht_verwendet);
            process::exit(2);
        }
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            process::exit(3);
        }
    }
    println!("--------------");
    let arg2 = Test2::kommandozeilen_argumente();
    match arg2.parse(iter::once(OsString::from("--help".to_owned()))) {
        (ParseErgebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
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
        (ParseErgebnis::Fehler(fehler), nicht_verwendet) => {
            for f in fehler {
                match f.als_string() {
                    Ok(f_str) => eprintln!("{}", f_str.error_message()),
                    Err(f_os_str) => eprintln!("{:?}", f_os_str),
                }
            }
            eprintln!("{:?}", nicht_verwendet);
            process::exit(4);
        }
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            process::exit(5);
        }
    }
}

#[test]
fn verschmelze_kurzformen() {
    let arg = Test::kommandozeilen_argumente();
    match arg.parse(iter::once(OsString::from("-vh".to_owned()))) {
        (ParseErgebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
            let übrige = nicht_verwendet.iter().count();
            if übrige > 0 {
                eprintln!("Nicht verwendete Argumente: {:?}", nicht_verwendet);
                process::exit(1);
            } else if nachrichten.len() != 2 {
                eprintln!("Unerwartete Anzahl an Nachrichten: {:?}", nachrichten);
                process::exit(2);
            } else {
                for nachricht in nachrichten {
                    println!("{}", nachricht);
                }
            }
        }
        (ParseErgebnis::Fehler(fehler), nicht_verwendet) => {
            for f in fehler {
                match f.als_string() {
                    Ok(f_str) => eprintln!("{}", f_str.error_message()),
                    Err(f_os_str) => eprintln!("{:?}", f_os_str),
                }
            }
            eprintln!("{:?}", nicht_verwendet);
            process::exit(3);
        }
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            process::exit(4);
        }
    }
    println!("--------------");
    let arg2 = Test2::kommandozeilen_argumente();
    match arg2.parse(iter::once(OsString::from("-fe".to_owned()))) {
        (ParseErgebnis::Wert(test2), nicht_verwendet) => {
            let übrige = nicht_verwendet.iter().count();
            let erwartet = Test2 { bla: Bla::Meh, inner: Inner { empty: true }, flag: true };
            if übrige > 0 {
                eprintln!("Nicht verwendete Argumente: {:?}", nicht_verwendet);
                process::exit(5);
            } else if test2 != erwartet {
                eprintln!("Unerwarteter Wert: {:?} != {:?}", test2, erwartet);
                process::exit(6);
            } else {
                println!("{:?}", test2)
            }
        }
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            process::exit(7);
        }
    }
}
