//! Ein Beispiel des Funktion-Apis.

// dependencies of the lib
#![allow(unused_crate_dependencies)]

use std::{
    borrow::Cow,
    convert::identity,
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
    num::NonZeroI32,
};

use nonempty::{nonempty, NonEmpty};

use kommandozeilen_argumente::{
    argumente::{flag::Flag, hilfe::Standard, wert::Wert},
    crate_name, crate_version, kombiniere, Argumente, Beschreibung, EnumArgument, ParseArgument,
    ParseFehler, Sprache, Vergleich,
};

/// Beispiel-enum um den Anwendung von [`EnumArgument`] zu zeigen.
#[derive(Debug, Clone)]
enum Aufzählung {
    /// eins
    Eins,
    /// zwei
    Zwei,
    /// drei
    Drei,
}

impl EnumArgument for Aufzählung {
    fn varianten() -> Option<NonEmpty<Self>> {
        use Aufzählung::{Drei, Eins, Zwei};
        Some(nonempty![Eins, Zwei, Drei])
    }

    fn parse_enum(arg: &OsStr) -> Result<Self, ParseFehler<String>> {
        use Aufzählung::{Drei, Eins, Zwei};
        if let Some(string) = arg.to_str() {
            // Vergleich-Strings enthalten nur ASCII-Zeichen,
            // alle anderen können demnach ignoriert werden.
            let lowercase = string.to_ascii_lowercase();
            match lowercase.as_str() {
                "eins" => Ok(Eins),
                "zwei" => Ok(Zwei),
                "drei" => Ok(Drei),
                _ => Err(ParseFehler::ParseFehler(format!("Unbekannte Variante: {string}"))),
            }
        } else {
            Err(ParseFehler::InvaliderString(OsString::from(arg)))
        }
    }
}

impl Display for Aufzählung {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

/// Struktur um die geparsten Kommandozeilen-Argumente zu repräsentieren.
#[derive(Debug)]
struct Args {
    /// Eine Flag mit Standard-Einstellungen.
    flag: bool,
    /// Eine Flag mit alternativen Namen.
    umbenannt: bool,
    /// Eine Flag ohne Standard-Wert mit alternativem Präfix zum invertieren.
    benötigt: bool,
    /// Ein String-Wert.
    wert: String,
    /// Ein Aufzählung-Wert mit Standard-Wert und alternativer Meta-Variable.
    aufzählung: Aufzählung,
}

impl Display for Args {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Args { flag, umbenannt, benötigt, wert, aufzählung } = self;
        writeln!(formatter, "flag: {flag}")?;
        writeln!(formatter, "umbenannt: {umbenannt}")?;
        writeln!(formatter, "benötigt: {benötigt}")?;
        writeln!(formatter, "wert: {wert}")?;
        writeln!(formatter, "aufzählung: {aufzählung}")
    }
}

fn main() {
    let sprache = Sprache::DEUTSCH;
    let flag = Argumente::from(Flag::neu_mit_sprache(
        Beschreibung::neu_mit_sprache(
            "flag",
            None::<&str>,
            Some("Eine Flag mit Standard-Einstellungen."),
            Some(false),
            sprache,
        ),
        sprache,
    ));
    let umbenannt = Argumente::from(Flag::neu_mit_sprache(
        Beschreibung::neu_mit_sprache(
            NonEmpty { head: "andere", tail: vec!["namen"] },
            "u",
            Some("Eine Flag mit alternativen Namen."),
            Some(false),
            sprache,
        ),
        sprache,
    ));
    let benötigt = Argumente::from(Flag {
        beschreibung: Beschreibung::neu_mit_sprache(
            "benötigt",
            "b",
            Some("Eine Flag ohne Standard-Wert mit alternativem Präfix zum invertieren."),
            None,
            sprache,
        ),
        invertiere_präfix: Vergleich::from("no"),
        invertiere_infix: Vergleich::from(sprache.invertiere_infix),
        konvertiere: Cow::Borrowed(&identity),
        anzeige: Cow::Borrowed(&ToString::to_string),
    });
    let wert = Argumente::einzel_argument(String::argumente_mit_sprache(
        Beschreibung::neu_mit_sprache(
            "wert",
            None::<&str>,
            Some("Ein String-Wert."),
            None,
            sprache,
        ),
        sprache,
    ));
    let aufzählung = Argumente::from(Wert::neu_enum_mit_sprache(
        Beschreibung::neu_mit_sprache(
            "aufzählung",
            "a",
            Some("Ein Aufzählung-Wert mit Standard-Wert und alternativer Meta-Variable."),
            Some(Aufzählung::Zwei),
            sprache,
        ),
        sprache,
    ));
    #[allow(clippy::shadow_unrelated)]
    let zusammenfassen = |flag, umbenannt, benötigt, wert, aufzählung| Args {
        flag,
        umbenannt,
        benötigt,
        wert,
        aufzählung,
    };
    let argumente = kombiniere!(zusammenfassen, flag, umbenannt, benötigt, wert, aufzählung);
    let argumente_mit_hilfe_und_version = argumente
        .mit_hilfe_und_version_frühes_beenden_mit_sprache(
            &Standard,
            crate_name!(),
            Some("Programm-Beschreibung."),
            crate_version!(),
            sprache,
        );
    let args = argumente_mit_hilfe_und_version
        .parse_vollständig_mit_sprache_aus_env(NonZeroI32::new(1).expect("1 != 0"), sprache);
    #[allow(clippy::print_stdout, clippy::use_debug)]
    {
        println!("{args:?}");
    }
}
