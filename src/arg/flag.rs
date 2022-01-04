//! Flag-Argumente.

use std::{convert::identity, ffi::OsStr, fmt::Display};

use itertools::Itertools;
use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    arg::{Arg, ArgString},
    beschreibung::ArgBeschreibung,
    ergebnis::{ParseErgebnis, ParseFehler},
};

impl<E> Arg<bool, E> {
    #[inline(always)]
    pub fn flag_deutsch(beschreibung: ArgBeschreibung<bool>) -> Arg<bool, E> {
        Arg::flag(beschreibung, "kein")
    }

    #[inline(always)]
    pub fn flag_english(beschreibung: ArgBeschreibung<bool>) -> Arg<bool, E> {
        Arg::flag(beschreibung, "no")
    }

    #[inline(always)]
    pub fn flag(
        beschreibung: ArgBeschreibung<bool>,
        invertiere_prefix: &'static str,
    ) -> Arg<bool, E> {
        Arg::flag_allgemein(beschreibung, identity, invertiere_prefix)
    }
}

impl<T: 'static + Display + Clone, E> Arg<T, E> {
    #[inline(always)]
    pub fn flag_deutsch_allgemein(
        beschreibung: ArgBeschreibung<T>,
        konvertiere: impl 'static + Fn(bool) -> T,
    ) -> Arg<T, E> {
        Arg::flag_allgemein(beschreibung, konvertiere, "kein")
    }

    #[inline(always)]
    pub fn flag_english_general(
        beschreibung: ArgBeschreibung<T>,
        konvertiere: impl 'static + Fn(bool) -> T,
    ) -> Arg<T, E> {
        Arg::flag_allgemein(beschreibung, konvertiere, "no")
    }

    pub fn flag_allgemein(
        beschreibung: ArgBeschreibung<T>,
        konvertiere: impl 'static + Fn(bool) -> T,
        invertiere_prefix: &'static str,
    ) -> Arg<T, E> {
        let name_kurz = beschreibung.kurz.clone();
        let name_lang = beschreibung.lang.clone();
        let invertiere_prefix_minus = format!("{}-", invertiere_prefix);
        let (beschreibung, standard) = beschreibung.als_string_beschreibung();
        Arg {
            beschreibungen: vec![ArgString::Flag {
                beschreibung,
                invertiere_prefix: Some(invertiere_prefix.to_owned()),
            }],
            flag_kurzformen: name_kurz.iter().cloned().collect(),
            parse: Box::new(move |args| {
                let name_kurz_str = name_kurz.as_ref().map(String::as_str);
                let name_kurz_existiert = name_kurz_str.is_some();
                let mut ergebnis = None;
                let mut nicht_verwendet = Vec::new();
                for arg in args {
                    if let Some(string) = arg.and_then(OsStr::to_str) {
                        if let Some(lang) = string.strip_prefix("--") {
                            if lang == name_lang {
                                ergebnis = Some(konvertiere(true));
                                nicht_verwendet.push(None);
                                continue;
                            } else if let Some(negiert) =
                                lang.strip_prefix(&invertiere_prefix_minus)
                            {
                                if negiert == name_lang {
                                    ergebnis = Some(konvertiere(false));
                                    nicht_verwendet.push(None);
                                    continue;
                                }
                            }
                        } else if name_kurz_existiert {
                            if let Some(kurz) = string.strip_prefix('-') {
                                if kurz.graphemes(true).exactly_one().ok() == name_kurz_str {
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
                    ParseErgebnis::Wert(wert)
                } else if let Some(wert) = &standard {
                    ParseErgebnis::Wert(wert.clone())
                } else {
                    let fehler = ParseFehler::FehlendeFlag {
                        lang: name_lang.clone(),
                        kurz: name_kurz.clone(),
                        invertiere_prefix: invertiere_prefix.to_owned(),
                    };
                    ParseErgebnis::Fehler(NonEmpty::singleton(fehler))
                };
                (ergebnis, nicht_verwendet)
            }),
        }
    }
}
