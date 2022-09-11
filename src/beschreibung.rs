//! Beschreibung eines Arguments.

use std::{
    borrow::Cow,
    convert::AsRef,
    ffi::{OsStr, OsString},
    fmt::Display,
};

use itertools::Itertools;
use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    sprache::{Language, Sprache},
    unicode::{Case, Compare, Normalisiert, Vergleich},
};

/// Alle Namen eines Arguments.
///
/// ## English
/// All names of an argument.
#[derive(Debug, Clone)]
pub struct Name<'t> {
    /// Präfix vor dem LangNamen.
    ///
    /// ## English
    /// Prefix before the long name.
    pub lang_präfix: Vergleich<'t>,

    /// Voller Name, wird nach `lang_präfix` angegeben.
    ///
    /// ## English
    /// Full Name, given after `lang_präfix`.
    pub lang: NonEmpty<Vergleich<'t>>,

    /// Präfix vor dem KurzNamen.
    ///
    /// ## English
    /// Prefix before the short name.
    pub kurz_präfix: Vergleich<'t>,

    /// Kurzer Name, wird nach `kurz_präfix` angegeben.
    /// Bei Flag-Argumenten können KurzNamen mit identischen `kurz_präfix` zusammen angegeben werden,
    /// zum Beispiel "-fgh".
    /// Kurznamen länger als ein [Grapheme](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// werden nicht unterstützt.
    ///
    /// ## English
    /// Short name, given after `short_präfix`.
    /// Flag arguments with identical `kurz_präfix` may be given at once, e.g. "-fgh".
    /// Short names longer than a [Grapheme](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// are not supported.
    pub kurz: Vec<Vergleich<'t>>,
}

impl Name<'_> {
    fn parse_flag_aux<E>(
        &self,
        name_gefunden: impl FnOnce() -> E,
        parse_invertiert: impl FnOnce(&NonEmpty<Vergleich<'_>>, &Normalisiert<'_>) -> Option<E>,
        arg: &OsStr,
    ) -> Option<E> {
        let Name { lang_präfix, lang, kurz_präfix, kurz } = self;
        let name_kurz_existiert = !kurz.is_empty();
        if let Some(string) = arg.to_str() {
            let normalisiert = Normalisiert::neu(string);
            if let Some(lang_str) = &lang_präfix.strip_als_präfix_n(&normalisiert) {
                if contains_str(lang, lang_str.as_str()) {
                    return Some(name_gefunden());
                } else if let Some(e) = parse_invertiert(lang, lang_str) {
                    return Some(e);
                }
            } else if name_kurz_existiert {
                if let Some(kurz_graphemes) = kurz_präfix.strip_als_präfix_n(&normalisiert) {
                    if kurz_graphemes
                        .as_str()
                        .graphemes(true)
                        .exactly_one()
                        .map(|name| contains_str(kurz, name))
                        .unwrap_or(false)
                    {
                        return Some(name_gefunden());
                    }
                }
            }
        }
        None
    }

    pub(crate) fn parse_flag(
        &self,
        invertiere_präfix: &Vergleich<'_>,
        invertiere_infix: &Vergleich<'_>,
        arg: &OsStr,
    ) -> Option<bool> {
        let parse_invertiert =
            |lang: &NonEmpty<Vergleich<'_>>, lang_str: &Normalisiert<'_>| -> Option<bool> {
                if let Some(infix_name) = invertiere_präfix.strip_als_präfix_n(&lang_str) {
                    let infix_name_normalisiert = infix_name;
                    if let Some(negiert) =
                        invertiere_infix.strip_als_präfix_n(&infix_name_normalisiert)
                    {
                        if contains_str(lang, negiert.as_str()) {
                            return Some(false);
                        }
                    }
                }
                None
            };
        self.parse_flag_aux(|| true, parse_invertiert, arg)
    }

    #[inline(always)]
    pub(crate) fn parse_frühes_beenden(&self, arg: &OsStr) -> bool {
        self.parse_flag_aux(|| (), |_, _| None, arg).is_some()
    }

    pub(crate) fn parse_mit_wert<'t>(
        &self,
        wert_infix: &Vergleich<'_>,
        arg: &'t OsStr,
    ) -> Option<Option<Cow<'t, OsStr>>> {
        let Name { lang_präfix, lang, kurz_präfix, kurz } = self;
        let kurz_existiert = !kurz.is_empty();
        if let Some(string) = arg.to_str() {
            let normalisiert = Normalisiert::neu(string);
            if let Some(lang_str) = lang_präfix.strip_als_präfix_n(&normalisiert) {
                let suffixe = contains_prefix(lang, &lang_str);
                for suffix in suffixe {
                    let suffix_normalisiert = Normalisiert::neu_borrowed_unchecked(suffix);
                    if suffix.is_empty() {
                        return Some(None);
                    } else if let Some(wert_graphemes) =
                        wert_infix.strip_als_präfix_n(&suffix_normalisiert)
                    {
                        let wert_str = wert_graphemes.as_str();
                        let wert_länge = wert_str.len();
                        let wert_cow = match normalisiert.cow_ref() {
                            Cow::Borrowed(_) => {
                                let string_länge = string.len();
                                let start_index = string_länge - wert_länge - 1;
                                Cow::Borrowed(string[start_index..string_länge].as_ref())
                            },
                            Cow::Owned(_) => {
                                Cow::Owned(OsString::from(wert_graphemes.as_str().to_owned()))
                            },
                        };
                        return Some(Some(wert_cow));
                    }
                }
            } else if kurz_existiert {
                if let Some(kurz_str) = kurz_präfix.strip_als_präfix_n(&normalisiert) {
                    let mut kurz_graphemes = kurz_str.as_str().graphemes(true);
                    if kurz_graphemes.next().map(|name| contains_str(kurz, name)).unwrap_or(false) {
                        let rest = Normalisiert::neu_borrowed_unchecked(kurz_graphemes.as_str());
                        let wert_str = if rest.as_str().is_empty() {
                            None
                        } else {
                            let wert_str = wert_infix
                                .strip_als_präfix_n(&rest)
                                .unwrap_or_else(|| rest.clone());
                            let wert_länge = wert_str.as_str().len();
                            Some(match normalisiert.cow_ref() {
                                Cow::Borrowed(_) => {
                                    let string_länge = string.len();
                                    let start_index = string_länge - wert_länge - 1;
                                    Cow::Borrowed(string[start_index..string_länge].as_ref())
                                },
                                Cow::Owned(_) => {
                                    Cow::Owned(OsString::from(wert_str.cow().into_owned()))
                                },
                            })
                        };
                        return Some(wert_str);
                    }
                }
            }
        }
        None
    }

    /// Füge eine Regex-Darstellung der Langnamen zum übergebenen String hinzu.
    pub(crate) fn möglichkeiten_als_regex(
        head: &Vergleich<'_>,
        tail: &[Vergleich<'_>],
        s: &mut String,
    ) {
        if !tail.is_empty() {
            s.push('(')
        }
        s.push_str(head.as_str());
        for l in tail {
            s.push('|');
            s.push_str(l.as_str());
        }
        if !tail.is_empty() {
            s.push(')')
        }
    }
}

/// Beschreibung eines [Kommandozeilen-Arguments](EinzelArgument).
///
/// ## English synonym
/// [Description]
#[derive(Debug, Clone)]
pub struct Beschreibung<'t, T> {
    /// Namen um das Argument zu verwenden.
    ///
    /// ## English
    /// Name to use the argument.
    pub name: Name<'t>,

    /// Im automatischen Hilfetext angezeigte Beschreibung.
    ///
    /// ## English
    /// Description shown in the automatically created help text.
    pub hilfe: Option<&'t str>,

    /// Standard-Wert falls kein passendes Kommandozeilen-Argument verwendet wurde.
    ///
    /// ## English
    /// Default value if no fitting command line argument has been used.
    pub standard: Option<T>,
}

/// Description of a command line argument.
///
/// ## Deutsches Synonym
/// [Beschreibung]
pub type Description<'t, T> = Beschreibung<'t, T>;

impl<'t, T: Display> Beschreibung<'t, T> {
    #[inline(always)]
    pub(crate) fn als_string_beschreibung(self) -> (Beschreibung<'t, String>, Option<T>) {
        self.als_string_beschreibung_allgemein(ToString::to_string)
    }
}

impl<'t, T> Beschreibung<'t, T> {
    pub(crate) fn als_string_beschreibung_allgemein(
        self,
        anzeige: impl Fn(&T) -> String,
    ) -> (Beschreibung<'t, String>, Option<T>) {
        let Beschreibung { name: Name { lang_präfix, lang, kurz_präfix, kurz }, hilfe, standard } =
            self;
        let standard_str = standard.as_ref().map(anzeige);
        (
            Beschreibung {
                name: Name { lang_präfix, lang, kurz_präfix, kurz },
                hilfe,
                standard: standard_str,
            },
            standard,
        )
    }

    /// Konvertiere eine [Beschreibung] zu einem anderen Typ.
    ///
    /// ## English synonym
    /// [convert](Description::convert)
    pub fn konvertiere<S>(self, konvertiere: impl FnOnce(T) -> S) -> Beschreibung<'t, S> {
        let Beschreibung { name, hilfe, standard } = self;
        Beschreibung { name, hilfe, standard: standard.map(konvertiere) }
    }

    /// Convert a [Description] to a different type.
    ///
    /// ## Deutsches Synonym
    /// [konvertiere](Beschreibung::konvertiere)
    #[inline(always)]
    pub fn convert<S>(self, convert: impl FnOnce(T) -> S) -> Description<'t, S> {
        self.konvertiere(convert)
    }
}

pub(crate) fn contains_str<'t>(
    collection: impl IntoIterator<Item = &'t Vergleich<'t>>,
    gesucht: &str,
) -> bool {
    collection.into_iter().any(|ziel| ziel.eq(gesucht))
}

pub(crate) fn contains_prefix<'t>(
    collection: impl 't + IntoIterator<Item = &'t Vergleich<'t>>,
    input: &'t Normalisiert<'t>,
) -> impl 't + Iterator<Item = &'t str> {
    collection.into_iter().filter_map(|ziel| ziel.strip_als_präfix(input))
}

/// Mindestens ein String als Definition für den vollen Namen.
///
/// ## English
/// At least one String as definition for the full name.
pub trait LangNamen<'t> {
    /// Konvertiere in ein [NonEmpty].
    ///
    /// ## English
    /// Convert into a [NonEmpty].
    fn lang_namen(self) -> NonEmpty<Vergleich<'t>>;
}

macro_rules! impl_lang_namen {
    ($type: ty) => {
        impl<'t> LangNamen<'t> for $type {
            fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
                NonEmpty::singleton(self.into())
            }
        }

        impl<'t> LangNamen<'t> for ($type, Case) {
            fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
                NonEmpty::singleton(self.into())
            }
        }

        impl<'t> LangNamen<'t> for NonEmpty<$type> {
            fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
                let NonEmpty { head, tail } = self;
                NonEmpty { head: head.into(), tail: tail.into_iter().map(Into::into).collect() }
            }
        }

        impl<'t> LangNamen<'t> for NonEmpty<($type, Case)> {
            fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
                let NonEmpty { head, tail } = self;
                NonEmpty { head: head.into(), tail: tail.into_iter().map(Into::into).collect() }
            }
        }
    };
}

impl_lang_namen! {String}
impl_lang_namen! {&'t str}
impl_lang_namen! {Normalisiert<'t>}

impl<'t> LangNamen<'t> for Vergleich<'t> {
    fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
        NonEmpty::singleton(self)
    }
}

impl<'t> LangNamen<'t> for NonEmpty<Vergleich<'t>> {
    fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
        self
    }
}

impl<'t, S: AsRef<str>> LangNamen<'t> for &'t NonEmpty<S> {
    fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
        let NonEmpty { head, tail } = self;
        NonEmpty {
            head: head.as_ref().into(),
            tail: tail.into_iter().map(|s| s.as_ref().into()).collect(),
        }
    }
}

/// Beliebige Anzahl an Strings für den kurzen Namen.
///
/// ## English
/// Arbitrary number of strings for the short name.
pub trait KurzNamen<'t> {
    /// Konvertiere in einen [Vec].
    ///
    /// ## English
    /// Convert into a [Vec].
    fn kurz_namen(self) -> Vec<Vergleich<'t>>;
}

macro_rules! impl_kurz_namen {
    ($type: ty) => {
        impl<'t> KurzNamen<'t> for $type {
            fn kurz_namen(self) -> Vec<Vergleich<'t>> {
                vec![self.into()]
            }
        }

        impl<'t> KurzNamen<'t> for ($type, Case) {
            fn kurz_namen(self) -> Vec<Vergleich<'t>> {
                vec![self.into()]
            }
        }

        macro_rules! impl_into_iter {
            ($collection: ident) => {
                impl<'t> KurzNamen<'t> for $collection<$type> {
                    fn kurz_namen(self) -> Vec<Vergleich<'t>> {
                        self.into_iter().map(Into::into).collect()
                    }
                }

                impl<'t> KurzNamen<'t> for $collection<($type, Case)> {
                    fn kurz_namen(self) -> Vec<Vergleich<'t>> {
                        self.into_iter().map(Into::into).collect()
                    }
                }
            };
        }

        impl_into_iter! {Option}
        impl_into_iter! {Vec}
        impl_into_iter! {NonEmpty}
    };
}

impl_kurz_namen! {String}
impl_kurz_namen! {&'t str}
impl_kurz_namen! {Normalisiert<'t>}

impl<'t> KurzNamen<'t> for Vec<Vergleich<'t>> {
    fn kurz_namen(self) -> Vec<Vergleich<'t>> {
        self
    }
}

impl<'t, S: AsRef<str>> KurzNamen<'t> for &'t Vec<S> {
    fn kurz_namen(self) -> Vec<Vergleich<'t>> {
        self.into_iter().map(|s| s.as_ref().into()).collect()
    }
}

impl<'t, T> Beschreibung<'t, T> {
    /// Erzeuge eine neue [Beschreibung].
    ///
    /// ## English synonym
    /// [new](Description::new)
    pub fn neu(
        lang_präfix: impl Into<Vergleich<'t>>,
        lang: impl LangNamen<'t>,
        kurz_präfix: impl Into<Vergleich<'t>>,
        kurz: impl KurzNamen<'t>,
        hilfe: Option<&'t str>,
        standard: Option<T>,
    ) -> Beschreibung<'t, T> {
        Beschreibung {
            name: Name {
                lang_präfix: lang_präfix.into(),
                lang: lang.lang_namen(),
                kurz_präfix: kurz_präfix.into(),
                kurz: kurz.kurz_namen(),
            },
            hilfe,
            standard,
        }
    }

    /// Create a new [Description].
    ///
    /// ## Deutsches Synonym
    /// [neu](Beschreibung::neu)
    #[inline(always)]
    pub fn new(
        long_prefix: Compare<'t>,
        long: impl LangNamen<'t>,
        short_prefix: Compare<'t>,
        short: impl KurzNamen<'t>,
        help: Option<&'t str>,
        default: Option<T>,
    ) -> Description<'t, T> {
        Beschreibung::neu(long_prefix, long, short_prefix, short, help, default)
    }

    /// Erzeuge eine neue [Beschreibung].
    ///
    /// ## English synonym
    /// [new_with_language](Description::new_with_language)
    #[inline(always)]
    pub fn neu_mit_sprache(
        lang: impl LangNamen<'t>,
        kurz: impl KurzNamen<'t>,
        hilfe: Option<&'t str>,
        standard: Option<T>,
        sprache: Sprache,
    ) -> Beschreibung<'t, T> {
        Beschreibung::neu(sprache.lang_präfix, lang, sprache.kurz_präfix, kurz, hilfe, standard)
    }

    /// Create a new [Description].
    ///
    /// ## Deutsches Synonym
    /// [neu_mit_sprache](Description::neu_mit_sprache)
    #[inline(always)]
    pub fn new_with_language(
        long: impl LangNamen<'t>,
        short: impl KurzNamen<'t>,
        help: Option<&'t str>,
        default: Option<T>,
        language: Language,
    ) -> Beschreibung<'t, T> {
        Beschreibung::neu_mit_sprache(long, short, help, default, language)
    }
}

/// Konfiguration eines Kommandozeilen-Arguments.
///
/// ## English synonym
/// [Configuration]
#[derive(Debug)]
pub enum Konfiguration<'t> {
    /// Es handelt sich um ein Flag-Argument.
    ///
    /// ## English
    /// It is a flag argument.
    Flag {
        /// Allgemeine Beschreibung des Arguments.
        ///
        /// ## English
        /// General description of the argument.
        beschreibung: Beschreibung<'t, String>,

        /// Präfix und folgendes Infix zum invertieren des Flag-Arguments.
        /// Der Wert ist [None], wenn es sich um eine Flag die zu frühem beenden führt handelt.
        ///
        /// ## English
        /// Prefix and following infix to invert the flag argument.
        /// The value is [None] if it is a flag causing an early exit.
        invertiere_präfix_infix: Option<(Vergleich<'t>, Vergleich<'t>)>,
    },

    /// Es handelt sich um ein Wert-Argument.
    ///
    /// ## English
    /// It is a value argument.
    Wert {
        /// Allgemeine Beschreibung des Arguments.
        ///
        /// ## English
        /// General description of the argument.
        beschreibung: Beschreibung<'t, String>,

        /// Infix um einen Wert im selben Argument wie den Namen anzugeben.
        ///
        /// ## English
        /// Infix to give a value in the same argument as the name.
        wert_infix: Vergleich<'t>,

        /// Meta-Variable im Hilfe-Text.
        ///
        /// ## English
        /// Meta-variable used in the help-text.
        meta_var: &'t str,

        /// String-Darstellung der erlaubten Werte.
        ///
        /// ## English
        /// String-representation of the allowed values.
        mögliche_werte: Option<NonEmpty<String>>,
    },
}

/// Configuration of a command line argument.
///
/// ## Deutsches Synonym
/// [Konfiguration]
pub type Configuration<'t> = Konfiguration<'t>;
