//! Wert-Argument basierend auf seiner [FromStr]-Implementierung.

use std::{
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    str::FromStr,
};

use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    arg::{Arg, ArgString},
    beschreibung::Beschreibung,
    ergebnis::{ParseErgebnis, ParseFehler},
};

#[cfg(feature = "derive")]
pub use kommandozeilen_argumente_derive::ArgEnum;

impl<T: 'static + Clone + Display, E: 'static + Clone> Arg<T, E> {
    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    #[inline(always)]
    pub fn wert(
        beschreibung: Beschreibung<T>,
        meta_var: String,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 'static + Fn(&OsStr) -> Result<T, E>,
    ) -> Arg<T, E> {
        Arg::wert_allgemein(beschreibung, meta_var, mögliche_werte, parse, ToString::to_string)
    }
}

impl<T: 'static + Clone, E: 'static + Clone> Arg<T, E> {
    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    pub fn wert_allgemein(
        beschreibung: Beschreibung<T>,
        meta_var: String,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 'static + Fn(&OsStr) -> Result<T, E>,
        anzeige: impl Fn(&T) -> String,
    ) -> Arg<T, E> {
        let name_kurz = beschreibung.kurz.clone();
        let name_lang = beschreibung.lang.clone();
        let meta_var_clone = meta_var.clone();
        let (beschreibung, standard) = beschreibung.als_string_beschreibung_allgemein(&anzeige);
        let fehler_kein_wert = ParseFehler::FehlenderWert {
            lang: name_lang.clone(),
            kurz: name_kurz.clone(),
            meta_var: meta_var_clone.clone(),
        };
        Arg {
            beschreibungen: vec![ArgString::Wert {
                beschreibung,
                meta_var,
                mögliche_werte: mögliche_werte
                    .and_then(|werte| NonEmpty::from_vec(werte.iter().map(anzeige).collect())),
            }],
            flag_kurzformen: Vec::new(),
            parse: Box::new(move |args| {
                let name_kurz_str = name_kurz.as_ref().map(String::as_str);
                let name_kurz_existiert = name_kurz_str.is_some();
                let mut ergebnis = None;
                let mut fehler = Vec::new();
                let mut name_ohne_wert = false;
                let mut nicht_verwendet = Vec::new();
                let mut parse_auswerten = |arg| {
                    if let Some(wert_os_str) = arg {
                        match parse(wert_os_str) {
                            Ok(wert) => ergebnis = Some(wert),
                            Err(parse_fehler) => fehler.push(ParseFehler::ParseFehler {
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
                                if name == name_lang {
                                    parse_auswerten(Some(wert_str.as_ref()));
                                    continue;
                                }
                            } else if lang == name_lang {
                                name_ohne_wert = true;
                                nicht_verwendet.push(None);
                                continue;
                            }
                        } else if name_kurz_existiert {
                            if let Some(kurz) = string.strip_prefix('-') {
                                let mut graphemes = kurz.graphemes(true);
                                if graphemes.next() == name_kurz_str {
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
                    (ParseErgebnis::Fehler(fehler), nicht_verwendet)
                } else if let Some(wert) = ergebnis {
                    (ParseErgebnis::Wert(wert), nicht_verwendet)
                } else if let Some(wert) = &standard {
                    (ParseErgebnis::Wert(wert.clone()), nicht_verwendet)
                } else {
                    (
                        ParseErgebnis::Fehler(NonEmpty::singleton(fehler_kein_wert.clone())),
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
/// Mit aktiviertem `derive`-Feature kann die Implementierung automatisch erzeugt werden.
pub trait ArgEnum: Sized {
    /// Alle Varianten des Typs.
    fn varianten() -> Vec<Self>;

    // TODO FromStrFehler, z.B. als ParseWertFehler verwenden
    /// Versuche einen Wert ausgehend vom übergebenen [OsStr] zu erzeugen.
    fn parse_enum(arg: &OsStr) -> Result<Self, OsString>;
}

impl<T: 'static + Display + Clone + ArgEnum> Arg<T, OsString> {
    /// Erzeuge ein Wert-Argument für ein [ArgEnum].
    pub fn wert_enum(beschreibung: Beschreibung<T>, meta_var: String) -> Arg<T, OsString> {
        let mögliche_werte = NonEmpty::from_vec(T::varianten());
        Arg::wert(beschreibung, meta_var, mögliche_werte, T::parse_enum)
    }
}

/// Mögliche Fehler-Quellen beim Parsen aus einem [OsStr].
#[derive(Debug, Clone)]
pub enum FromStrFehler<E> {
    /// Die Konvertierung in ein &[str] ist fehlgeschlagen.
    InvaliderString(OsString),
    /// Fehler beim Parsen des Strings.
    ParseFehler(E),
}

impl<T> Arg<T, FromStrFehler<T::Err>>
where
    T: 'static + Display + Clone + FromStr,
    T::Err: 'static + Clone,
{
    /// Erzeuge ein Wert-Argument anhand der [FromStr]-Implementierung.
    pub fn wert_from_str(
        beschreibung: Beschreibung<T>,
        meta_var: String,
        mögliche_werte: Option<NonEmpty<T>>,
    ) -> Arg<T, FromStrFehler<T::Err>> {
        Arg::wert(beschreibung, meta_var, mögliche_werte, |os_str| {
            os_str
                .to_str()
                .ok_or_else(|| FromStrFehler::InvaliderString(os_str.to_owned()))
                .and_then(|string| T::from_str(string).map_err(FromStrFehler::ParseFehler))
        })
    }
}
