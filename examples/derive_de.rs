//! Ein Beispiel des "derive"-features mit deutschen Namen.

#![expect(unused_crate_dependencies, reason = "dependencies of the lib")]

use core::{
    fmt::{self, Debug, Display},
    num::NonZeroI32,
};

use kommandozeilen_argumente::{EnumArgument, Parse};

/// Beispiel-enum um den Anwendung von [`EnumArgument`] zu zeigen.
#[derive(Debug, Clone, EnumArgument)]
#[kommandozeilen_argumente(case: insensitive)]
enum Aufzählung {
    /// eins
    Eins,
    /// zwei
    Zwei,
    /// drei
    Drei,
}

impl Display for Aufzählung {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

/// Struktur zur Definition der Kommandozeilen-Argumente.
#[derive(Debug, Parse)]
#[kommandozeilen_argumente(hilfe(beschreibung: "Programm-Beschreibung.", kurz))]
#[kommandozeilen_argumente(version, sprache: deutsch, programm(version))]
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
    /// Ein Aufzählung-Wert mit Standard-Wert und alternativer Meta-Variable.
    #[kommandozeilen_argumente(kurz, standard: Aufzählung::Zwei, meta_var: VAR)]
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
    let args = Args::parse_mit_fehlermeldung_aus_env(NonZeroI32::new(1i32).expect("1 != 0"));
    #[expect(clippy::print_stdout, reason = "example implementation.")]
    {
        println!("{args}");
    }
}
