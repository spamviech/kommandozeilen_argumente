//! Beschreibung eines Arguments.

use std::fmt::Display;

use nonempty::NonEmpty;

/// Beschreibung eines Kommandozeilen-Arguments.
#[derive(Debug, Clone)]
pub struct Beschreibung<T> {
    /// Voller Name, wird nach zwei Minus angegeben "--<lang>".
    pub lang: NonEmpty<String>,
    /// Kurzer Name, wird nach einem Minus angegeben "-<kurz>".
    /// Falls der Name länger wie ein [unicode_segmentation::grapheme] ist wird parsen nie erfolgreich sein.
    pub kurz: Vec<String>,
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

    /// Konvertiere eine Beschreibung zu einem anderen Typ.
    pub fn konvertiere<S>(self, konvertiere: impl FnOnce(T) -> S) -> Beschreibung<S> {
        let Beschreibung { lang, kurz, hilfe, standard } = self;
        Beschreibung { lang, kurz, hilfe, standard: standard.map(konvertiere) }
    }
}

pub trait LangNamen {
    fn lang_namen(self) -> NonEmpty<String>;
}

impl LangNamen for String {
    fn lang_namen(self) -> NonEmpty<String> {
        NonEmpty::singleton(self)
    }
}

impl LangNamen for NonEmpty<String> {
    fn lang_namen(self) -> NonEmpty<String> {
        self
    }
}

pub trait KurzNamen {
    fn kurz_namen(self) -> Vec<String>;
}

impl KurzNamen for Option<String> {
    fn kurz_namen(self) -> Vec<String> {
        self.into_iter().collect()
    }
}

impl KurzNamen for String {
    fn kurz_namen(self) -> Vec<String> {
        vec![self]
    }
}

impl KurzNamen for Vec<String> {
    fn kurz_namen(self) -> Vec<String> {
        self
    }
}

impl<T> Beschreibung<T> {
    pub fn neu(
        lang: impl LangNamen,
        kurz: impl KurzNamen,
        hilfe: Option<String>,
        standard: Option<T>,
    ) -> Beschreibung<T> {
        Beschreibung { lang: lang.lang_namen(), kurz: kurz.kurz_namen(), hilfe, standard }
    }
}
