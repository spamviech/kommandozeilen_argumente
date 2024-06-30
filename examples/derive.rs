//! Ein Beispiel des "derive"-features mit deutschen Namen.

// dependencies of the lib
#![allow(unused_crate_dependencies)]
// checking if the derive macro triggers any lint
#![allow(clippy::blanket_clippy_restriction_lints)]
#![warn(clippy::restriction)]
// Funktioniert nur, wenn es für das gesamte Modul deaktiviert wird.
// Benötigt für einige [`Sprache`]-Felder.
#![allow(clippy::disallowed_script_idents)]

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
        #[allow(clippy::implicit_return)]
        Debug::fmt(self, formatter)
    }
}

/// Struktur zur Definition der Kommandozeilen-Argumente.
#[derive(Debug, Parse)]
#[kommandozeilen_argumente(hilfe(beschreibung: "Programm-Beschreibung.", kurz))]
#[kommandozeilen_argumente(version, sprache: deutsch, programm(version))]
// wegen `invertiere_präfix`
#[allow(clippy::disallowed_script_idents)]
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
    #[allow(
        clippy::pattern_type_mismatch,
        clippy::question_mark_used,
        clippy::implicit_return,
        clippy::non_ascii_literal
    )]
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
    #[allow(clippy::expect_used, clippy::separated_literal_suffix)]
    let args = Args::parse_mit_fehlermeldung_aus_env(NonZeroI32::new(1_i32).expect("1 != 0"));
    #[allow(clippy::print_stdout, clippy::use_debug)]
    {
        println!("{args:?}");
    }
}
