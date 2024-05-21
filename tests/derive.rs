//! Tests zum Parsen von Kommandozeilen-Argumenten, erzeugt über das derive-Feature.

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
    borrow::Cow,
    ffi::OsString,
    fmt::{self, Debug, Display, Formatter},
    iter,
};

use nonempty::{nonempty, NonEmpty};

use kommandozeilen_argumente::{Argumente, EnumArgument, Ergebnis, Fehler, Parse, ParseArgument};

#[derive(Debug, Clone, PartialEq, Eq, EnumArgument)]
#[kommandozeilen_argumente(case: insensitive)]
enum Bla {
    Meh,
    Muh,
}

impl Display for Bla {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

#[test]
fn arg_enum_derive() {
    assert_eq!(Bla::varianten(), Some(nonempty![Bla::Meh, Bla::Muh]));
    let os_string: OsString = "meh".to_owned().into();
    let parse_res = Bla::parse_enum(&os_string);
    assert_eq!(parse_res, Ok(Bla::Meh));
}

struct DisplayAsNewline<Collection>(Collection);

impl<T: Display> Display for DisplayAsNewline<Vec<T>> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        for item in &self.0 {
            writeln!(formatter, "{item}")?;
        }
        Ok(())
    }
}

impl Display for DisplayAsNewline<NonEmpty<Cow<'_, str>>> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        for item in &self.0 {
            writeln!(formatter, "{item}")?;
        }
        Ok(())
    }
}

impl<T: Display> Display for DisplayAsNewline<NonEmpty<Fehler<'_, T>>> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        for fehler in &self.0 {
            writeln!(formatter, "{}", fehler.fehlermeldung())?;
        }
        Ok(())
    }
}

// Wrapper um String, mit Debug=Display Implementierung
struct DString(String);

impl Debug for DString {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Parse)]
#[kommandozeilen_argumente(sprache: deutsch, version, hilfe)]
struct Test {
    /// bla
    bla: Bla,
    /// opt
    #[kommandozeilen_argumente(lang: alternativ, kurz: [p, q, r])]
    opt: Option<Bla>,
    #[allow(clippy::doc_markdown)]
    /// from_str
    #[kommandozeilen_argumente(FromStr, standard: 42, meta_var: VAR)]
    from_str: i32,
    /// flag
    #[kommandozeilen_argumente(standard: true)]
    flag: bool,
}

#[derive(Debug, PartialEq, Eq, Parse)]
#[kommandozeilen_argumente(language: english)]
struct Empty;

#[test]
fn derive_hilfe_test() -> Result<(), DString> {
    let arg = Test::kommandozeilen_argumente();
    match arg.parse(iter::once(OsString::from("--hilfe".to_owned()))) {
        (Ergebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
            for nachricht in nachrichten {
                println!("{nachricht}");
            }
            if nicht_verwendet.is_empty() {
                Ok(())
            } else {
                Err(DString(format!("Nicht verwendete Argumente: {nicht_verwendet:?}")))
            }
        },
        (Ergebnis::Fehler(fehler_sammlung), nicht_verwendet) => {
            // FIXME: standard-Wert ignoriert, FrühesBeenden soll FehlenderWert/Flag "überschreiben"
            for fehler in &fehler_sammlung {
                eprintln!("{}", fehler.fehlermeldung());
            }
            eprintln!("{nicht_verwendet:?}");
            Err(DString(format!("Parsen mit fehler:\n{}", DisplayAsNewline(fehler_sammlung))))
        },
        res => Err(DString(format!("Unerwartetes Ergebnis: {res:?}"))),
    }
}

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
    syntax_präfix: "dummy",
    syntax_padding: 'd',
    alternative_präfix: "dummy",
    alternative_trennzeichen: 'd',
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
    ) -> Argumente<'t, Self, String> {
        Argumente::from(kommandozeilen_argumente::Flag {
            beschreibung,
            invertiere_präfix: invertiere_präfix.into(),
            invertiere_infix: invertiere_infix.into(),
            konvertiere: Cow::Borrowed(&|bool| {
                if bool {
                    Flag::Active
                } else {
                    Flag::Inactive
                }
            }),
            anzeige: Cow::Borrowed(&|flag| format!("{flag:?}")),
        })
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
fn derive_help_test() -> Result<(), DString> {
    let arg = Test2::kommandozeilen_argumente();
    match arg.parse(iter::once(OsString::from("--help".to_owned()))) {
        (Ergebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
            for nachricht in nachrichten {
                println!("{nachricht}");
            }
            if nicht_verwendet.is_empty() {
                Ok(())
            } else {
                Err(DString(format!("Nicht verwendete Argumente: {nicht_verwendet:?}")))
            }
        },
        (Ergebnis::Fehler(fehler_sammlung), nicht_verwendet) => {
            for fehler in &fehler_sammlung {
                eprintln!("{}", fehler.fehlermeldung());
            }
            eprintln!("{nicht_verwendet:?}");
            Err(DString(format!("Parsen mit fehler:\n{}", DisplayAsNewline(fehler_sammlung))))
        },
        res => Err(DString(format!("Unerwartetes Ergebnis: {res:?}"))),
    }
}

#[test]
fn verschmelze_kurzformen_hilfe() -> Result<(), DString> {
    let arg = Test::kommandozeilen_argumente();
    match arg.parse(iter::once(OsString::from("-vh".to_owned()))) {
        (Ergebnis::FrühesBeenden(nachrichten), nicht_verwendet) => {
            let übrige = nicht_verwendet.len();
            if übrige > 0 {
                Err(DString(format!("Nicht verwendete Argumente: {nicht_verwendet:?}")))
            } else if nachrichten.len() != 2 {
                Err(DString(format!(
                    "Unerwartete Anzahl an Nachrichten: {}",
                    DisplayAsNewline(nachrichten)
                )))
            } else {
                for nachricht in nachrichten {
                    println!("{nachricht}");
                }
                Ok(())
            }
        },
        (Ergebnis::Fehler(fehler_sammlung), nicht_verwendet) => {
            for fehler in &fehler_sammlung {
                eprintln!("{}", fehler.fehlermeldung());
            }
            eprintln!("{nicht_verwendet:?}");
            Err(DString(format!("Parsen mit fehler:\n{}", DisplayAsNewline(fehler_sammlung))))
        },
        res => Err(DString(format!("Unerwartetes Ergebnis: {res:?}"))),
    }
}

#[test]
fn verschmelze_kurzformen_wert() -> Result<(), DString> {
    let arg2 = Test2::kommandozeilen_argumente();
    match arg2.parse(iter::once(OsString::from("-fb".to_owned()))) {
        (Ergebnis::Wert(test2), nicht_verwendet) => {
            let übrige = nicht_verwendet.len();
            let erwartet = Test2 {
                bla: Bla::Meh,
                inner: Inner { inner_flag: false },
                flag: Flag::Active,
                bool_flag: true,
            };
            if übrige > 0 {
                Err(DString(format!("Nicht verwendete Argumente: {nicht_verwendet:?}")))
            } else if test2 != erwartet {
                Err(DString(format!("Unerwarteter Wert: {test2:?} != {erwartet:?}")))
            } else {
                println!("{test2:?}");
                Ok(())
            }
        },
        res => Err(DString(format!("Unerwartetes Ergebnis: {res:?}"))),
    }
}
