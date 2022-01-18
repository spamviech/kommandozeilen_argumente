use std::{
    fmt::{Debug, Display},
    num::NonZeroI32,
};

use kommandozeilen_argumente::{EnumArgument, Parse};

#[derive(Debug, Clone, EnumArgument)]
enum Aufzählung {
    Eins,
    Zwei,
    Drei,
}

impl Display for Aufzählung {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug, Parse)]
#[kommandozeilen_argumente(hilfe, version, sprache: deutsch)]
struct Args {
    /// Eine Flag mit Standard-Einstellungen.
    flag: bool,
    /// Eine Flag mit alternativen Namen.
    #[kommandozeilen_argumente(lang: [andere, namen], kurz: u)]
    umbenannt: bool,
    /// Eine Flag ohne Standard-Wert mit alternativem Präfix zum invertieren.
    #[kommandozeilen_argumente(benötigt, kurz, invertiere_präfix: no)]
    benötigt: bool,
    /// Ein String-Wert.
    wert: String,
    /// Ein Aufzählung-Wert mit Standard-Wert.
    #[kommandozeilen_argumente(kurz, standard: Aufzählung::Zwei)]
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
    let args = Args::parse_mit_fehlermeldung_aus_env(NonZeroI32::new(1).expect("1 != 0"));
    println!("{:?}", args)
}
