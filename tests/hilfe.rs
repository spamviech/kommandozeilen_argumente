//! Tests zur automatisch erzeugen Hilfe.

#![expect(unused_crate_dependencies, reason = "dependencies of the lib")]
#![expect(clippy::tests_outside_test_module, clippy::print_stdout, reason = "Integration test.")]

use std::{
    assert_matches,
    ffi::OsString,
    fmt::{self, Debug, Formatter},
    iter,
    result::Result,
};

use kommandozeilen_argumente::{
    Argumente, Beschreibung, Ergebnis, Sprache, Wert,
    arguments::{flag::Flag, help::Standard, single_argument::EinzelArgument},
};

// Wrapper um String, mit Debug=Display Implementierung
struct DString(String);

impl Debug for DString {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[test]
fn german_public_api_parses_canonically_equivalent_unicode_names() {
    let wert = Wert::neu(Beschreibung::neu("--", "pört", "-", "p", None, None), None);
    let argumente = Argumente::from(wert);
    let (ergebnis, verbleibend) =
        argumente.parse([OsString::from("--po\u{308}rt=8080")].into_iter());

    assert_eq!(verbleibend, []);
    assert_matches!(ergebnis, Ergebnis::Wert(8080));
}

#[test]
fn unicode_hilfe_uses_byte_length_padding() {
    let sprache = Sprache::DEUTSCH;
    let flag = Flag::neu_mit_sprache(
        Beschreibung::neu_mit_sprache("föö", "f", Some("Unicode-Flag."), Some(false), sprache),
        sprache,
    );
    let argumente = Argumente::einzel_argument(EinzelArgument::flag(flag))
        .mit_hilfe_frühes_beenden_mit_sprache(&Standard, "programm", None, None, sprache);
    let (ergebnis, verbleibend) = argumente.parse(iter::once(OsString::from("--hilfe")));

    assert_eq!(verbleibend, []);
    let Ergebnis::FrühesBeenden(nachrichten) = ergebnis else {
        unreachable!("expected help early exit");
    };
    assert!(
        nachrichten.head.contains("OPTIONEN:\n")
            && nachrichten.head.contains("  --[kein-]föö | -f Unicode-Flag. Standard: false\n")
            && nachrichten.head.contains("  --hilfe | -h        Zeige diesen Text an.\n"),
        "{}",
        nachrichten.head
    );
}

#[test]
fn hilfe_test() -> Result<(), DString> {
    let sprache = Sprache::DEUTSCH;
    let flag = Flag::neu_mit_sprache(
        Beschreibung::neu_mit_sprache("test", None::<&str>, Some("hilfe"), Some(false), sprache),
        sprache,
    );
    let argument = Argumente::einzel_argument(EinzelArgument::flag(flag));
    let argumente_mit_hilfe = argument.mit_hilfe_frühes_beenden_mit_sprache(
        &Standard,
        "programm",
        Some("Mein tolles Programm."),
        Some("0.test"),
        sprache,
    );

    match argumente_mit_hilfe.parse(iter::once(OsString::from("--hilfe"))) {
        (Ergebnis::FrühesBeenden(nachrichten), nicht_verwendet) if nicht_verwendet.is_empty() => {
            for nachricht in nachrichten {
                println!("{nachricht}");
            }
            Ok(())
        },
        (_ergebnis, nicht_verwendet) if !nicht_verwendet.is_empty() => {
            Err(DString(format!("Nicht verwendete Argumente: {nicht_verwendet:?}")))
        },
        ergebnis => Err(DString(format!("Unerwartetes Ergebnis: {ergebnis:?}"))),
    }
}
