//! Flag-Argumente.

use std::{borrow::Cow, convert::identity, ffi::OsStr, fmt::Display, ops::Deref};

use itertools::Itertools;
use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    argumente::{Argumente, Arguments},
    beschreibung::{contains_str, Beschreibung, Description, Konfiguration},
    ergebnis::{Ergebnis, Fehler},
    sprache::{Language, Sprache},
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
        invertiere_präfix: impl Into<Cow<'t, str>>,
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
        invertiere_präfix: impl Into<Cow<'t, str>>,
    ) -> Argumente<'t, T, E> {
        let name_kurz: Vec<_> =
            beschreibung.kurz.iter().map(|cow| cow.deref().to_owned()).collect();
        let name_lang = {
            let NonEmpty { head, tail } = beschreibung.lang;
            NonEmpty {
                head: head.deref().to_owned(),
                tail: tail.iter().map(|cow| cow.deref().to_owned()).collect(),
            }
        };
        let invertiere_präfix_cow = invertiere_präfix.into();
        let invertiere_präfix_string = invertiere_präfix_cow.deref().to_owned();
        let invertiere_präfix_minus = format!("{}-", invertiere_präfix_cow);
        let (beschreibung, standard) = beschreibung.als_string_beschreibung();
        Argumente {
            konfigurationen: vec![Konfiguration::Flag {
                beschreibung,
                invertiere_präfix: Some(invertiere_präfix_cow),
            }],
            flag_kurzformen: name_kurz.iter().map(|s| Cow::Borrowed(s.as_str())).collect(),
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
                        lang: {
                            let NonEmpty { head, tail } = &name_lang;
                            NonEmpty {
                                head: Cow::Owned(head.clone()),
                                tail: tail.iter().cloned().map(Cow::Owned).collect(),
                            }
                        },
                        kurz: name_kurz.iter().cloned().map(Cow::Owned).collect(),
                        invertiere_präfix: Cow::Owned(invertiere_präfix_string),
                    };
                    Ergebnis::Fehler(NonEmpty::singleton(fehler))
                };
                (ergebnis, nicht_verwendet)
            }),
        }
    }
}
