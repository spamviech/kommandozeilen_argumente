//! Flag-Argumente.

use std::{borrow::Cow, convert::identity, ffi::OsString, fmt::Display, iter};

use itertools::Itertools;
use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    argumente::{Argumente, Arguments},
    beschreibung::{contains_str, Beschreibung, Description, Konfiguration, Name},
    ergebnis::{Ergebnis, Fehler},
    sprache::{Language, Sprache},
    unicode::{Normalisiert, Vergleich},
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
        Argumente::flag_bool(beschreibung, sprache.invertiere_präfix, sprache.invertiere_infix)
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
        invertiere_präfix: impl Into<Vergleich<'t>>,
        invertiere_infix: impl Into<Vergleich<'t>>,
    ) -> Argumente<'t, bool, E> {
        Argumente::flag_display(beschreibung, identity, invertiere_präfix, invertiere_infix)
    }
}

impl<'t, T: 't + Display + Clone, E> Argumente<'t, T, E> {
    /// Erzeuge ein Flag-Argument, dass mit einem "kein"-Präfix deaktiviert werden kann.
    ///
    /// ## English version
    /// [flag_english](Arguments::flag_english)
    #[inline(always)]
    pub fn flag_display_deutsch(
        beschreibung: Beschreibung<'t, T>,
        konvertiere: impl 't + Fn(bool) -> T,
    ) -> Argumente<'t, T, E> {
        Argumente::flag_display_mit_sprache(beschreibung, konvertiere, Sprache::DEUTSCH)
    }

    /// Create a flag-argument which can be deactivated with a "no" prefix.
    ///
    /// ## Deutsche Version
    /// [flag_deutsch](Argumente::flag_deutsch)
    #[inline(always)]
    pub fn flag_display_english(
        description: Description<'t, T>,
        convert: impl 't + Fn(bool) -> T,
    ) -> Argumente<'t, T, E> {
        Argumente::flag_display_mit_sprache(description, convert, Sprache::ENGLISH)
    }

    /// Erzeuge ein Flag-Argument, dass mit dem konfigurierten Präfix deaktiviert werden kann.
    ///
    /// ## English synonym
    /// [flag_with_language](Arguments::flag_with_language)
    #[inline(always)]
    pub fn flag_display_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        konvertiere: impl 't + Fn(bool) -> T,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::flag(
            beschreibung,
            konvertiere,
            sprache.invertiere_präfix,
            sprache.invertiere_infix,
            ToString::to_string,
        )
    }

    /// Create a flag-argument which can be deactivated with a "no" prefix.
    ///
    /// ## Deutsches Synonym
    /// [flag_mit_sprache](Argumente::flag_mit_sprache)
    #[inline(always)]
    pub fn flag_display_with_language(
        description: Description<'t, T>,
        convert: impl 't + Fn(bool) -> T,
        language: Language,
    ) -> Arguments<'t, T, E> {
        Argumente::flag_display_mit_sprache(description, convert, language)
    }

    /// Erzeuge ein Flag-Argument, dass mit dem konfigurierten Präfix deaktiviert werden kann.
    ///
    /// ## English
    /// Create a flag-argument which can be deactivated with the configured prefix.
    #[inline(always)]
    pub fn flag_display(
        beschreibung: Beschreibung<'t, T>,
        konvertiere: impl 't + Fn(bool) -> T,
        invertiere_präfix: impl Into<Vergleich<'t>>,
        invertiere_infix: impl Into<Vergleich<'t>>,
    ) -> Argumente<'t, T, E> {
        Argumente::flag(
            beschreibung,
            konvertiere,
            invertiere_präfix,
            invertiere_infix,
            ToString::to_string,
        )
    }
}

impl<'t, T: 't + Clone, E> Argumente<'t, T, E> {
    /// Erzeuge ein Flag-Argument, dass mit einem "kein"-Präfix deaktiviert werden kann.
    ///
    /// ## English version
    /// [flag_english](Arguments::flag_english)
    #[inline(always)]
    pub fn flag_deutsch(
        beschreibung: Beschreibung<'t, T>,
        konvertiere: impl 't + Fn(bool) -> T,
        anzeige: impl Fn(&T) -> String,
    ) -> Argumente<'t, T, E> {
        Argumente::flag_mit_sprache(beschreibung, konvertiere, anzeige, Sprache::DEUTSCH)
    }

    /// Create a flag-argument which can be deactivated with a "no" prefix.
    ///
    /// ## Deutsche Version
    /// [flag_deutsch](Argumente::flag_deutsch)
    #[inline(always)]
    pub fn flag_english(
        description: Description<'t, T>,
        convert: impl 't + Fn(bool) -> T,
        display: impl Fn(&T) -> String,
    ) -> Argumente<'t, T, E> {
        Argumente::flag_mit_sprache(description, convert, display, Sprache::ENGLISH)
    }

    /// Erzeuge ein Flag-Argument, dass mit dem konfigurierten Präfix deaktiviert werden kann.
    ///
    /// ## English synonym
    /// [flag_with_language](Arguments::flag_with_language)
    #[inline(always)]
    pub fn flag_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        konvertiere: impl 't + Fn(bool) -> T,
        anzeige: impl Fn(&T) -> String,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::flag(
            beschreibung,
            konvertiere,
            sprache.invertiere_präfix,
            sprache.invertiere_infix,
            anzeige,
        )
    }

    /// Create a flag-argument which can be deactivated with a "no" prefix.
    ///
    /// ## Deutsches Synonym
    /// [flag_mit_sprache](Argumente::flag_mit_sprache)
    #[inline(always)]
    pub fn flag_with_language(
        description: Description<'t, T>,
        convert: impl 't + Fn(bool) -> T,
        display: impl Fn(&T) -> String,
        language: Language,
    ) -> Arguments<'t, T, E> {
        Argumente::flag_mit_sprache(description, convert, display, language)
    }

    /// Erzeuge ein Flag-Argument, dass mit dem konfigurierten Präfix deaktiviert werden kann.
    ///
    /// ## English
    /// Create a flag-argument which can be deactivated with the configured prefix.
    pub fn flag(
        beschreibung: Beschreibung<'t, T>,
        konvertiere: impl 't + Fn(bool) -> T,
        invertiere_präfix: impl Into<Vergleich<'t>>,
        invertiere_infix: impl Into<Vergleich<'t>>,
        anzeige: impl Fn(&T) -> String,
    ) -> Argumente<'t, T, E> {
        let name_lang_präfix = beschreibung.name.lang_präfix.clone();
        let name_lang = beschreibung.name.lang.clone();
        let name_kurz_präfix = beschreibung.name.kurz_präfix.clone();
        let name_kurz = beschreibung.name.kurz.clone();
        let flag_kurzformen =
            iter::once((beschreibung.name.kurz_präfix.clone(), beschreibung.name.kurz.clone()))
                .collect();
        let invertiere_präfix_vergleich = invertiere_präfix.into();
        let invertiere_infix_vergleich = invertiere_infix.into();
        let (beschreibung, standard) = beschreibung.als_string_beschreibung_allgemein(anzeige);
        Argumente {
            konfigurationen: vec![Konfiguration::Flag {
                beschreibung,
                invertiere_präfix_infix: Some((
                    invertiere_präfix_vergleich.clone(),
                    invertiere_infix_vergleich.clone(),
                )),
            }],
            flag_kurzformen,
            parse: Box::new(move |args| {
                let name_kurz_existiert = !name_kurz.is_empty();
                let mut ergebnis = None;
                let mut nicht_verwendet = Vec::new();
                for arg in args {
                    if let Some(string) = arg.as_ref().and_then(|os_string| os_string.to_str()) {
                        let normalisiert = Normalisiert::neu(string);
                        if let Some(lang_str) = name_lang_präfix.strip_als_präfix(&normalisiert) {
                            if contains_str(&name_lang, lang_str) {
                                ergebnis = Some(konvertiere(true));
                                nicht_verwendet.push(None);
                                continue;
                            } else if let Some(infix_name) = invertiere_präfix_vergleich
                                .strip_als_präfix(&Normalisiert::neu_borrowed_unchecked(lang_str))
                            {
                                let infix_name_normalisiert =
                                    Normalisiert::neu_borrowed_unchecked(infix_name);
                                if let Some(negiert) = invertiere_infix_vergleich
                                    .strip_als_präfix(&infix_name_normalisiert)
                                {
                                    if contains_str(&name_lang, negiert) {
                                        ergebnis = Some(konvertiere(false));
                                        nicht_verwendet.push(None);
                                        continue;
                                    }
                                }
                            }
                        } else if name_kurz_existiert {
                            if let Some(kurz_graphemes) =
                                name_kurz_präfix.strip_als_präfix(&normalisiert)
                            {
                                if kurz_graphemes
                                    .graphemes(true)
                                    .exactly_one()
                                    .map(|name| contains_str(&name_kurz, name))
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
                        name: Name {
                            lang_präfix: name_lang_präfix.clone(),
                            lang: name_lang.clone(),
                            kurz_präfix: name_kurz_präfix.clone(),
                            kurz: name_kurz.clone(),
                        },
                        invertiere_präfix: invertiere_präfix_vergleich.string.clone(),
                        invertiere_infix: invertiere_infix_vergleich.string.clone(),
                    };
                    Ergebnis::Fehler(NonEmpty::singleton(fehler))
                };
                (ergebnis, nicht_verwendet)
            }),
        }
    }
}

// TODO Hilfs-funktionen zum einfachren erstellen (mit_sprache, etc.)
/// Es handelt sich um ein Flag-Argument.
///
/// ## English
/// It is a flag argument.
#[derive(Debug)]
pub struct Flag<'t, T, Bool, Anzeige> {
    /// Allgemeine Beschreibung des Arguments.
    ///
    /// ## English
    /// General description of the argument.
    pub beschreibung: Beschreibung<'t, T>,

    /// Präfix invertieren des Flag-Arguments.
    ///
    /// ## English
    /// Prefix to invert the flag argument.
    pub invertiere_präfix: Vergleich<'t>,

    /// Infix zum invertieren des Flag-Arguments.
    ///
    /// ## English
    /// Infix to invert the flag argument.
    pub invertiere_infix: Vergleich<'t>,

    /// Erzeuge einen Wert aus einer [bool].
    ///
    /// ## English
    /// Create a value from a [bool].
    pub konvertiere: Bool,

    /// Anzeige eines Wertes (default value).
    ///
    /// ## English
    /// Display a value (default value).
    pub anzeige: Anzeige,
}

impl<'t, T, Bool, Anzeige> Flag<'t, T, Bool, Anzeige>
where
    Bool: Fn(bool) -> T,
{
    pub fn parse<F>(
        self,
        args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, T, F>, Vec<Option<OsString>>) {
        let Flag { beschreibung, invertiere_präfix, invertiere_infix, konvertiere, anzeige: _ } =
            self;
        let Beschreibung { name, hilfe: _, standard } = beschreibung;
        let mut nicht_verwendet = Vec::new();
        let mut iter = args.into_iter();
        while let Some(arg_opt) = iter.next() {
            if let Some(arg) = &arg_opt {
                if let Some(b) = name.parse_flag(&invertiere_präfix, &invertiere_infix, &arg) {
                    nicht_verwendet.push(None);
                    nicht_verwendet.extend(iter);
                    return (Ergebnis::Wert(konvertiere(b)), nicht_verwendet);
                } else {
                    nicht_verwendet.push(arg_opt)
                }
            } else {
                nicht_verwendet.push(arg_opt)
            }
        }
        let ergebnis = if let Some(wert) = standard {
            Ergebnis::Wert(wert)
        } else {
            let fehler = Fehler::FehlendeFlag {
                name,
                invertiere_präfix: invertiere_präfix.string,
                invertiere_infix: invertiere_infix.string,
            };
            Ergebnis::Fehler(NonEmpty::singleton(fehler))
        };
        (ergebnis, nicht_verwendet)
    }
}

impl<T, Bool, Anzeige> Flag<'_, T, Bool, Anzeige>
where
    Anzeige: Fn(&T) -> String,
{
    pub fn erzeuge_hilfe_text(&self, meta_standard: &str) -> (String, Option<Cow<'_, str>>) {
        let Flag { beschreibung, invertiere_präfix, invertiere_infix, konvertiere: _, anzeige } =
            self;
        let Beschreibung { name, hilfe, standard } = beschreibung;
        let Name { lang_präfix, lang, kurz_präfix, kurz } = name;
        let mut hilfe_text = String::new();
        hilfe_text.push_str(lang_präfix.as_str());
        hilfe_text.push('[');
        hilfe_text.push_str(invertiere_präfix.as_str());
        hilfe_text.push_str(invertiere_infix.as_str());
        hilfe_text.push(']');
        let NonEmpty { head, tail } = lang;
        Name::möglichkeiten_als_regex(head, tail.as_slice(), &mut hilfe_text);
        if let Some((h, t)) = kurz.split_first() {
            hilfe_text.push_str(" | ");
            hilfe_text.push_str(kurz_präfix.as_str());
            Name::möglichkeiten_als_regex(h, t, &mut hilfe_text);
        }
        let cow: Option<Cow<'_, str>> = match (hilfe, standard) {
            (None, None) => None,
            (None, Some(standard)) => {
                Some(Cow::Owned(format!("{meta_standard}: {}", anzeige(standard))))
            },
            (Some(hilfe), None) => Some(Cow::Borrowed(hilfe)),
            (Some(hilfe), Some(standard)) => {
                let mut hilfe_text = (*hilfe).to_owned();
                hilfe_text.push(' ');
                hilfe_text.push_str(meta_standard);
                hilfe_text.push_str(": ");
                hilfe_text.push_str(&anzeige(standard));
                Some(Cow::Owned(hilfe_text))
            },
        };
        (hilfe_text, cow)
    }
}
