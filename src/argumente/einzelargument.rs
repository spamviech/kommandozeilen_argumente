//! Ein einzelnes Kommandozeilen-Argument.

use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
};

use crate::{
    argumente::{flag::Flag, frühes_beenden::FrühesBeenden, wert::Wert},
    ergebnis::{Ergebnis, ParseFehler},
};

/// Konfiguration eines einzelnen Kommandozeilen-Arguments.
///
/// ## English
/// Configuration of a single command line argument.
#[derive(Debug)]
pub enum EinzelArgument<'t, T, Bool, Parse, Anzeige> {
    /// Es handelt sich um ein Flag-Argument.
    ///
    /// ## English
    /// It is a flag argument.
    Flag(Flag<'t, T, Bool, Anzeige>),

    /// Es handelt sich um ein Flag-Argument, das zu frühem beenden führt.
    ///
    /// ## English
    /// It is a flag argument, causing an early exit.
    FrühesBeenden {
        /// Die Flag und angezeigte Nachricht.
        ///
        /// ## English
        /// The flag and displayed message.
        frühes_beenden: FrühesBeenden<'t>,
        /// Der Wert, wenn die Flag nicht übergeben wird.
        ///
        /// ## English
        /// The value if the flag was missing.
        wert: T,
    },

    /// Es handelt sich um ein Wert-Argument.
    ///
    /// ## English
    /// It is a value argument.
    Wert(Wert<'t, T, Parse, Anzeige>),
}

impl<'t, T, Bool, Parse, Fehler, Anzeige> EinzelArgument<'t, T, Bool, Parse, Anzeige>
where
    Bool: Fn(bool) -> T,
    Parse: Fn(&OsStr) -> Result<T, ParseFehler<Fehler>>,
{
    /// Parse die übergebenen Argumente und erzeuge den zugehörigen Wert.
    ///
    /// ## English
    /// Parse the given arguments and return the corresponding value.
    pub fn parse(
        self,
        args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>) {
        match self {
            EinzelArgument::Flag(flag) => flag.parse(args),
            EinzelArgument::FrühesBeenden { frühes_beenden, wert } => {
                let (ergebnis, nicht_verwendet) = frühes_beenden.parse(args);
                (ergebnis.konvertiere(|()| wert), nicht_verwendet)
            },
            EinzelArgument::Wert(wert) => wert.parse(args),
        }
    }
}

impl<T, Bool, Parse, Anzeige> EinzelArgument<'_, T, Bool, Parse, Anzeige>
where
    Anzeige: Fn(&T) -> String,
{
    // TODO [Sprache::standard] kann als meta_standard verwendet werden.
    /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
    ///
    /// ## English
    /// Create the Message for the syntax of the arguments and the corresponding help text.
    pub fn erzeuge_hilfe_text(
        &self,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> (String, Option<Cow<'_, str>>) {
        match self {
            EinzelArgument::Flag(flag) => flag.erzeuge_hilfe_text(meta_standard),
            EinzelArgument::FrühesBeenden { frühes_beenden, wert: _ } => {
                frühes_beenden.erzeuge_hilfe_text()
            },
            EinzelArgument::Wert(wert) => {
                wert.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
            },
        }
    }
}
