//! Ein einzelnes Kommandozeilen-Argument.

use std::ffi::OsString;

use void::Void;

use crate::{
    argumente::{flag::Flag, frühes_beenden::FrühesBeenden, hilfe::Hilfe, wert::Wert},
    ergebnis::Ergebnis,
};

/// Konfiguration eines einzelnen Kommandozeilen-Arguments.
///
/// ## English
/// Configuration of a single command line argument.
#[derive(Debug)]
#[must_use]
pub enum EinzelArgument<'t, T, Fehler> {
    /// Es handelt sich um ein Flag-Argument.
    ///
    /// ## English
    /// It is a flag argument.
    Flag(Flag<'t, T>),

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
    Wert(Wert<'t, T, Fehler>),
}

impl<'t, T, Fehler> From<Flag<'t, T>> for EinzelArgument<'t, T, Fehler> {
    #[inline]
    fn from(flag: Flag<'t, T>) -> Self {
        EinzelArgument::Flag(flag)
    }
}

impl<'t, T, Fehler> From<(FrühesBeenden<'t>, T)> for EinzelArgument<'t, T, Fehler> {
    #[inline]
    fn from((frühes_beenden, wert): (FrühesBeenden<'t>, T)) -> Self {
        EinzelArgument::FrühesBeenden { frühes_beenden, wert }
    }
}

impl<'t, Fehler> From<FrühesBeenden<'t>> for EinzelArgument<'t, (), Fehler> {
    #[inline]
    fn from(frühes_beenden: FrühesBeenden<'t>) -> Self {
        EinzelArgument::FrühesBeenden { frühes_beenden, wert: () }
    }
}

impl<'t, T, Fehler> From<Wert<'t, T, Fehler>> for EinzelArgument<'t, T, Fehler> {
    #[inline]
    fn from(wert: Wert<'t, T, Fehler>) -> Self {
        EinzelArgument::Wert(wert)
    }
}

impl<'t, T> EinzelArgument<'t, T, Void> {
    /// Erzeuge eine [`EinzelArgument::Flag`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`EinzelArgument::Flag`]-variant with sensible type parameters.
    #[inline]
    pub fn flag(flag: Flag<'t, T>) -> Self {
        EinzelArgument::Flag(flag)
    }

    /// Erzeuge eine [`EinzelArgument::FrühesBeenden`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`EinzelArgument::FrühesBeenden`]-variant with sensible type parameters.
    #[inline]
    pub fn frühes_beenden_mit_wert(frühes_beenden: FrühesBeenden<'t>, wert: T) -> Self {
        EinzelArgument::FrühesBeenden { frühes_beenden, wert }
    }
}

impl<'t> EinzelArgument<'t, (), Void> {
    /// Erzeuge eine [`EinzelArgument::FrühesBeenden`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`EinzelArgument::FrühesBeenden`]-variant with sensible type parameters.
    #[inline]
    pub fn frühes_beenden(frühes_beenden: FrühesBeenden<'t>) -> Self {
        EinzelArgument::FrühesBeenden { frühes_beenden, wert: () }
    }
}

impl<'t, T, Fehler> EinzelArgument<'t, T, Fehler> {
    /// Erzeuge eine [`EinzelArgument::Wert`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`EinzelArgument::Wert`]-variant with sensible type parameters.Wert
    #[inline]
    pub fn wert(wert: Wert<'t, T, Fehler>) -> Self {
        EinzelArgument::Wert(wert)
    }
}

impl<'t, T, Fehler> EinzelArgument<'t, T, Fehler> {
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

impl<T, Fehler> EinzelArgument<'_, T, Fehler> {
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
