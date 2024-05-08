//! Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.

use std::borrow::Cow;

use nonempty::NonEmpty;

use crate::argumente::einzelargument::EinzelArgument;

/// Darstellung eines Arguments im Hilfe-Text.
///
/// ## English
/// Text-representation of an argument in the help-text.
#[derive(Debug, Clone)]
pub struct Hilfe<'t> {
    /// Darstellen der Syntax zum (de)aktivieren der Flag/setzen des Argument-Wertes.
    ///
    /// ## English
    /// Display for the syntax to set/unset the flag/argument value.
    pub syntax: String,
    /// Hilfe-Text für das Argument.
    ///
    /// ## English
    /// Help-Text for the argument.
    pub hilfe: Option<Cow<'t, str>>,
}

/// Darstellung eines Arguments oder mehrerer Alternativen im Hilfe-Text.
///
/// ## English
/// Text-representation of an argument or several alternatives in the help-text.
#[derive(Debug, Clone)]
pub enum Alternativen<'t> {
    /// Darstellung eines einzelnen Arguments im Hilfe-Text.
    ///
    /// ## English
    /// Text-representation of a singular argument in the help-text.
    EinzelArgument(Hilfe<'t>),
    /// Mehrere als Alternativen geparste Argumente.
    ///
    /// ## English
    /// Multiple arguments parsed as alternatives.
    Alternativen(Box<NonEmpty<Alternativen<'t>>>),
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
    fn erzeuge_hilfe_text<'t, S, Bool, Parse, Anzeige: Fn(&S) -> String>(
        arg: &'t EinzelArgument<'t, S, Bool, Parse, Anzeige>,
        meta_standard: &'t str,
        meta_erlaubte_werte: &'t str,
    ) -> Hilfe<'t>;
}

#[deprecated = "Wird nicht verwendet. Vmtl. wurde vergessen es zu entfernen."]
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub struct Standard;

#[allow(deprecated)]
impl ErzeugeHilfeText for Standard {
    #[inline]
    fn erzeuge_hilfe_text<'t, S, Bool, Parse, Anzeige: Fn(&S) -> String>(
        arg: &'t EinzelArgument<'t, S, Bool, Parse, Anzeige>,
        meta_standard: &'t str,
        meta_erlaubte_werte: &'t str,
    ) -> Hilfe<'t> {
        arg.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
    }
}
