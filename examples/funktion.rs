use std::{
    fmt::{Debug, Display},
    num::NonZeroI32,
};

use kommandozeilen_argumente::{
    crate_name, crate_version, kombiniere, Argumente, Beschreibung, EnumArgument, NonEmpty,
    ParseArgument, ParseFehler, Sprache,
};

#[derive(Debug, Clone)]
enum Aufzählung {
    Eins,
    Zwei,
    Drei,
}

impl EnumArgument for Aufzählung {
    fn varianten() -> Vec<Self> {
        use Aufzählung::*;
        vec![Eins, Zwei, Drei]
    }

    fn parse_enum(
        arg: &std::ffi::OsStr,
    ) -> Result<Self, kommandozeilen_argumente::ParseFehler<String>> {
        use Aufzählung::*;
        if let Some(string) = arg.to_str() {
            // Vergleich-Strings enthalten nur ASCII-characters,
            // alle anderen können demnach ignoriert werden.
            let lowercase = string.to_ascii_lowercase();
            match lowercase.as_str() {
                "eins" => Ok(Eins),
                "zwei" => Ok(Zwei),
                "drei" => Ok(Drei),
                _ => Err(ParseFehler::ParseFehler(format!("Unbekannte Variante: {}", string))),
            }
        } else {
            Err(ParseFehler::InvaliderString(arg.to_owned()))
        }
    }
}

impl Display for Aufzählung {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug)]
struct Args {
    flag: bool,
    umbenannt: bool,
    benötigt: bool,
    wert: String,
    aufzählung: Aufzählung,
}

impl Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Args { flag, umbenannt, benötigt, wert, aufzählung } = self;
        write!(f, "flag: {flag}\n")?;
        write!(f, "umbenannt: {umbenannt}\n")?;
        write!(f, "benötigt: {benötigt}\n")?;
        write!(f, "wert: {wert}\n")?;
        write!(f, "aufzählung: {aufzählung}\n")
    }
}

fn main() {
    let sprache = Sprache::DEUTSCH;
    let flag = Argumente::flag_bool_mit_sprache(
        Beschreibung::neu(
            "flag",
            None::<&str>,
            Some("Eine Flag mit Standard-Einstellungen"),
            Some(false),
        ),
        sprache,
    );
    let umbenannt = Argumente::flag_bool_mit_sprache(
        Beschreibung::neu(
            NonEmpty { head: "andere", tail: vec!["namen"] },
            "u",
            Some("Eine Flag mit Standard-Einstellungen"),
            Some(false),
        ),
        sprache,
    );
    let benötigt = Argumente::flag_bool(
        Beschreibung::neu(
            "benötigt",
            "b",
            Some("Eine Flag ohne Standard-Wert mit alternativem Präfix zum invertieren."),
            None,
        ),
        "no",
    );
    let wert = String::argumente_mit_sprache(
        Beschreibung::neu("wert", None::<&str>, Some("Ein String-Wert."), None),
        sprache,
    );
    let aufzählung = Argumente::wert_enum_display(
        Beschreibung::neu(
            "aufzählung",
            "a",
            Some("Ein Aufzählung-Wert mit Standard-Wert und alternativer Meta-Variable."),
            Some(Aufzählung::Zwei),
        ),
        "VAR",
    );
    let zusammenfassen = |flag, umbenannt, benötigt, wert, aufzählung| Args {
        flag,
        umbenannt,
        benötigt,
        wert,
        aufzählung,
    };
    let argumente = kombiniere!(zusammenfassen, flag, umbenannt, benötigt, wert, aufzählung)
        .hilfe_und_version_mit_sprache(crate_name!(), crate_version!(), sprache);
    let args = argumente
        .parse_vollständig_mit_sprache_aus_env(NonZeroI32::new(1).expect("1 != 0"), sprache);
    println!("{:?}", args)
}
