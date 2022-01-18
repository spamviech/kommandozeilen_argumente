//! Wert-Argument basierend auf seiner [FromStr]-Implementierung.

use std::{ffi::OsStr, fmt::Display, str::FromStr};

use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    argumente::{ArgString, Argumente},
    beschreibung::{contains_str, Beschreibung},
    ergebnis::{Ergebnis, Fehler, ParseFehler},
    sprache::Sprache,
};

#[cfg(any(feature = "derive", doc))]
#[cfg_attr(doc, doc(cfg(feature = "derive")))]
pub use kommandozeilen_argumente_derive::EnumArgument;

impl<T: 'static + Clone + Display, E: 'static + Clone> Argumente<T, E> {
    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    #[inline(always)]
    pub fn wert_display_mit_sprache(
        beschreibung: Beschreibung<T>,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 'static + Fn(&OsStr) -> Result<T, ParseFehler<E>>,
        sprache: Sprache,
    ) -> Argumente<T, E> {
        Argumente::wert_display(beschreibung, sprache.meta_var.to_owned(), mögliche_werte, parse)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    #[inline(always)]
    pub fn wert_display(
        beschreibung: Beschreibung<T>,
        meta_var: String,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 'static + Fn(&OsStr) -> Result<T, ParseFehler<E>>,
    ) -> Argumente<T, E> {
        Argumente::wert(beschreibung, meta_var, mögliche_werte, parse, ToString::to_string)
    }
}

impl<T: 'static + Clone, E: 'static + Clone> Argumente<T, E> {
    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    #[inline(always)]
    pub fn wert_mit_sprache(
        beschreibung: Beschreibung<T>,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 'static + Fn(&OsStr) -> Result<T, ParseFehler<E>>,
        anzeige: impl Fn(&T) -> String,
        sprache: Sprache,
    ) -> Argumente<T, E> {
        Argumente::wert(beschreibung, sprache.meta_var.to_owned(), mögliche_werte, parse, anzeige)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    pub fn wert(
        beschreibung: Beschreibung<T>,
        meta_var: String,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 'static + Fn(&OsStr) -> Result<T, ParseFehler<E>>,
        anzeige: impl Fn(&T) -> String,
    ) -> Argumente<T, E> {
        let name_kurz = beschreibung.kurz.clone();
        let name_lang = beschreibung.lang.clone();
        let meta_var_clone = meta_var.clone();
        let (beschreibung, standard) = beschreibung.als_string_beschreibung_allgemein(&anzeige);
        let fehler_kein_wert = Fehler::FehlenderWert {
            lang: name_lang.clone(),
            kurz: name_kurz.clone(),
            meta_var: meta_var_clone.clone(),
        };
        Argumente {
            beschreibungen: vec![ArgString::Wert {
                beschreibung,
                meta_var,
                mögliche_werte: mögliche_werte
                    .and_then(|werte| NonEmpty::from_vec(werte.iter().map(anzeige).collect())),
            }],
            flag_kurzformen: Vec::new(),
            parse: Box::new(move |args| {
                let name_kurz_existiert = !name_kurz.is_empty();
                let mut ergebnis = None;
                let mut fehler = Vec::new();
                let mut name_ohne_wert = false;
                let mut nicht_verwendet = Vec::new();
                let mut parse_auswerten = |arg| {
                    if let Some(wert_os_str) = arg {
                        match parse(wert_os_str) {
                            Ok(wert) => ergebnis = Some(wert),
                            Err(parse_fehler) => fehler.push(Fehler::Fehler {
                                lang: name_lang.clone(),
                                kurz: name_kurz.clone(),
                                meta_var: meta_var_clone.clone(),
                                fehler: parse_fehler,
                            }),
                        }
                    } else {
                        fehler.push(fehler_kein_wert.clone())
                    }
                };
                for arg in args {
                    if name_ohne_wert {
                        parse_auswerten(arg);
                        name_ohne_wert = false;
                        continue;
                    } else if let Some(string) = arg.and_then(OsStr::to_str) {
                        if let Some(lang) = string.strip_prefix("--") {
                            if let Some((name, wert_str)) = lang.split_once('=') {
                                if contains_str!(&name_lang, name) {
                                    parse_auswerten(Some(wert_str.as_ref()));
                                    continue;
                                }
                            } else if contains_str!(&name_lang, lang) {
                                name_ohne_wert = true;
                                nicht_verwendet.push(None);
                                continue;
                            }
                        } else if name_kurz_existiert {
                            if let Some(kurz) = string.strip_prefix('-') {
                                let mut graphemes = kurz.graphemes(true);
                                if graphemes
                                    .next()
                                    .map(|name| contains_str!(&name_kurz, name))
                                    .unwrap_or(false)
                                {
                                    let rest = graphemes.as_str();
                                    let wert_str = if let Some(wert_str) = rest.strip_prefix('=') {
                                        wert_str
                                    } else if !rest.is_empty() {
                                        rest
                                    } else {
                                        name_ohne_wert = true;
                                        nicht_verwendet.push(None);
                                        continue;
                                    };
                                    parse_auswerten(Some(wert_str.as_ref()));
                                }
                            }
                        }
                    }
                    nicht_verwendet.push(arg);
                }
                if let Some(fehler) = NonEmpty::from_vec(fehler) {
                    (Ergebnis::Fehler(fehler), nicht_verwendet)
                } else if let Some(wert) = ergebnis {
                    (Ergebnis::Wert(wert), nicht_verwendet)
                } else if let Some(wert) = &standard {
                    (Ergebnis::Wert(wert.clone()), nicht_verwendet)
                } else {
                    (
                        Ergebnis::Fehler(NonEmpty::singleton(fehler_kein_wert.clone())),
                        nicht_verwendet,
                    )
                }
            }),
        }
    }
}

/// Trait für Typen mit einer festen Anzahl an Werten und Methode zum Parsen.
/// Gedacht für Summentypen ohne extra Daten (nur Unit-Varianten).
///
/// Mit aktiviertem `derive`-Feature kann die Implementierung
/// [automatisch erzeugt werden](derive@EnumArgument).
pub trait EnumArgument: Sized {
    /// Alle Varianten des Typs.
    fn varianten() -> Vec<Self>;

    /// Versuche einen Wert ausgehend vom übergebenen [OsStr] zu erzeugen.
    fn parse_enum(arg: &OsStr) -> Result<Self, ParseFehler<String>>;
}

impl<T: 'static + Display + Clone + EnumArgument> Argumente<T, String> {
    /// Erzeuge ein Wert-Argument für ein [EnumArgument].
    #[inline(always)]
    pub fn wert_enum_display_mit_sprache(
        beschreibung: Beschreibung<T>,
        sprache: Sprache,
    ) -> Argumente<T, String> {
        Argumente::wert_enum_display(beschreibung, sprache.meta_var.to_owned())
    }

    /// Erzeuge ein Wert-Argument für ein [EnumArgument].
    #[inline(always)]
    pub fn wert_enum_display(
        beschreibung: Beschreibung<T>,
        meta_var: String,
    ) -> Argumente<T, String> {
        Argumente::wert_enum(beschreibung, meta_var, T::to_string)
    }
}

impl<T: 'static + Display + Clone + EnumArgument> Argumente<T, String> {
    /// Erzeuge ein Wert-Argument für ein [EnumArgument].
    #[inline(always)]
    pub fn wert_enum_mit_sprache(
        beschreibung: Beschreibung<T>,
        anzeige: impl Fn(&T) -> String,
        sprache: Sprache,
    ) -> Argumente<T, String> {
        Argumente::wert_enum(beschreibung, sprache.meta_var.to_owned(), anzeige)
    }

    /// Erzeuge ein Wert-Argument für ein [EnumArgument].
    pub fn wert_enum(
        beschreibung: Beschreibung<T>,
        meta_var: String,
        anzeige: impl Fn(&T) -> String,
    ) -> Argumente<T, String> {
        let mögliche_werte = NonEmpty::from_vec(T::varianten());
        Argumente::wert(beschreibung, meta_var, mögliche_werte, T::parse_enum, anzeige)
    }
}

impl<T> Argumente<T, String>
where
    T: 'static + Display + Clone + FromStr,
    T::Err: Display,
{
    /// Erzeuge ein Wert-Argument anhand der [FromStr]-Implementierung.
    #[inline(always)]
    pub fn wert_from_str_display_mit_sprache(
        beschreibung: Beschreibung<T>,
        mögliche_werte: Option<NonEmpty<T>>,
        sprache: Sprache,
    ) -> Argumente<T, String> {
        Argumente::wert_from_str_display(beschreibung, sprache.meta_var.to_owned(), mögliche_werte)
    }

    /// Erzeuge ein Wert-Argument anhand der [FromStr]-Implementierung.
    #[inline(always)]
    pub fn wert_from_str_display(
        beschreibung: Beschreibung<T>,
        meta_var: String,
        mögliche_werte: Option<NonEmpty<T>>,
    ) -> Argumente<T, String> {
        Argumente::wert_from_str(beschreibung, meta_var, mögliche_werte, T::to_string, |fehler| {
            fehler.to_string()
        })
    }
}

impl<T, E> Argumente<T, E>
where
    T: 'static + Clone + FromStr,
    E: 'static + Clone,
{
    /// Erzeuge ein Wert-Argument anhand der [FromStr]-Implementierung.
    #[inline(always)]
    pub fn wert_from_str_mit_sprache(
        beschreibung: Beschreibung<T>,
        mögliche_werte: Option<NonEmpty<T>>,
        anzeige: impl Fn(&T) -> String,
        konvertiere_fehler: impl 'static + Fn(T::Err) -> E,
        sprache: Sprache,
    ) -> Argumente<T, E> {
        Argumente::wert_from_str(
            beschreibung,
            sprache.meta_var.to_owned(),
            mögliche_werte,
            anzeige,
            konvertiere_fehler,
        )
    }

    /// Erzeuge ein Wert-Argument anhand der [FromStr]-Implementierung.
    pub fn wert_from_str(
        beschreibung: Beschreibung<T>,
        meta_var: String,
        mögliche_werte: Option<NonEmpty<T>>,
        anzeige: impl Fn(&T) -> String,
        konvertiere_fehler: impl 'static + Fn(T::Err) -> E,
    ) -> Argumente<T, E> {
        Argumente::wert(
            beschreibung,
            meta_var,
            mögliche_werte,
            move |os_str| {
                os_str
                    .to_str()
                    .ok_or_else(|| ParseFehler::InvaliderString(os_str.to_owned()))
                    .and_then(|string| {
                        T::from_str(string)
                            .map_err(&konvertiere_fehler)
                            .map_err(ParseFehler::ParseFehler)
                    })
            },
            anzeige,
        )
    }
}
