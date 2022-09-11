//! Wert-Argumente.

use std::{
    borrow::Cow,
    collections::HashMap,
    ffi::{OsStr, OsString},
    fmt::Display,
    str::FromStr,
};

use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    argumente::{Argumente, Arguments},
    beschreibung::{contains_prefix, contains_str, Beschreibung, Description, Konfiguration, Name},
    ergebnis::{Ergebnis, Fehler, ParseError, ParseFehler},
    sprache::{Language, Sprache},
    unicode::{Compare, Normalisiert, Vergleich},
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
        parse: impl 't + Fn(&str) -> Result<T, E>,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_string_display(
            beschreibung,
            sprache.wert_infix,
            sprache.meta_var,
            mögliche_werte,
            parse,
        )
    }

    /// Create a value-argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_string_display_mit_sprache](Argumente::wert_string_display_mit_sprache)
    #[inline(always)]
    pub fn value_string_display_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&str) -> Result<T, E>,
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
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&str) -> Result<T, E>,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_string(
            beschreibung,
            wert_infix,
            meta_var,
            mögliche_werte,
            parse,
            ToString::to_string,
        )
    }

    /// Create a value-argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_string_display](Argumente::wert_string_display)
    #[inline(always)]
    pub fn value_string_display(
        description: Description<'t, T>,
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&str) -> Result<T, E>,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_string_display(description, value_infix, meta_var, possible_values, parse)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    ///
    /// ## English synonym
    /// [value_display_with_language](Arguments::value_display_with_language)
    #[inline(always)]
    pub fn wert_display_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(OsString) -> Result<T, ParseError<E>>,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_display(
            beschreibung,
            sprache.wert_infix,
            sprache.meta_var,
            mögliche_werte,
            parse,
        )
    }

    /// Create a value-argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_display_mit_sprache](Argumente::wert_display_mit_sprache)
    #[inline(always)]
    pub fn value_display_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(OsString) -> Result<T, ParseError<E>>,
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
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(OsString) -> Result<T, ParseError<E>>,
    ) -> Argumente<'t, T, E> {
        Argumente::wert(
            beschreibung,
            wert_infix,
            meta_var,
            mögliche_werte,
            parse,
            ToString::to_string,
        )
    }

    /// Create a Value-Argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_display](Argumente::wert_display)
    #[inline(always)]
    pub fn value_display(
        description: Description<'t, T>,
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(OsString) -> Result<T, ParseError<E>>,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_display(description, value_infix, meta_var, possible_values, parse)
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
        parse: impl 't + Fn(&str) -> Result<T, E>,
        anzeige: impl Fn(&T) -> String,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_string(
            beschreibung,
            sprache.wert_infix,
            sprache.meta_var,
            mögliche_werte,
            parse,
            anzeige,
        )
    }

    /// Create a Value-Argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_string_mit_sprache](Argumente::wert_string_mit_sprache)
    #[inline(always)]
    pub fn value_string_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&str) -> Result<T, E>,
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
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&str) -> Result<T, E>,
        anzeige: impl Fn(&T) -> String,
    ) -> Argumente<'t, T, E> {
        Argumente::wert(
            beschreibung,
            wert_infix,
            meta_var,
            mögliche_werte,
            move |os_str| {
                if let Some(s) = os_str.to_str() {
                    parse(s).map_err(ParseFehler::ParseFehler)
                } else {
                    Err(ParseFehler::InvaliderString(os_str.to_owned()))
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
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(&str) -> Result<T, E>,
        display: impl Fn(&T) -> String,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_string(description, value_infix, meta_var, possible_values, parse, display)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der übergebenen `parse`-Funktion.
    ///
    /// ## English synonym
    /// [value_with_language](Arguments::value_with_language)
    #[inline(always)]
    pub fn wert_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(OsString) -> Result<T, ParseError<E>>,
        anzeige: impl Fn(&T) -> String,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        Argumente::wert(
            beschreibung,
            sprache.wert_infix,
            sprache.meta_var,
            mögliche_werte,
            parse,
            anzeige,
        )
    }

    /// Create a Value-Argument, based on the given `parse`-function.
    ///
    /// ## Deutsches Synonym
    /// [wert_mit_sprache](Arguments::wert_mit_sprache)
    #[inline(always)]
    pub fn value_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(OsString) -> Result<T, ParseError<E>>,
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
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
        parse: impl 't + Fn(OsString) -> Result<T, ParseFehler<E>>,
        anzeige: impl Fn(&T) -> String,
    ) -> Argumente<'t, T, E> {
        let name_lang_präfix = beschreibung.name.lang_präfix.clone();
        let name_lang = beschreibung.name.lang.clone();
        let name_kurz_präfix = beschreibung.name.kurz_präfix.clone();
        let name_kurz = beschreibung.name.kurz.clone();
        let (beschreibung, standard) = beschreibung.als_string_beschreibung_allgemein(&anzeige);
        let wert_infix_vergleich = wert_infix.into();
        Argumente {
            konfigurationen: vec![Konfiguration::Wert {
                beschreibung,
                wert_infix: wert_infix_vergleich.clone(),
                meta_var,
                mögliche_werte: mögliche_werte
                    .and_then(|werte| NonEmpty::from_vec(werte.iter().map(anzeige).collect())),
            }],
            flag_kurzformen: HashMap::new(),
            parse: Box::new(move |args| {
                let fehler_namen = || Name {
                    lang_präfix: name_lang_präfix.clone(),
                    lang: name_lang.clone(),
                    kurz_präfix: name_kurz_präfix.clone(),
                    kurz: name_kurz.clone(),
                };
                let fehler_kein_wert = || Fehler::FehlenderWert {
                    name: fehler_namen(),
                    wert_infix: wert_infix_vergleich.string.clone(),
                    meta_var,
                };
                let name_kurz_existiert = !name_kurz.is_empty();
                let mut ergebnis = None;
                let mut fehler = Vec::new();
                let mut name_ohne_wert = false;
                let mut nicht_verwendet = Vec::new();
                let mut parse_auswerten = |arg: Option<OsString>| {
                    if let Some(wert_os_str) = arg {
                        match parse(wert_os_str) {
                            Ok(wert) => ergebnis = Some(wert),
                            Err(parse_fehler) => fehler.push(Fehler::Fehler {
                                name: fehler_namen(),
                                wert_infix: wert_infix_vergleich.string.clone(),
                                meta_var,
                                fehler: parse_fehler,
                            }),
                        }
                    } else {
                        fehler.push(fehler_kein_wert())
                    }
                };
                'args: for arg in args {
                    if name_ohne_wert {
                        parse_auswerten(arg);
                        name_ohne_wert = false;
                        nicht_verwendet.push(None);
                        continue;
                    } else if let Some(string) =
                        arg.as_ref().and_then(|os_string| os_string.to_str())
                    {
                        let normalisiert = Normalisiert::neu(string);
                        if let Some(lang) = name_lang_präfix.strip_als_präfix(&normalisiert) {
                            let lang_normalisiert = Normalisiert::neu_borrowed_unchecked(lang);
                            let suffixe = contains_prefix(&name_lang, &lang_normalisiert);
                            for suffix in suffixe {
                                let suffix_normalisiert =
                                    Normalisiert::neu_borrowed_unchecked(suffix);
                                if suffix.is_empty() {
                                    name_ohne_wert = true;
                                    nicht_verwendet.push(None);
                                    continue 'args;
                                } else if let Some(wert_graphemes) =
                                    wert_infix_vergleich.strip_als_präfix(&suffix_normalisiert)
                                {
                                    parse_auswerten(Some(wert_graphemes.to_owned().into()));
                                    nicht_verwendet.push(None);
                                    continue 'args;
                                }
                            }
                        } else if name_kurz_existiert {
                            if let Some(kurz) = name_kurz_präfix.strip_als_präfix(&normalisiert) {
                                let mut kurz_graphemes = kurz.graphemes(true);
                                if kurz_graphemes
                                    .next()
                                    .map(|name| contains_str(&name_kurz, name))
                                    .unwrap_or(false)
                                {
                                    let rest = kurz_graphemes.as_str();
                                    let kurz_normalisiert =
                                        Normalisiert::neu_borrowed_unchecked(rest);
                                    let wert_str = if rest.is_empty() {
                                        name_ohne_wert = true;
                                        nicht_verwendet.push(None);
                                        continue 'args;
                                    } else {
                                        wert_infix_vergleich
                                            .strip_als_präfix(&kurz_normalisiert)
                                            .unwrap_or(rest)
                                    };
                                    parse_auswerten(Some(wert_str.to_owned().into()));
                                    nicht_verwendet.push(None);
                                    continue 'args;
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
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
        parse: impl 't + Fn(OsString) -> Result<T, ParseError<E>>,
        display: impl Fn(&T) -> String,
    ) -> Arguments<'t, T, E> {
        Argumente::wert(description, value_infix, meta_var, possible_values, parse, display)
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

    /// Versuche einen Wert ausgehend vom übergebenen [OsString] zu erzeugen.
    ///
    /// ## English
    /// Try to parse a value from the given [OsString].
    fn parse_enum(arg: OsString) -> Result<Self, ParseFehler<String>>;
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
        Argumente::wert_enum_display(beschreibung, sprache.wert_infix, sprache.meta_var)
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
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
    ) -> Argumente<'t, T, String> {
        Argumente::wert_enum(beschreibung, wert_infix, meta_var, T::to_string)
    }

    /// Create a value-argument for an [EnumArgument].
    ///
    /// ## Deutsches Synonym
    /// [wert_enum_display](Argumente::wert_enum_display)
    #[inline(always)]
    pub fn value_enum_display(
        description: Description<'t, T>,
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
    ) -> Arguments<'t, T, String> {
        Argumente::wert_enum_display(description, value_infix, meta_var)
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
        Argumente::wert_enum(beschreibung, sprache.wert_infix, sprache.meta_var, anzeige)
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
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
        anzeige: impl Fn(&T) -> String,
    ) -> Argumente<'t, T, String> {
        let mögliche_werte = NonEmpty::from_vec(T::varianten());
        Argumente::wert(beschreibung, wert_infix, meta_var, mögliche_werte, T::parse_enum, anzeige)
    }

    /// Create a value-argument for an [EnumArgument].
    ///
    /// ## Deutsches Synonym
    /// [wert_enum](Argumente::wert_enum)
    #[inline(always)]
    pub fn value_enum(
        description: Description<'t, T>,
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
        display: impl Fn(&T) -> String,
    ) -> Arguments<'t, T, String> {
        Argumente::wert_enum(description, value_infix, meta_var, display)
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
        Argumente::wert_from_str_display(
            beschreibung,
            sprache.wert_infix,
            sprache.meta_var,
            mögliche_werte,
        )
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
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
    ) -> Argumente<'t, T, String> {
        Argumente::wert_from_str(
            beschreibung,
            wert_infix,
            meta_var,
            mögliche_werte,
            T::to_string,
            |fehler| fehler.to_string(),
        )
    }

    /// Create a value-argument based on its [FromStr] implementation.
    ///
    /// ## Deutsches Synonym
    /// [wert_from_str_display](Argumente::wert_from_str_display)
    #[inline(always)]
    pub fn value_from_str_display(
        description: Description<'t, T>,
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
    ) -> Argumente<'t, T, String> {
        Argumente::wert_from_str_display(description, value_infix, meta_var, possible_values)
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
            sprache.wert_infix,
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
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
        mögliche_werte: Option<NonEmpty<T>>,
        anzeige: impl Fn(&T) -> String,
        konvertiere_fehler: impl 't + Fn(T::Err) -> E,
    ) -> Argumente<'t, T, E> {
        Argumente::wert_string(
            beschreibung,
            wert_infix,
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
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
        possible_values: Option<NonEmpty<T>>,
        display: impl Fn(&T) -> String,
        convert_error: impl 't + Fn(T::Err) -> E,
    ) -> Arguments<'t, T, E> {
        Argumente::wert_from_str(
            description,
            value_infix,
            meta_var,
            possible_values,
            display,
            convert_error,
        )
    }
}

/// Es handelt sich um ein Wert-Argument.
///
/// ## English
/// It is a value argument.
#[derive(Debug)]
pub struct Wert<'t, T, Parse, Anzeige> {
    /// Allgemeine Beschreibung des Arguments.
    ///
    /// ## English
    /// General description of the argument.
    pub beschreibung: Beschreibung<'t, T>,

    /// Infix um einen Wert im selben Argument wie den Namen anzugeben.
    ///
    /// ## English
    /// Infix to give a value in the same argument as the name.
    pub wert_infix: Vergleich<'t>,

    /// Meta-Variable im Hilfe-Text.
    ///
    /// ## English
    /// Meta-variable used in the help-text.
    pub meta_var: &'t str,

    /// String-Darstellung der erlaubten Werte.
    ///
    /// ## English
    /// String-representation of the allowed values.
    pub mögliche_werte: Option<NonEmpty<T>>,

    /// Parse einen Wert aus einem [OsString].
    ///
    /// ## English
    /// Parse a value from an [OsString].
    pub parse: Parse,

    /// Anzeige eines Wertes (standard/mögliche Werte).
    ///
    /// ## English
    /// Display a value (default/possible values).
    pub anzeige: Anzeige,
}

fn zeige_elemente<'t, T: 't, Anzeige: Fn(&T) -> String>(
    s: &mut String,
    anzeige: &Anzeige,
    elemente: impl IntoIterator<Item = &'t T>,
) {
    let mut erstes = true;
    for element in elemente {
        if erstes {
            erstes = false;
        } else {
            s.push_str(", ");
        }
        s.push_str(&anzeige(element));
    }
}

impl<'t, T, Parse, F, Anzeige> Wert<'t, T, Parse, Anzeige>
where
    Parse: Fn(&OsStr) -> Result<T, ParseFehler<F>>,
{
    pub fn parse<I: Iterator<Item = Option<OsString>>>(
        self,
        args: I,
    ) -> (Ergebnis<'t, T, F>, Vec<Option<OsString>>) {
        let Wert { beschreibung, wert_infix, meta_var, mögliche_werte: _, parse, anzeige: _ } =
            self;
        let Beschreibung { name, hilfe: _, standard } = beschreibung;
        let mut nicht_verwendet = Vec::new();
        let mut iter = args.into_iter();
        let mut name_ohne_wert = None;
        let parse_wert = |arg: &OsStr,
                          mut nicht_verwendet: Vec<_>,
                          name: Name<'t>,
                          wert_infix: Vergleich<'t>,
                          iter: I|
         -> (Ergebnis<'t, T, F>, Vec<Option<OsString>>) {
            nicht_verwendet.push(None);
            nicht_verwendet.extend(iter);
            let ergebnis = match parse(arg) {
                Ok(wert) => Ergebnis::Wert(wert),
                Err(fehler) => Ergebnis::Fehler(NonEmpty::singleton(Fehler::Fehler {
                    name,
                    wert_infix: wert_infix.string,
                    meta_var,
                    fehler,
                })),
            };
            (ergebnis, nicht_verwendet)
        };
        while let Some(arg_opt) = iter.next() {
            if let Some(arg) = &arg_opt {
                if let Some(_name_arg) = name_ohne_wert.take() {
                    return parse_wert(arg, nicht_verwendet, name, wert_infix, iter);
                } else if let Some(wert_opt) = name.parse_mit_wert(&wert_infix, arg) {
                    if let Some(wert_str) = wert_opt {
                        return parse_wert(&wert_str, nicht_verwendet, name, wert_infix, iter);
                    } else {
                        name_ohne_wert = Some(arg_opt);
                    }
                } else {
                    nicht_verwendet.push(arg_opt)
                }
            } else {
                if let Some(name) = name_ohne_wert.take() {
                    nicht_verwendet.push(name);
                }
                nicht_verwendet.push(arg_opt);
            }
        }
        let ergebnis = if let Some(wert) = standard {
            Ergebnis::Wert(wert)
        } else {
            Ergebnis::Fehler(NonEmpty::singleton(Fehler::FehlenderWert {
                name,
                wert_infix: wert_infix.string,
                meta_var,
            }))
        };
        (ergebnis, nicht_verwendet)
    }
}

impl<T, Parse, Anzeige> Wert<'_, T, Parse, Anzeige>
where
    Anzeige: Fn(&T) -> String,
{
    pub fn erzeuge_hilfe_text(
        &self,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> (String, Option<Cow<'_, str>>) {
        let Wert { beschreibung, wert_infix, meta_var, mögliche_werte, parse: _, anzeige } = self;
        let Beschreibung { name, hilfe, standard } = beschreibung;
        let Name { lang_präfix, lang, kurz_präfix, kurz } = name;
        let mut hilfe_text = String::new();
        hilfe_text.push_str(lang_präfix.as_str());
        let NonEmpty { head, tail } = lang;
        Name::möglichkeiten_als_regex(head, tail.as_slice(), &mut hilfe_text);
        hilfe_text.push_str("( |");
        hilfe_text.push_str(wert_infix.as_str());
        hilfe_text.push(')');
        hilfe_text.push_str(meta_var);
        if let Some((h, t)) = kurz.split_first() {
            hilfe_text.push_str(" | ");
            hilfe_text.push_str(kurz_präfix.as_str());
            Name::möglichkeiten_als_regex(h, t, &mut hilfe_text);
            hilfe_text.push_str("[ |");
            hilfe_text.push_str(wert_infix.as_str());
            hilfe_text.push(']');
            hilfe_text.push_str(meta_var);
        }
        // TODO a lot of code duplication...
        let cow: Option<Cow<'_, str>> = match (hilfe, standard, mögliche_werte) {
            (None, None, None) => None,
            (None, None, Some(mögliche_werte)) => {
                let mut s = format!("[{meta_erlaubte_werte}: ");
                zeige_elemente(&mut s, &self.anzeige, mögliche_werte);
                s.push(']');
                Some(Cow::Owned(s))
            },
            (None, Some(standard), None) => {
                Some(Cow::Owned(format!("[{meta_standard}: {}]", anzeige(standard))))
            },
            (None, Some(standard), Some(mögliche_werte)) => {
                let mut s =
                    format!("[{meta_standard}: {}, {meta_erlaubte_werte}: ", anzeige(standard));
                zeige_elemente(&mut s, &self.anzeige, mögliche_werte);
                s.push(']');
                Some(Cow::Owned(s))
            },
            (Some(hilfe), None, None) => Some(Cow::Borrowed(hilfe)),
            (Some(hilfe), None, Some(mögliche_werte)) => {
                let mut s = format!("{hilfe} [{meta_erlaubte_werte}: ");
                zeige_elemente(&mut s, &self.anzeige, mögliche_werte);
                s.push(']');
                Some(Cow::Owned(s))
            },
            (Some(hilfe), Some(standard), None) => {
                Some(Cow::Owned(format!("{hilfe} [{meta_standard}: {}]", anzeige(standard))))
            },
            (Some(hilfe), Some(standard), Some(mögliche_werte)) => {
                let mut s = format!(
                    "{hilfe} [{meta_standard}: {}, {meta_erlaubte_werte}: ",
                    anzeige(standard)
                );
                zeige_elemente(&mut s, &self.anzeige, mögliche_werte);
                s.push(']');
                Some(Cow::Owned(s))
            },
        };
        (hilfe_text, cow)
    }
}
