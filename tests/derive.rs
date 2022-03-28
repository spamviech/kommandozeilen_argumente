//! Tests zum Parsen von Kommandozeilen-Argumenten, erzeugt über das derive-Feature.

use std::{
    ffi::OsString,
    fmt::{self, Debug, Display, Formatter},
    iter, process,
};

use kommandozeilen_argumente::{Argumente, EnumArgument, Ergebnis, Parse, ParseArgument};

#[derive(Debug, Clone, PartialEq, Eq, EnumArgument)]
#[kommandozeilen_argumente(case: insensitive)]
enum Bla {
    Meh,
    Muh,
}

impl Display for Bla {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

#[test]
fn arg_enum_derive() {
    assert_eq!(Bla::varianten(), vec![Bla::Meh, Bla::Muh]);
    let os_string: OsString = "meh".to_owned().into();
    let parse_res = Bla::parse_enum(os_string);
    assert_eq!(parse_res, Ok(Bla::Meh));
}

#[derive(Debug, PartialEq, Eq, Parse)]
#[kommandozeilen_argumente(sprache: deutsch, version, hilfe)]
struct Test {
    /// bla
    bla: Bla,
    /// opt
    #[kommandozeilen_argumente(lang: alternativ, kurz: [p, q, r])]
    opt: Option<Bla>,
    /// from_str
    #[kommandozeilen_argumente(FromStr, standard: 42, meta_var: VAR)]
    from_str: i32,
    /// flag
    #[kommandozeilen_argumente(standard: true)]
    flag: bool,
}

#[derive(Debug, PartialEq, Eq, Parse)]
#[kommandozeilen_argumente(language: english)]
struct Empty {}

const DUMMY: kommandozeilen_argumente::Sprache = kommandozeilen_argumente::Sprache {
    lang_präfix: "(-.-)",
    kurz_präfix: "~",
    invertiere_präfix: "dummy",
    invertiere_infix: "*",
    wert_infix: "+",
    meta_var: "dummy",
    optionen: "dummy",
    standard: "dummy",
    erlaubte_werte: "dummy",
    fehlende_flag: "dummy",
    fehlender_wert: "dummy",
    parse_fehler: "dummy",
    invalider_string: "dummy",
    argument_nicht_verwendet: "dummy",
    hilfe_beschreibung: "dummy",
    hilfe_lang: "dummy",
    hilfe_kurz: "dummy",
    version_beschreibung: "dummy",
    version_lang: "dummy",
    version_kurz: "dummy",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flag {
    Active,
    Inactive,
}

impl ParseArgument for Flag {
    fn argumente<'t>(
        beschreibung: kommandozeilen_argumente::Beschreibung<'t, Self>,
        invertiere_präfix: impl Into<kommandozeilen_argumente::Vergleich<'t>>,
        invertiere_infix: impl Into<kommandozeilen_argumente::Vergleich<'t>>,
        _wert_infix: impl Into<kommandozeilen_argumente::Vergleich<'t>>,
        _meta_var: &'t str,
    ) -> kommandozeilen_argumente::Argumente<'t, Self, String> {
        Argumente::flag(
            beschreibung,
            |b| if b { Flag::Active } else { Flag::Inactive },
            invertiere_präfix,
            invertiere_infix,
            |f| format!("{f:?}"),
        )
    }

    fn standard() -> Option<Self> {
        Some(Flag::Inactive)
    }
}

#[derive(Debug, PartialEq, Eq, Parse)]
#[kommandozeilen_argumente(language: DUMMY)]
struct Inner {
    #[kommandozeilen_argumente(default: false, short)]
    inner_flag: bool,
}

#[derive(Debug, PartialEq, Eq, Parse)]
#[kommandozeilen_argumente(version, help(lang: [hilfe, help], kurz: h))]
struct Test2 {
    #[kommandozeilen_argumente(default: Bla::Meh, long: [bla, meh, muh])]
    /// bla
    bla: Bla,
    /// flag
    #[kommandozeilen_argumente(required, short, invertiere_präfix: ähm)]
    flag: Flag,
    /// bool-flag
    #[kommandozeilen_argumente(required, short, invertiere_präfix: möp)]
    bool_flag: bool,
    #[kommandozeilen_argumente(flatten)]
    inner: Inner,
}

#[test]
fn derive_test() {
    let arg = Test::kommandozeilen_argumente();
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
        (Ergebnis::Fehler(fehler_sammlung), nicht_verwendet) => {
            for fehler in fehler_sammlung {
                eprintln!("{}", fehler.fehlermeldung())
            }
            eprintln!("{:?}", nicht_verwendet);
            process::exit(2);
        },
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            process::exit(3);
        },
    }
    println!("--------------");
    let arg2 = Test2::kommandozeilen_argumente();
    match arg2.parse(iter::once(OsString::from("--help".to_owned()))) {
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
        (Ergebnis::Fehler(fehler_sammlung), nicht_verwendet) => {
            for fehler in fehler_sammlung {
                eprintln!("{}", fehler.fehlermeldung())
            }
            eprintln!("{:?}", nicht_verwendet);
            process::exit(4);
        },
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            process::exit(5);
        },
    }
}

#[test]
fn verschmelze_kurzformen() {
    let arg = Test::kommandozeilen_argumente();
    match arg.parse(iter::once(OsString::from("-vh".to_owned()))) {
        (Ergebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
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
        },
        (Ergebnis::Fehler(fehler_sammlung), nicht_verwendet) => {
            for fehler in fehler_sammlung {
                eprintln!("{}", fehler.fehlermeldung())
            }
            eprintln!("{:?}", nicht_verwendet);
            process::exit(3);
        },
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            process::exit(4);
        },
    }
    println!("--------------");
    let arg2 = Test2::kommandozeilen_argumente();
    match arg2.parse(iter::once(OsString::from("-fb".to_owned()))) {
        (Ergebnis::Wert(test2), nicht_verwendet) => {
            let übrige = nicht_verwendet.iter().count();
            let erwartet = Test2 {
                bla: Bla::Meh,
                inner: Inner { inner_flag: false },
                flag: Flag::Active,
                bool_flag: true,
            };
            if übrige > 0 {
                eprintln!("Nicht verwendete Argumente: {:?}", nicht_verwendet);
                process::exit(5);
            } else if test2 != erwartet {
                eprintln!("Unerwarteter Wert: {:?} != {:?}", test2, erwartet);
                process::exit(6);
            } else {
                println!("{:?}", test2)
            }
        },
        res => {
            eprintln!("Unerwartetes Ergebnis: {:?}", res);
            process::exit(7);
        },
    }
}
