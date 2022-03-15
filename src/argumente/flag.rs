//! Flag-Argumente.

use std::{
    convert::{identity, AsRef},
    ffi::OsStr,
    fmt::Display,
};

use itertools::Itertools;
use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    argumente::{Argumente, Arguments},
    beschreibung::{contains_str, Beschreibung, Description, Konfiguration},
    ergebnis::{Ergebnis, Fehler},
    sprache::{Language, Sprache},
    unicode::Normalisiert,
};

impl<'t, E> Argumente<'t, bool, E> {
    /// Erzeuge ein Flag-Argument, dass mit einem "kein"-Präfix deaktiviert werden kann.
    ///
    /// ## English version
    /// [flag_bool_english](Arguments::flag_bool_english)
    #[inline(always)]
    pub fn flag_bool_deutsch(beschreibung: Beschreibung<'t, bool>) -> Argumente<'t, bool, E> {
        Argumente::flag_bool_mit_sprache(beschreibung, Sprache::DEUTSCH)
    }

    /// Create a flag-argument which can be deactivated with a "no" prefix.
    ///
    /// ## Deutsche Version
    /// [flag_bool_deutsch](Argumente::flag_bool_deutsch)
    #[inline(always)]
    pub fn flag_bool_english(description: Description<'t, bool>) -> Arguments<'t, bool, E> {
        Argumente::flag_bool_mit_sprache(description, Sprache::ENGLISH)
    }

    /// Erzeuge ein Flag-Argument, dass mit dem konfigurierten Präfix deaktiviert werden kann.
    ///
    /// ## English synonym
    /// [flag_bool_with_language](Arguments::flag_bool_with_language)
    #[inline(always)]
    pub fn flag_bool_mit_sprache(
        beschreibung: Beschreibung<'t, bool>,
        sprache: Sprache,
    ) -> Argumente<'t, bool, E> {
        Argumente::flag_bool(beschreibung, sprache.invertiere_präfix)
    }

    /// Create a flag-argument which can be deactivated with the configured prefix.
    ///
    /// ## Deutsches Synonym
    /// [flag_bool_mit_sprache](Argumente::flag_bool_mit_sprache)
    #[inline(always)]
    pub fn flag_bool_with_language(
        description: Description<'t, bool>,
        language: Language,
    ) -> Arguments<'t, bool, E> {
        Argumente::flag_bool_mit_sprache(description, language)
    }

    /// Erzeuge ein Flag-Argument, dass mit dem konfigurierten Präfix deaktiviert werden kann.
    ///
    /// ## English
    /// Create a flag-argument which can be deactivated with the configured prefix.
    #[inline(always)]
    pub fn flag_bool(
        beschreibung: Beschreibung<'t, bool>,
        invertiere_präfix: &'t str,
    ) -> Argumente<'t, bool, E> {
        Argumente::flag(beschreibung, identity, invertiere_präfix)
    }
}

impl<'t, T: 'static + Display + Clone, E> Argumente<'t, T, E> {
    /// Erzeuge ein Flag-Argument, dass mit einem "kein"-Präfix deaktiviert werden kann.
    ///
    /// ## English version
    /// [flag_english](Arguments::flag_english)
    #[inline(always)]
    pub fn flag_deutsch(
        beschreibung: Beschreibung<'t, T>,
        konvertiere: impl 'static + Fn(bool) -> T,
    ) -> Argumente<'t, T, E> {
        Argumente::flag_mit_sprache(beschreibung, konvertiere, Sprache::DEUTSCH)
    }

    /// Create a flag-argument which can be deactivated with a "no" prefix.
    ///
    /// ## Deutsche Version
    /// [flag_deutsch](Argumente::flag_deutsch)
    #[inline(always)]
    pub fn flag_english(
        description: Description<'t, T>,
        convert: impl 'static + Fn(bool) -> T,
    ) -> Argumente<'t, T, E> {
        Argumente::flag_mit_sprache(description, convert, Sprache::ENGLISH)
    }

    /// Erzeuge ein Flag-Argument, dass mit dem konfigurierten Präfix deaktiviert werden kann.
    ///
    /// ## English synonym
    /// [flag_with_language](Arguments::flag_with_language)
    #[inline(always)]
    pub fn flag_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        konvertiere: impl 'static + Fn(bool) -> T,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::flag(beschreibung, konvertiere, sprache.invertiere_präfix)
    }

    /// Create a flag-argument which can be deactivated with a "no" prefix.
    ///
    /// ## Deutsches Synonym
    /// [flag_mit_sprache](Argumente::flag_mit_sprache)
    #[inline(always)]
    pub fn flag_with_language(
        description: Description<'t, T>,
        convert: impl 'static + Fn(bool) -> T,
        language: Language,
    ) -> Arguments<'t, T, E> {
        Argumente::flag_mit_sprache(description, convert, language)
    }

    /// Erzeuge ein Flag-Argument, dass mit dem konfigurierten Präfix deaktiviert werden kann.
    ///
    /// ## English
    /// Create a flag-argument which can be deactivated with the configured prefix.
    pub fn flag(
        beschreibung: Beschreibung<'t, T>,
        konvertiere: impl 'static + Fn(bool) -> T,
        invertiere_präfix: &'t str,
    ) -> Argumente<'t, T, E> {
        let name_kurz = beschreibung.kurz.clone();
        let flag_kurzformen = beschreibung.kurz.clone();
        let name_lang = beschreibung.lang.clone();
        let invertiere_präfix_normalisiert = Normalisiert::neu(invertiere_präfix);
        let invertiere_präfix_str = invertiere_präfix_normalisiert.as_ref();
        let invertiere_präfix_minus = format!("{invertiere_präfix_str}-");
        let (beschreibung, standard) = beschreibung.als_string_beschreibung();
        // TODO
        let case_sensitive = false;
        Argumente {
            konfigurationen: vec![Konfiguration::Flag {
                beschreibung,
                invertiere_präfix: Some(invertiere_präfix_normalisiert.clone()),
            }],
            flag_kurzformen,
            parse: Box::new(move |args| {
                let name_kurz_existiert = !name_kurz.is_empty();
                let mut ergebnis = None;
                let mut nicht_verwendet = Vec::new();
                for arg in args {
                    if let Some(string) = arg.and_then(OsStr::to_str) {
                        if let Some(lang) = string.strip_prefix("--") {
                            if contains_str!(&name_lang, lang, case_sensitive) {
                                ergebnis = Some(konvertiere(true));
                                nicht_verwendet.push(None);
                                continue;
                            } else if let Some(negiert) =
                                lang.strip_prefix(&invertiere_präfix_minus)
                            {
                                if contains_str!(&name_lang, negiert, case_sensitive) {
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
                                    .map(|name| contains_str!(&name_kurz, name, case_sensitive))
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
                        invertiere_präfix: invertiere_präfix_normalisiert.clone(),
                    };
                    Ergebnis::Fehler(NonEmpty::singleton(fehler))
                };
                (ergebnis, nicht_verwendet)
            }),
        }
    }
}
