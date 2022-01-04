//! Beschreibung eines Arguments.

use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Beschreibung<T> {
    pub lang: String,
    pub kurz: Option<String>,
    pub hilfe: Option<String>,
    pub standard: Option<T>,
}

impl<T: Display> Beschreibung<T> {
    pub(crate) fn als_string_beschreibung(self) -> (Beschreibung<String>, Option<T>) {
        let Beschreibung { lang, kurz, hilfe, standard } = self;
        let standard_string = standard.as_ref().map(ToString::to_string);
        (Beschreibung { lang, kurz, hilfe, standard: standard_string }, standard)
    }
}
