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
    ffi::OsString,
    fmt::{self, Debug, Formatter},
    iter,
};

use kommandozeilen_argumente::{
    argumente::{einzelargument::EinzelArgument, flag::Flag, hilfe::Standard},
    ergebnis::ZwischenErgebnis,
    Argumente, Beschreibung, Sprache,
};

// Wrapper um String, mit Debug=Display Implementierung
struct DString(String);

impl Debug for DString {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[test]
fn hilfe_test() -> Result<(), DString> {
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
    let arg_mit_hilfe = arg.mit_hilfe_frühes_beenden_mit_sprache(
        &Standard,
        "programm",
        Some("Mein Tolles Programm."),
        Some("0.test"),
        sprache,
    );

    match arg_mit_hilfe
        .parse_rekursiv(iter::once(Some(OsString::from("--hilfe".to_owned()).as_os_str())))
    {
        (ZwischenErgebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
            let nicht_verwendet: Vec<_> = nicht_verwendet.into_iter().flatten().collect();
            let übrige = nicht_verwendet.len();
            if übrige > 0 {
                Err(DString(format!("Nicht verwendete Argumente: {nicht_verwendet:?}")))
            } else {
                for nachricht in nachrichten {
                    println!("{nachricht}");
                }
                Ok(())
            }
        },
        res => Err(DString(format!("Unerwartetes Ergebnis: {res:?}"))),
    }
}
