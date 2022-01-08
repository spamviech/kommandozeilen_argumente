//! Flag-Argumente.

use std::{convert::identity, ffi::OsStr, fmt::Display};

use itertools::Itertools;
use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    argumente::{ArgString, Argumente},
    beschreibung::{contains_str, Beschreibung},
    ergebnis::{Ergebnis, Fehler},
};

impl<E> Argumente<bool, E> {
    #[inline(always)]
    /// Erzeuge ein Flag-Argument, dass mit einem "kein"-Präfix deaktiviert werden kann.
    pub fn flag_deutsch(beschreibung: Beschreibung<bool>) -> Argumente<bool, E> {
        Argumente::flag(beschreibung, "kein")
    }

    #[inline(always)]
    /// Create a flag-argument which can be deactivated with a "no" prefix.
    pub fn flag_english(beschreibung: Beschreibung<bool>) -> Argumente<bool, E> {
        Argumente::flag(beschreibung, "no")
    }

    #[inline(always)]
    /// Erzeuge ein Flag-Argument, dass mit dem konfigurierten Präfix deaktiviert werden kann.
    pub fn flag(
        beschreibung: Beschreibung<bool>,
        invertiere_präfix: &'static str,
    ) -> Argumente<bool, E> {
        Argumente::flag_allgemein(beschreibung, identity, invertiere_präfix)
    }
}

impl<T: 'static + Display + Clone, E> Argumente<T, E> {
    #[inline(always)]
    /// Erzeuge ein Flag-Argument, dass mit einem "kein"-Präfix deaktiviert werden kann.
    pub fn flag_deutsch_allgemein(
        beschreibung: Beschreibung<T>,
        konvertiere: impl 'static + Fn(bool) -> T,
    ) -> Argumente<T, E> {
        Argumente::flag_allgemein(beschreibung, konvertiere, "kein")
    }

    #[inline(always)]
    /// Create a flag-argument which can be deactivated with a "no" prefix.
    pub fn flag_english_general(
        beschreibung: Beschreibung<T>,
        konvertiere: impl 'static + Fn(bool) -> T,
    ) -> Argumente<T, E> {
        Argumente::flag_allgemein(beschreibung, konvertiere, "no")
    }

    /// Erzeuge ein Flag-Argument, dass mit dem konfigurierten Präfix deaktiviert werden kann.
    pub fn flag_allgemein(
        beschreibung: Beschreibung<T>,
        konvertiere: impl 'static + Fn(bool) -> T,
        invertiere_präfix: &'static str,
    ) -> Argumente<T, E> {
        let name_kurz = beschreibung.kurz.clone();
        let name_lang = beschreibung.lang.clone();
        let invertiere_präfix_minus = format!("{}-", invertiere_präfix);
        let (beschreibung, standard) = beschreibung.als_string_beschreibung();
        Argumente {
            beschreibungen: vec![ArgString::Flag {
                beschreibung,
                invertiere_präfix: Some(invertiere_präfix.to_owned()),
            }],
            flag_kurzformen: name_kurz.iter().cloned().collect(),
            parse: Box::new(move |args| {
                let name_kurz_existiert = !name_kurz.is_empty();
                let mut ergebnis = None;
                let mut nicht_verwendet = Vec::new();
                for arg in args {
                    if let Some(string) = arg.and_then(OsStr::to_str) {
                        if let Some(lang) = string.strip_prefix("--") {
                            if contains_str!(&name_lang, lang) {
                                ergebnis = Some(konvertiere(true));
                                nicht_verwendet.push(None);
                                continue;
                            } else if let Some(negiert) =
                                lang.strip_prefix(&invertiere_präfix_minus)
                            {
                                if contains_str!(&name_lang, negiert) {
                                    ergebnis = Some(konvertiere(false));
                                    nicht_verwendet.push(None);
                                    continue;
                                }
                            }
                        } else if name_kurz_existiert {
                            if let Some(kurz) = string.strip_prefix('-') {
                                if kurz
                                    .graphemes(true)
                                    .exactly_one()
                                    .map(|name| contains_str!(&name_kurz, name))
                                    .unwrap_or(false)
                                {
                                    ergebnis = Some(konvertiere(true));
                                    nicht_verwendet.push(None);
                                    continue;
                                }
                            }
                        }
                    }
                    nicht_verwendet.push(arg);
                }
                let ergebnis = if let Some(wert) = ergebnis {
                    Ergebnis::Wert(wert)
                } else if let Some(wert) = &standard {
                    Ergebnis::Wert(wert.clone())
                } else {
                    let fehler = Fehler::FehlendeFlag {
                        lang: name_lang.clone(),
                        kurz: name_kurz.clone(),
                        invertiere_präfix: invertiere_präfix.to_owned(),
                    };
                    Ergebnis::Fehler(NonEmpty::singleton(fehler))
                };
                (ergebnis, nicht_verwendet)
            }),
        }
    }
}
