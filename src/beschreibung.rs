//! Beschreibung eines Arguments.

use std::fmt::Display;

#[derive(Debug, Clone)]
/// Beschreibung eines Kommandozeilen-Arguments.
pub struct Beschreibung<T> {
    /// Voller Name, wird nach zwei Minus angegeben "--<lang>".
    pub lang: String,
    /// Kurzer Name, wird nach einem Minus angegeben "-<kurz>".
    /// Falls der Name länger wie ein [unicode_segmentation::grapheme] ist wird parsen nie erfolgreich sein.
    pub kurz: Option<String>,
    /// Im automatischen Hilfetext angezeigte Beschreibung.
    pub hilfe: Option<String>,
    /// Standard-Wert falls kein passendes Kommandozeilen-Argument verwendet wurde.
    pub standard: Option<T>,
}

impl<T: Display> Beschreibung<T> {
    #[inline(always)]
    pub(crate) fn als_string_beschreibung(self) -> (Beschreibung<String>, Option<T>) {
        self.als_string_beschreibung_allgemein(ToString::to_string)
    }
}

impl<T> Beschreibung<T> {
    pub(crate) fn als_string_beschreibung_allgemein(
        self,
        anzeige: impl Fn(&T) -> String,
    ) -> (Beschreibung<String>, Option<T>) {
        let Beschreibung { lang, kurz, hilfe, standard } = self;
        let standard_string = standard.as_ref().map(anzeige);
        (Beschreibung { lang, kurz, hilfe, standard: standard_string }, standard)
    }
}
