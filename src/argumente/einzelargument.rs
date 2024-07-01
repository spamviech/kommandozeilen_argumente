//! Ein einzelnes Kommandozeilen-Argument.

use std::{
    borrow::Cow,
    fmt::{self, Debug, Display},
};

use void::Void;

use crate::{
    argumente::{flag::Flag, frühes_beenden::FrühesBeenden, hilfe::Hilfe, wert::Wert},
    beschreibung::ArgumentInput,
    dyn_to_owned::Anzeige,
    ergebnis::Ergebnis,
};

/// Konfiguration eines einzelnen Kommandozeilen-Arguments.
///
/// ## English
/// Configuration of a single command line argument.
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
        /// Anzeige eines Wertes (default value).
        ///
        /// ## English
        /// Display a value (default value).
        anzeige: Cow<'t, dyn Anzeige<'t, T>>,
    },

    /// Es handelt sich um ein Wert-Argument.
    ///
    /// ## English
    /// It is a value argument.
    Wert(Wert<'t, T, Fehler>),
}

impl<T: Debug, Fehler: Debug> Debug for EinzelArgument<'_, T, Fehler> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flag(arg0) => formatter.debug_tuple("Flag").field(arg0).finish(),
            Self::FrühesBeenden { frühes_beenden, wert, anzeige: _ } => formatter
                .debug_struct("FrühesBeenden")
                .field("frühes_beenden", frühes_beenden)
                .field("wert", wert)
                .field("anzeige", &"<closure>")
                .finish(),
            Self::Wert(arg0) => formatter.debug_tuple("Wert").field(arg0).finish(),
        }
    }
}

impl<'t, T, Fehler> From<Flag<'t, T>> for EinzelArgument<'t, T, Fehler> {
    #[inline]
    fn from(flag: Flag<'t, T>) -> Self {
        EinzelArgument::Flag(flag)
    }
}

impl<'t, T: Display, Fehler> From<(FrühesBeenden<'t>, T)> for EinzelArgument<'t, T, Fehler> {
    #[inline]
    fn from((frühes_beenden, wert): (FrühesBeenden<'t>, T)) -> Self {
        EinzelArgument::FrühesBeenden {
            frühes_beenden,
            wert,
            anzeige: Cow::Borrowed(&ToString::to_string),
        }
    }
}

impl<'t, Fehler> From<FrühesBeenden<'t>> for EinzelArgument<'t, (), Fehler> {
    #[inline]
    fn from(frühes_beenden: FrühesBeenden<'t>) -> Self {
        EinzelArgument::FrühesBeenden {
            frühes_beenden,
            wert: (),
            anzeige: Cow::Borrowed(&|wert| format!("{wert:?}")),
        }
    }
}

impl<'t, T, Fehler> From<Wert<'t, T, Fehler>> for EinzelArgument<'t, T, Fehler> {
    #[inline]
    fn from(wert: Wert<'t, T, Fehler>) -> Self {
        EinzelArgument::Wert(wert)
    }
}

impl<'t, T: Display> EinzelArgument<'t, T, Void> {
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
        EinzelArgument::FrühesBeenden {
            frühes_beenden,
            wert,
            anzeige: Cow::Borrowed(&ToString::to_string),
        }
    }
}

impl<'t> EinzelArgument<'t, (), Void> {
    /// Erzeuge eine [`EinzelArgument::FrühesBeenden`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`EinzelArgument::FrühesBeenden`]-variant with sensible type parameters.
    #[inline]
    pub fn frühes_beenden(frühes_beenden: FrühesBeenden<'t>) -> Self {
        EinzelArgument::FrühesBeenden {
            frühes_beenden,
            wert: (),
            anzeige: Cow::Borrowed(&|wert| format!("{wert:?}")),
        }
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
        args: impl Iterator<Item = Option<ArgumentInput>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<ArgumentInput>>) {
        match self {
            EinzelArgument::Flag(flag) => flag.parse(args),
            EinzelArgument::FrühesBeenden { frühes_beenden, wert, anzeige: _ } => {
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
    pub fn erzeuge_hilfe_text(&self, meta_standard: &str, meta_erlaubte_werte: &str) -> Hilfe {
        match self {
            EinzelArgument::Flag(flag) => flag.erzeuge_hilfe_text(meta_standard),
            EinzelArgument::FrühesBeenden { frühes_beenden, wert: _, anzeige: _ } => {
                frühes_beenden.erzeuge_hilfe_text()
            },
            EinzelArgument::Wert(wert) => {
                wert.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
            },
        }
    }

    /// Konvertiere den Wert und Fehler der parse-Funktion in einen String,
    /// unter Zuhilfenahme der jeweiligen `anzeige*`-Funktionen.
    ///
    /// ## English
    /// Convert the value and error of the parse-function to a string, using the respective `anzeige*`-function.
    #[inline]
    pub fn als_string_wert(&self) -> EinzelArgument<'_, String, String> {
        match self {
            EinzelArgument::Flag(flag) => EinzelArgument::Flag(flag.als_string_flag()),
            EinzelArgument::FrühesBeenden { frühes_beenden, wert, anzeige } => {
                EinzelArgument::FrühesBeenden {
                    frühes_beenden: frühes_beenden.clone(),
                    wert: anzeige(wert),
                    anzeige: Cow::Borrowed(&Clone::clone),
                }
            },
            EinzelArgument::Wert(wert) => EinzelArgument::Wert(wert.als_string_wert()),
        }
    }
}
