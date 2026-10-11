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

use kommandozeilen_argumente::{
    ArgumentInput, Arguments, Compare, Description, EnumArgument, Error, Language, Normalized,
    Parse, ParseArgument, Result,
    description::{AdjustedMergedShortNames, MergedShortNameSuffix},
};
use nonempty::{NonEmpty, nonempty};

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

impl<T: Display> Display for DisplayAsNewline<NonEmpty<Error<'_, T>>> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        for error in &self.0 {
            writeln!(formatter, "{}", error.error_message())?;
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
fn derive_empty_test() -> std::result::Result<(), DString> {
    let arguments: Arguments<'_, Empty, String> = Empty::arguments();
    match arguments.parse(iter::empty()) {
        (Result::Value(Empty), unused) if unused.is_empty() => Ok(()),
        (_result, unused) if !unused.is_empty() => {
            Err(DString(format!("Unused arguments: {unused:?}")))
        },
        result => Err(DString(format!("Unexpected result: {result:?}"))),
    }
}

#[test]
fn derive_hilfe_test() -> std::result::Result<(), DString> {
    let arg = Test::arguments();
    match arg.parse(iter::once(OsString::from("--hilfe"))) {
        (Result::EarlyExit(nachrichten), nicht_verwendet) => {
            for nachricht in nachrichten {
                println!("{nachricht}");
            }
            if nicht_verwendet.is_empty() {
                Ok(())
            } else {
                Err(DString(format!("Nicht verwendete Argumente: {nicht_verwendet:?}")))
            }
        },
        (Result::Error(fehler_sammlung), nicht_verwendet) => {
            for fehler in &fehler_sammlung {
                eprintln!("{}", fehler.error_message());
            }
            eprintln!("{nicht_verwendet:?}");
            Err(DString(format!("Parsen mit fehler:\n{}", DisplayAsNewline(fehler_sammlung))))
        },
        res => Err(DString(format!("Unerwartetes Ergebnis: {res:?}"))),
    }
}

const DUMMY: Language = Language {
    long_prefix: "(-.-)",
    short_prefix: "~",
    invert_prefix: "dummy",
    invert_infix: "*",
    value_infix: "+",
    meta_var: "dummy",
    options: "dummy",
    default: "dummy",
    allowed_values: "dummy",
    missing_flag: "dummy",
    missing_value: "dummy",
    parse_error: "dummy",
    invalid_string: "dummy",
    unused_argument: "dummy",
    help_description: "dummy",
    help_long: "dummy",
    help_short: "dummy",
    version_description: "dummy",
    version_long: "dummy",
    version_short: "dummy",
    syntax_prefix: "dummy",
    syntax_padding: 'd',
    alternative_prefix: "dummy",
    alternative_separator: 'd',
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flag {
    Active,
    Inactive,
}

impl ParseArgument for Flag {
    fn arguments<'t>(
        description: Description<'t, Self>,
        invert_prefix: impl Into<Compare<'t>>,
        invert_infix: impl Into<Compare<'t>>,
        _value_infix: impl Into<Compare<'t>>,
        _meta_var: &'t str,
    ) -> Arguments<'t, Self, String> {
        Arguments::from(kommandozeilen_argumente::Flag {
            description,
            invert_prefix: invert_prefix.into(),
            invert_infix: invert_infix.into(),
            convert: Cow::Borrowed(&|bool| {
                if bool { Flag::Active } else { Flag::Inactive }
            }),
            display: Cow::Borrowed(&|flag| format!("{flag:?}")),
        })
    }

    fn default() -> Option<Self> {
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
    #[kommandozeilen_argumente(default: Bla::Meh, long: [bla, meh, muh], short: x)]
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
fn derive_help_test() -> std::result::Result<(), DString> {
    let arg = Test2::arguments();
    match arg.parse(iter::once(OsString::from("--help"))) {
        (Result::EarlyExit(nachrichten), nicht_verwendet) => {
            for nachricht in nachrichten {
                println!("{nachricht}");
            }
            if nicht_verwendet.is_empty() {
                Ok(())
            } else {
                Err(DString(format!("Nicht verwendete Argumente: {nicht_verwendet:?}")))
            }
        },
        (Result::Error(fehler_sammlung), nicht_verwendet) => {
            for fehler in &fehler_sammlung {
                eprintln!("{}", fehler.error_message());
            }
            eprintln!("{nicht_verwendet:?}");
            Err(DString(format!("Parsen mit fehler:\n{}", DisplayAsNewline(fehler_sammlung))))
        },
        res => Err(DString(format!("Unerwartetes Ergebnis: {res:?}"))),
    }
}

#[test]
fn verschmelze_kurzformen_hilfe() -> std::result::Result<(), DString> {
    let arg = Test::arguments();
    match arg.parse(iter::once(OsString::from("-vh"))) {
        (Result::EarlyExit(nachrichten), nicht_verwendet) => {
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
        (Result::Error(fehler_sammlung), nicht_verwendet) => {
            for fehler in &fehler_sammlung {
                eprintln!("{}", fehler.error_message());
            }
            eprintln!("{nicht_verwendet:?}");
            Err(DString(format!("Parsen mit fehler:\n{}", DisplayAsNewline(fehler_sammlung))))
        },
        res => Err(DString(format!("Unerwartetes Ergebnis: {res:?}"))),
    }
}

#[test]
fn verschmelze_kurzformen_wert() -> std::result::Result<(), DString> {
    // soll nicht für Wert-Argumente (vor allem am Anfang der Liste) funktionieren!
    // FIXME wert am Anfang soll keinen Parse-Fehler auslösen!
    let arg2 = Test2::arguments();
    match arg2.parse(
        [OsString::from(String::from("-xfb")), OsString::from(String::from("Muh"))].into_iter(),
    ) {
        (Result::Value(test2), nicht_verwendet) => {
            let erwartet = Test2 {
                bla: Bla::Meh,
                inner: Inner { inner_flag: false },
                flag: Flag::Active,
                bool_flag: true,
            };
            // Der Wert Kurz-Name soll nicht "nach hinten durchrutschen"!
            let erwartet_nicht_verwendet = [
                ArgumentInput::AdjustedMergedShortNames(AdjustedMergedShortNames {
                    prefix: Normalized::new(Cow::from("-")),
                    graphemes: nonempty![Box::from("x")],
                    suffix: MergedShortNameSuffix::Removed,
                }),
                ArgumentInput::Unchanged(OsString::from(String::from("Muh"))),
            ];
            if nicht_verwendet != erwartet_nicht_verwendet {
                Err(DString(format!("Unerwartete nicht verwendete Argumente: {nicht_verwendet:?}")))
            } else if test2 != erwartet {
                Err(DString(format!("Unerwarteter Wert: {test2:?} != {erwartet:?}")))
            } else {
                println!("{test2:?}");
                println!("{nicht_verwendet:?}");
                Ok(())
            }
        },
        res => Err(DString(format!("Unerwartetes Ergebnis: {res:?}"))),
    }
}

#[test]
fn verschmelze_kurzformen_erfolgreich() -> std::result::Result<(), DString> {
    let arg2 = Test2::arguments();
    match arg2.parse(iter::once(OsString::from("-fb"))) {
        (Result::Value(test2), nicht_verwendet) => {
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
