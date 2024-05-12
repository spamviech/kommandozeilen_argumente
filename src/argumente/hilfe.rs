//! Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.

use nonempty::NonEmpty;

use crate::argumente::einzelargument::EinzelArgument;

/// Darstellung eines Arguments im Hilfe-Text.
///
/// ## English
/// Text-representation of an argument in the help-text.
#[derive(Debug, Clone)]
pub struct Hilfe {
    /// Darstellen der Syntax zum (de)aktivieren der Flag/setzen des Argument-Wertes.
    ///
    /// ## English
    /// Display for the syntax to set/unset the flag/argument value.
    pub syntax: String,
    /// Hilfe-Text für das Argument.
    ///
    /// ## English
    /// Help-Text for the argument.
    pub hilfe: Option<String>,
}

/// Darstellung eines Arguments oder mehrerer Alternativen im Hilfe-Text.
///
/// ## English
/// Text-representation of an argument or several alternatives in the help-text.
#[derive(Debug, Clone)]
pub enum Alternativen {
    /// Darstellung eines einzelnen Arguments im Hilfe-Text.
    ///
    /// ## English
    /// Text-representation of a singular argument in the help-text.
    EinzelArgument(Hilfe),
    /// Mehrere als Alternativen geparste Argumente.
    ///
    /// ## English
    /// Multiple arguments parsed as alternatives.
    Alternativen(Box<NonEmpty<Alternativen>>),
    /// Fester Wert ohne assoziiertes Argument. Wird nicht im Hilfetext angezeigt.
    ///
    /// ## English
    /// Fixed value without associated argument. Wird nicht im Hilfetext angezeigt.
    Leer,
}

/// Trait zum simulieren einer Rank-2 Funktion.
///
/// # English
/// Trait to simulate a rank-2 function.
pub trait ErzeugeHilfeText {
    /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
    ///
    /// ## English
    /// Create the Message for the syntax of the arguments and the corresponding help text.
    fn erzeuge_hilfe_text(
        &self,
        arg: EinzelArgument<'_, String, String>,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> Hilfe;
}

/// Standard-Variante den Hilfe-Text für ein einzelnes Argument zu erzeugen, z.B.:
/// `  --hilfe | -h    Zeige diesen Text an.`
///
/// ## English synonym
/// [`Default`]
#[derive(Debug, Clone, Copy)]
pub struct Standard;

#[allow(deprecated)]
impl ErzeugeHilfeText for Standard {
    #[inline]
    fn erzeuge_hilfe_text(
        &self,
        arg: EinzelArgument<'_, String, String>,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> Hilfe {
        arg.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
    }
}

/// Default-Variant to create the help-text for a single argument, e.g.:
/// `  --help | -h    Show this text.`
///
/// ## Deutsches Synonym
/// [`Standard`]
#[derive(Debug, Clone, Copy)]
pub struct Default;

#[allow(deprecated)]
impl ErzeugeHilfeText for Default {
    #[inline]
    fn erzeuge_hilfe_text(
        &self,
        arg: EinzelArgument<'_, String, String>,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> Hilfe {
        Standard.erzeuge_hilfe_text(arg, meta_standard, meta_erlaubte_werte)
    }
}
