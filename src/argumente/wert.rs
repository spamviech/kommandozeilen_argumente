//! Wert-Argumente.

use std::{convert::AsRef, ffi::OsStr, fmt::Display, str::FromStr};

use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    argumente::{Argumente, Arguments},
    beschreibung::{contains_str, Beschreibung, Description, Konfiguration},
    ergebnis::{Ergebnis, Fehler, ParseError, ParseFehler},
    sprache::{Language, Sprache},
};

#[cfg(any(feature = "derive", all(doc, not(doctest))))]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use kommandozeilen_argumente_derive::EnumArgument;

impl<'t, T: 't + Clone + Display, E: Clone> Argumente<'t, T, E> {
    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    ///
    /// ## English synonym
    /// [value_string_display_with_language](Arguments::value_string_display_with_language)
    #[inline(always)]
    pub fn wert_string_display_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t str) -> Result<T, E>,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_string_display(beschreibung, sprache.meta_var, mögliche_werte, parse)
    }

    /// Create a value-argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_string_display_mit_sprache](Argumente::wert_string_display_mit_sprache)
    #[inline(always)]
    pub fn value_string_display_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t str) -> Result<T, E>,
        language: Language,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_string_display_mit_sprache(description, possible_values, parse, language)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    ///
    /// ## English synonym
    /// [value_string_display](Arguments::value_string_display)
    #[inline(always)]
    pub fn wert_string_display(
        beschreibung: Beschreibung<'t, T>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t str) -> Result<T, E>,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_string(beschreibung, meta_var, mögliche_werte, parse, ToString::to_string)
    }

    /// Create a value-argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_string_display](Argumente::wert_string_display)
    #[inline(always)]
    pub fn value_string_display(
        description: Description<'t, T>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t str) -> Result<T, E>,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_string_display(description, meta_var, possible_values, parse)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    ///
    /// ## English synonym
    /// [value_display_with_language](Arguments::value_display_with_language)
    #[inline(always)]
    pub fn wert_display_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t OsStr) -> Result<T, ParseFehler<'t, E>>,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_display(beschreibung, sprache.meta_var, mögliche_werte, parse)
    }

    /// Create a value-argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_display_mit_sprache](Argumente::wert_display_mit_sprache)
    #[inline(always)]
    pub fn value_display_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t OsStr) -> Result<T, ParseError<'t, E>>,
        language: Language,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_display_mit_sprache(description, possible_values, parse, language)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    ///
    /// ## English synonym
    /// [value_display](Arguments::value_display)
    #[inline(always)]
    pub fn wert_display(
        beschreibung: Beschreibung<'t, T>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t OsStr) -> Result<T, ParseFehler<'t, E>>,
    ) -> Argumente<'t, T, E> {
        Argumente::wert(beschreibung, meta_var, mögliche_werte, parse, ToString::to_string)
    }

    /// Create a Value-Argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_display](Argumente::wert_display)
    #[inline(always)]
    pub fn value_display(
        description: Description<'t, T>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t OsStr) -> Result<T, ParseError<'t, E>>,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_display(description, meta_var, possible_values, parse)
    }
}

impl<'t, T: 't + Clone, E> Argumente<'t, T, E> {
    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    ///
    /// ## English synonym
    /// [value_string_with_language](Arguments::value_string_with_language)
    #[inline(always)]
    pub fn wert_string_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t str) -> Result<T, E>,
        anzeige: impl Fn(&T) -> String,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_string(beschreibung, sprache.meta_var, mögliche_werte, parse, anzeige)
    }

    /// Create a Value-Argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_string_mit_sprache](Argumente::wert_string_mit_sprache)
    #[inline(always)]
    pub fn value_string_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t str) -> Result<T, E>,
        display: impl Fn(&T) -> String,
        language: Language,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_string_mit_sprache(description, possible_values, parse, display, language)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    ///
    /// ## English synonym
    /// [value_string](Arguments::value_string)
    #[inline(always)]
    pub fn wert_string(
        beschreibung: Beschreibung<'t, T>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t str) -> Result<T, E>,
        anzeige: impl Fn(&T) -> String,
    ) -> Argumente<'t, T, E> {
        Argumente::wert(
            beschreibung,
            meta_var,
            mögliche_werte,
            move |os_str| {
                if let Some(s) = os_str.to_str() {
                    parse(s).map_err(ParseFehler::ParseFehler)
                } else {
                    Err(ParseFehler::InvaliderString(os_str))
                }
            },
            anzeige,
        )
    }

    /// Create a Value-Argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_string](Argumente::wert_string)
    #[inline(always)]
    pub fn value_string(
        description: Description<'t, T>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t str) -> Result<T, E>,
        display: impl Fn(&T) -> String,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_string(description, meta_var, possible_values, parse, display)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    ///
    /// ## English synonym
    /// [value_with_language](Arguments::value_with_language)
    #[inline(always)]
    pub fn wert_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t OsStr) -> Result<T, ParseFehler<'t, E>>,
        anzeige: impl Fn(&T) -> String,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::wert(beschreibung, sprache.meta_var, mögliche_werte, parse, anzeige)
    }

    /// Create a Value-Argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_mit_sprache](Arguments::wert_mit_sprache)
    #[inline(always)]
    pub fn value_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t OsStr) -> Result<T, ParseFehler<'t, E>>,
        display: impl Fn(&T) -> String,
        language: Language,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_mit_sprache(description, possible_values, parse, display, language)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    ///
    /// ## English synonym
    /// [value](Arguments::value)
    pub fn wert(
        beschreibung: Beschreibung<'t, T>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t OsStr) -> Result<T, ParseFehler<'t, E>>,
        anzeige: impl Fn(&T) -> String,
    ) -> Argumente<'t, T, E> {
        let name_kurz = beschreibung.kurz.clone();
        let name_lang = beschreibung.lang.clone();
        let (beschreibung, standard) = beschreibung.als_string_beschreibung_allgemein(&anzeige);
        Argumente {
            konfigurationen: vec![Konfiguration::Wert {
                beschreibung,
                meta_var,
                mögliche_werte: mögliche_werte
                    .and_then(|werte| NonEmpty::from_vec(werte.iter().map(anzeige).collect())),
            }],
            flag_kurzformen: Vec::new(),
            parse: Box::new(move |args| {
                let fehler_kein_wert = || Fehler::FehlenderWert {
                    lang: name_lang.clone(),
                    kurz: name_kurz.clone(),
                    meta_var,
                };
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
                                meta_var,
                                fehler: parse_fehler,
                            }),
                        }
                    } else {
                        fehler.push(fehler_kein_wert())
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
                    (Ergebnis::Fehler(NonEmpty::singleton(fehler_kein_wert())), nicht_verwendet)
                }
            }),
        }
    }

    /// Create a Value-Argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert](Argumente::wert)
    #[inline(always)]
    pub fn value(
        description: Description<'t, T>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&'t OsStr) -> Result<T, ParseError<'t, E>>,
        display: impl Fn(&T) -> String,
    ) -> Arguments<'t, T, E> {
        Argumente::wert(description, meta_var, possible_values, parse, display)
    }
}

/// Trait für Typen mit einer festen Anzahl an Werten und Methode zum Parsen.
/// Gedacht für Summentypen ohne extra Daten (nur Unit-Varianten).
///
/// Mit aktiviertem `derive`-Feature kann die Implementierung
/// [automatisch erzeugt werden](derive@EnumArgument).
///
/// ## English
/// Trait for types with a fixed number of values and a parse method.
/// Intended for sum-types without extra data (only unit variants).
///
/// With activated `derive`-feature, the implementation can be
/// [created automatically](derive@EnumArgument).
pub trait EnumArgument: Sized {
    /// Alle Varianten des Typs.
    ///
    /// ## English synonym
    /// [variants](EnumArgument::variants)
    fn varianten() -> Vec<Self>;

    /// All variants of the type.
    ///
    /// ## Deutsches Synonym
    /// [varianten](EnumArgument::varianten)
    #[inline(always)]
    fn variants() -> Vec<Self> {
        Self::varianten()
    }

    /// Versuche einen Wert ausgehend vom übergebenen [OsStr] zu erzeugen.
    ///
    /// ## English
    /// Try to parse a value from the given [OsStr].
    fn parse_enum(arg: &OsStr) -> Result<Self, ParseFehler<'_, String>>;
}

impl<'t, T: 't + Display + Clone + EnumArgument> Argumente<'t, T, String> {
    /// Erzeuge ein Wert-Argument für ein [EnumArgument].
    ///
    /// ## English synonym
    /// [value_enum_display_with_language](Arguments::value_enum_display_with_language)
    #[inline(always)]
    pub fn wert_enum_display_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        sprache: Sprache,
    ) -> Argumente<'t, T, String> {
        Argumente::wert_enum_display(beschreibung, sprache.meta_var)
    }

    /// Create a value-argument for an [EnumArgument].
    ///
    /// ## Deutsches Synonym
    /// [wert_enum_display_mit_sprache](Argumente::wert_enum_display_mit_sprache)
    #[inline(always)]
    pub fn value_enum_display_with_language(
        description: Description<'t, T>,
        language: Language,
    ) -> Arguments<'t, T, String> {
        Argumente::wert_enum_display_mit_sprache(description, language)
    }

    /// Erzeuge ein Wert-Argument für ein [EnumArgument].
    ///
    /// ## English synonym
    /// [value_enum_display](Arguments::value_enum_display)
    #[inline(always)]
    pub fn wert_enum_display(
        beschreibung: Beschreibung<'t, T>,
        meta_var: &'t str,
    ) -> Argumente<'t, T, String> {
        Argumente::wert_enum(beschreibung, meta_var, T::to_string)
    }

    /// Create a value-argument for an [EnumArgument].
    ///
    /// ## Deutsches Synonym
    /// [wert_enum_display](Argumente::wert_enum_display)
    #[inline(always)]
    pub fn value_enum_display(
        description: Description<'t, T>,
        meta_var: &'t str,
    ) -> Arguments<'t, T, String> {
        Argumente::wert_enum_display(description, meta_var)
    }
}

impl<'t, T: 't + Display + Clone + EnumArgument> Argumente<'t, T, String> {
    /// Erzeuge ein Wert-Argument für ein [EnumArgument].
    ///
    /// ## English synonym
    /// [value_enum_with_language](Arguments::value_enum_with_language)
    #[inline(always)]
    pub fn wert_enum_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        anzeige: impl Fn(&T) -> String,
        sprache: Sprache,
    ) -> Argumente<'t, T, String> {
        Argumente::wert_enum(beschreibung, sprache.meta_var, anzeige)
    }

    /// Create a value-argument for an [EnumArgument].
    ///
    /// ## Deutsches Synonym
    /// [wert_enum_mit_sprache](Argumente::wert_enum_mit_sprache)
    #[inline(always)]
    pub fn value_enum_with_language(
        description: Description<'t, T>,
        display: impl Fn(&T) -> String,
        language: Language,
    ) -> Arguments<'t, T, String> {
        Argumente::wert_enum_mit_sprache(description, display, language)
    }

    /// Erzeuge ein Wert-Argument für ein [EnumArgument].
    ///
    /// ## English synonym
    /// [value_enum](Arguments::value_enum)
    pub fn wert_enum(
        beschreibung: Beschreibung<'t, T>,
        meta_var: &'t str,
        anzeige: impl Fn(&T) -> String,
    ) -> Argumente<'t, T, String> {
        let mögliche_werte = NonEmpty::from_vec(T::varianten());
        Argumente::wert(beschreibung, meta_var, mögliche_werte, T::parse_enum, anzeige)
    }

    /// Create a value-argument for an [EnumArgument].
    ///
    /// ## Deutsches Synonym
    /// [wert_enum](Argumente::wert_enum)
    #[inline(always)]
    pub fn value_enum(
        description: Description<'t, T>,
        meta_var: &'t str,
        display: impl Fn(&T) -> String,
    ) -> Arguments<'t, T, String> {
        Argumente::wert_enum(description, meta_var, display)
    }
}

impl<'t, T> Argumente<'t, T, String>
where
    T: 't + Display + Clone + FromStr,
    T::Err: Display,
{
    /// Erzeuge ein Wert-Argument anhand der [FromStr]-Implementierung.
    ///
    /// ## English synonym
    /// [value_from_str_display_with_language](Arguments::value_from_str_display_with_language)
    #[inline(always)]
    pub fn wert_from_str_display_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        mögliche_werte: Option<NonEmpty<T>>,
        sprache: Sprache,
    ) -> Argumente<'t, T, String> {
        Argumente::wert_from_str_display(beschreibung, sprache.meta_var, mögliche_werte)
    }

    /// Create a value-argument based on its [FromStr] implementation.
    ///
    /// ## Deutsches Synonym
    /// [wert_from_str_display_mit_sprache](Argumente::wert_from_str_display_mit_sprache)
    #[inline(always)]
    pub fn value_from_str_display_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        language: Language,
    ) -> Argumente<'t, T, String> {
        Argumente::wert_from_str_display_mit_sprache(description, possible_values, language)
    }

    /// Erzeuge ein Wert-Argument anhand der [FromStr]-Implementierung.
    ///
    /// ## English synonym
    /// [value_from_str_display](Arguments::value_from_str_display)
    #[inline(always)]
    pub fn wert_from_str_display(
        beschreibung: Beschreibung<'t, T>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
    ) -> Argumente<'t, T, String> {
        Argumente::wert_from_str(beschreibung, meta_var, mögliche_werte, T::to_string, |fehler| {
            fehler.to_string()
        })
    }

    /// Create a value-argument based on its [FromStr] implementation.
    ///
    /// ## Deutsches Synonym
    /// [wert_from_str_display](Argumente::wert_from_str_display)
    #[inline(always)]
    pub fn value_from_str_display(
        description: Description<'t, T>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
    ) -> Argumente<'t, T, String> {
        Argumente::wert_from_str_display(description, meta_var, possible_values)
    }
}

impl<'t, T: 't + Clone + FromStr, E: Clone> Argumente<'t, T, E> {
    /// Erzeuge ein Wert-Argument anhand der [FromStr]-Implementierung.
    ///
    /// ## English synonym
    /// [value_from_str_with_language](Arguments::value_from_str_with_language)
    #[inline(always)]
    pub fn wert_from_str_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        mögliche_werte: Option<NonEmpty<T>>,
        anzeige: impl Fn(&T) -> String,
        konvertiere_fehler: impl 't + Fn(T::Err) -> E,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_from_str(
            beschreibung,
            sprache.meta_var,
            mögliche_werte,
            anzeige,
            konvertiere_fehler,
        )
    }

    /// Create a value-argument based on its [FromStr] implementation.
    ///
    /// ## Deutsches Synonym
    /// [wert_from_str_mit_sprache](Argumente::wert_from_str_mit_sprache)
    #[inline(always)]
    pub fn value_from_str_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        display: impl Fn(&T) -> String,
        convert_error: impl 't + Fn(T::Err) -> E,
        language: Language,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_from_str_mit_sprache(
            description,
            possible_values,
            display,
            convert_error,
            language,
        )
    }

    /// Erzeuge ein Wert-Argument anhand der [FromStr]-Implementierung.
    ///
    /// ## English synonym
    /// [value_from_str](Arguments::value_from_str)
    pub fn wert_from_str(
        beschreibung: Beschreibung<'t, T>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
        anzeige: impl Fn(&T) -> String,
        konvertiere_fehler: impl 't + Fn(T::Err) -> E,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_string(
            beschreibung,
            meta_var,
            mögliche_werte,
            move |string| T::from_str(string).map_err(&konvertiere_fehler),
            anzeige,
        )
    }

    /// Create a value-argument based on its [FromStr] implementation.
    ///
    /// ## Deutsches Synonym
    /// [wert_from_str](Argumente::wert_from_str)
    #[inline(always)]
    pub fn value_from_str(
        description: Description<'t, T>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
        display: impl Fn(&T) -> String,
        convert_error: impl 't + Fn(T::Err) -> E,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_from_str(description, meta_var, possible_values, display, convert_error)
    }
}
