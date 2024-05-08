//! Ein einzelnes Kommandozeilen-Argument.

use std::ffi::{OsStr, OsString};

use void::Void;

use crate::{
    argumente::{flag::Flag, frühes_beenden::FrühesBeenden, hilfe::Hilfe, wert::Wert},
    ergebnis::{Ergebnis, ParseFehler},
};

/// Konfiguration eines einzelnen Kommandozeilen-Arguments.
///
/// ## English
/// Configuration of a single command line argument.
#[derive(Debug)]
#[must_use]
pub enum EinzelArgument<
    't,
    T,
    Bool = fn(bool) -> T,
    Parse = fn(&OsStr) -> Result<T, ParseFehler<Void>>,
    Anzeige = fn(&T) -> String,
> {
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

impl<'t, T, Bool, Parse, Anzeige> From<Flag<'t, T, Bool, Anzeige>>
    for EinzelArgument<'t, T, Bool, Parse, Anzeige>
{
    #[inline]
    fn from(flag: Flag<'t, T, Bool, Anzeige>) -> Self {
        EinzelArgument::Flag(flag)
    }
}

impl<'t, T, Bool, Parse, Anzeige> From<(FrühesBeenden<'t>, T)>
    for EinzelArgument<'t, T, Bool, Parse, Anzeige>
{
    #[inline]
    fn from((frühes_beenden, wert): (FrühesBeenden<'t>, T)) -> Self {
        EinzelArgument::FrühesBeenden { frühes_beenden, wert }
    }
}

impl<'t, Bool, Parse, Anzeige> From<FrühesBeenden<'t>>
    for EinzelArgument<'t, (), Bool, Parse, Anzeige>
{
    #[inline]
    fn from(frühes_beenden: FrühesBeenden<'t>) -> Self {
        EinzelArgument::FrühesBeenden { frühes_beenden, wert: () }
    }
}

impl<'t, T, Bool, Parse, Anzeige> From<Wert<'t, T, Parse, Anzeige>>
    for EinzelArgument<'t, T, Bool, Parse, Anzeige>
{
    #[inline]
    fn from(wert: Wert<'t, T, Parse, Anzeige>) -> Self {
        EinzelArgument::Wert(wert)
    }
}

impl<'t, T, Bool, Anzeige>
    EinzelArgument<'t, T, Bool, fn(&OsStr) -> Result<T, ParseFehler<Void>>, Anzeige>
{
    /// Erzeuge eine [`EinzelArgument::Flag`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`EinzelArgument::Flag`]-variant with sensible type parameters.
    #[inline]
    pub fn flag(flag: Flag<'t, T, Bool, Anzeige>) -> Self {
        EinzelArgument::Flag(flag)
    }
}

impl<'t, T>
    EinzelArgument<
        't,
        T,
        fn(bool) -> T,
        fn(&OsStr) -> Result<T, ParseFehler<Void>>,
        fn(&T) -> String,
    >
{
    /// Erzeuge eine [`EinzelArgument::FrühesBeenden`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`EinzelArgument::FrühesBeenden`]-variant with sensible type parameters.
    #[inline]
    pub fn frühes_beenden_mit_wert(frühes_beenden: FrühesBeenden<'t>, wert: T) -> Self {
        EinzelArgument::FrühesBeenden { frühes_beenden, wert }
    }
}

impl<'t>
    EinzelArgument<
        't,
        (),
        fn(bool) -> (),
        fn(&OsStr) -> Result<(), ParseFehler<Void>>,
        fn(&()) -> String,
    >
{
    /// Erzeuge eine [`EinzelArgument::FrühesBeenden`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`EinzelArgument::FrühesBeenden`]-variant with sensible type parameters.
    #[inline]
    pub fn frühes_beenden(frühes_beenden: FrühesBeenden<'t>) -> Self {
        EinzelArgument::FrühesBeenden { frühes_beenden, wert: () }
    }
}

impl<'t, T, Parse, Anzeige> EinzelArgument<'t, T, fn(bool) -> T, Parse, Anzeige> {
    /// Erzeuge eine [`EinzelArgument::Wert`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`EinzelArgument::Wert`]-variant with sensible type parameters.Wert
    #[inline]
    pub fn wert(wert: Wert<'t, T, Parse, Anzeige>) -> Self {
        EinzelArgument::Wert(wert)
    }
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
    #[inline]
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
    /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
    ///
    /// ## English
    /// Create the Message for the syntax of the arguments and the corresponding help text.
    #[inline]
    pub fn erzeuge_hilfe_text(&self, meta_standard: &str, meta_erlaubte_werte: &str) -> Hilfe<'_> {
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
