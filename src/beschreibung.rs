//! Beschreibung eines Arguments.

use std::{
    borrow::Cow,
    convert::AsRef,
    ffi::{OsStr, OsString},
};

use itertools::Itertools as _;
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
    /// Präfix vor dem Lang-Namen.
    ///
    /// ## English
    /// Prefix before the long name.
    pub lang_präfix: Vergleich<'t>,

    /// Voller Name, wird nach `lang_präfix` angegeben.
    ///
    /// ## English
    /// Full Name, given after `lang_präfix`.
    pub lang: NonEmpty<Vergleich<'t>>,

    /// Präfix vor dem Kurz-Namen.
    ///
    /// ## English
    /// Prefix before the short name.
    pub kurz_präfix: Vergleich<'t>,

    /// Kurzer Name, wird nach `kurz_präfix` angegeben.
    /// Bei Flag-Argumenten können Kurz-Namen mit identischen `kurz_präfix` zusammen angegeben werden,
    /// zum Beispiel "-fgh".
    /// Kurznamen länger als ein [`Grapheme`](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// werden nicht unterstützt.
    ///
    /// ## English
    /// Short name, given after `short_präfix`.
    /// Flag arguments with identical `kurz_präfix` may be given at once, e.g. "-fgh".
    /// Short names longer than a [`Grapheme`](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// are not supported.
    pub kurz: Vec<Vergleich<'t>>,
}

impl Name<'_> {
    /// Hilfs-Methode für [`parse_flag`](Name::parse_flag) und seine Varianten.
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
                #[allow(clippy::redundant_else)]
                if contains_str(lang, lang_str.as_str()) {
                    return Some(name_gefunden());
                } else if let Some(wert) = parse_invertiert(lang, lang_str) {
                    return Some(wert);
                } else {
                    // kein match für {lang_präfix}[invertiert_infix]{lang_name}
                }
            } else if name_kurz_existiert {
                // TODO Kurz-Namen verschmelzen
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
            } else {
                // kein match für "{lang_präfix}.*" und es gibt keine Kurz-Namen.
            }
        }
        None
    }

    /// Parse den namen als Flag.
    #[inline]
    pub(crate) fn parse_flag(
        &self,
        invertiere_präfix: &Vergleich<'_>,
        invertiere_infix: &Vergleich<'_>,
        arg: &OsStr,
    ) -> Option<bool> {
        let parse_invertiert =
            |lang: &NonEmpty<Vergleich<'_>>, lang_str: &Normalisiert<'_>| -> Option<bool> {
                if let Some(infix_name) = invertiere_präfix.strip_als_präfix_n(lang_str) {
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

    /// Parse den namen als Flag, die ein frühes beenden auslöst.
    #[inline]
    pub(crate) fn parse_frühes_beenden(&self, arg: &OsStr) -> bool {
        self.parse_flag_aux(|| (), |_, _| None, arg).is_some()
    }

    /// Parse den Namen als Wert.
    #[allow(clippy::option_option)]
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
                let suffixe = filter_prefix(lang, &lang_str);
                for suffix in suffixe {
                    let suffix_normalisiert = Normalisiert::neu(suffix);
                    #[allow(clippy::redundant_else)]
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
                                // Berechne Index aus suffix-Länge
                                #[allow(clippy::arithmetic_side_effects)]
                                let start_index = string_länge - wert_länge - 1;
                                #[allow(clippy::string_slice, clippy::indexing_slicing)]
                                Cow::Borrowed(string[start_index..string_länge].as_ref())
                            },
                            Cow::Owned(_) => {
                                Cow::Owned(OsString::from(wert_graphemes.as_str().to_owned()))
                            },
                        };
                        return Some(Some(wert_cow));
                    } else {
                        // Suffix ist nicht leer, beginnt aber nicht mit wert_infix
                    }
                }
            } else if kurz_existiert {
                if let Some(kurz_str) = kurz_präfix.strip_als_präfix_n(&normalisiert) {
                    let mut kurz_graphemes = kurz_str.as_str().graphemes(true);
                    if kurz_graphemes.next().is_some_and(|name| contains_str(kurz, name)) {
                        let rest = Normalisiert::neu(kurz_graphemes.as_str());
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
                                    // Berechne Index aus suffix-Länge
                                    #[allow(clippy::arithmetic_side_effects)]
                                    let start_index = string_länge - wert_länge - 1;
                                    #[allow(clippy::string_slice, clippy::indexing_slicing)]
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
            } else {
                // kein match für "{lang_name_präfix}.*" und Argument hat keinen kurz_namen.
            }
        }
        None
    }

    /// Füge eine Regex-Darstellung der Langnamen zum übergebenen String hinzu.
    pub(crate) fn möglichkeiten_als_regex(
        head: &Vergleich<'_>,
        tail: &[Vergleich<'_>],
        string: &mut String,
    ) {
        if !tail.is_empty() {
            string.push('(');
        }
        string.push_str(head.as_str());
        for elem in tail {
            string.push('|');
            string.push_str(elem.as_str());
        }
        if !tail.is_empty() {
            string.push(')');
        }
    }
}

/// Beschreibung eines [`Kommandozeilen-Arguments`](EinzelArgument).
///
/// ## English synonym
/// [`Description`]
#[must_use]
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
/// [`Beschreibung`]
pub type Description<'t, T> = Beschreibung<'t, T>;

impl<'t, T> Beschreibung<'t, T> {
    /// Konvertiere eine [`Beschreibung`] zu einem anderen Typ.
    ///
    /// ## English synonym
    /// [`convert`](Description::convert)
    #[inline]
    pub fn konvertiere<S>(self, konvertiere: impl FnOnce(T) -> S) -> Beschreibung<'t, S> {
        let Beschreibung { name, hilfe, standard } = self;
        Beschreibung { name, hilfe, standard: standard.map(konvertiere) }
    }

    /// Convert a [`Description`] to a different type.
    ///
    /// ## Deutsches Synonym
    /// [`konvertiere`](Beschreibung::konvertiere)
    #[inline]
    pub fn convert<S>(self, convert: impl FnOnce(T) -> S) -> Description<'t, S> {
        self.konvertiere(convert)
    }

    /// Konvertiere von `&Beschreibung<T>` zu `Beschreibung<&T>`.
    ///
    /// ## English synonym
    /// Convert from `&Description<T>` to `Description<&T>`.
    #[inline]
    pub fn as_ref(&self) -> Beschreibung<'t, &T> {
        let Beschreibung { name, hilfe, standard } = self;
        Beschreibung { name: name.clone(), hilfe: *hilfe, standard: standard.as_ref() }
    }
}

/// Ist `gesucht` in der `collection` enthalten?
pub(crate) fn contains_str<'t>(
    collection: impl IntoIterator<Item = &'t Vergleich<'t>>,
    gesucht: &str,
) -> bool {
    collection.into_iter().any(|ziel| ziel.eq(gesucht))
}

/// Gebe alle Strings der `collection` mit `input` als Präfix zurück.
pub(crate) fn filter_prefix<'t>(
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
    /// Konvertiere in ein [`NonEmpty`].
    ///
    /// ## English
    /// Convert into a [`NonEmpty`].
    fn lang_namen(self) -> NonEmpty<Vergleich<'t>>;
}

/// Implementiere [`LangNamen`] für String-artige Typen.
macro_rules! impl_lang_namen {
    ($type: ty) => {
        impl<'t> LangNamen<'t> for $type {
            #[inline]
            fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
                NonEmpty::singleton(self.into())
            }
        }

        impl<'t> LangNamen<'t> for ($type, Case) {
            #[inline]
            fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
                NonEmpty::singleton(self.into())
            }
        }

        impl<'t> LangNamen<'t> for NonEmpty<$type> {
            #[inline]
            fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
                let NonEmpty { head, tail } = self;
                NonEmpty { head: head.into(), tail: tail.into_iter().map(Into::into).collect() }
            }
        }

        impl<'t> LangNamen<'t> for NonEmpty<($type, Case)> {
            #[inline]
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
    #[inline]
    fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
        NonEmpty::singleton(self)
    }
}

impl<'t> LangNamen<'t> for NonEmpty<Vergleich<'t>> {
    #[inline]
    fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
        self
    }
}

impl<'t, S: AsRef<str>> LangNamen<'t> for &'t NonEmpty<S> {
    #[inline]
    fn lang_namen(self) -> NonEmpty<Vergleich<'t>> {
        let NonEmpty { head, tail } = self;
        NonEmpty {
            head: head.as_ref().into(),
            tail: tail.iter().map(|string| string.as_ref().into()).collect(),
        }
    }
}

/// Beliebige Anzahl an Strings für den kurzen Namen.
///
/// ## English
/// Arbitrary number of strings for the short name.
pub trait KurzNamen<'t> {
    /// Konvertiere in einen [`Vec`].
    ///
    /// ## English
    /// Convert into a [`Vec`].
    fn kurz_namen(self) -> Vec<Vergleich<'t>>;
}

/// Implementiere [`KurzNamen`] für String-artige Typen.
macro_rules! impl_kurz_namen {
    ($type: ty) => {
        impl<'t> KurzNamen<'t> for $type {
            #[inline]
            fn kurz_namen(self) -> Vec<Vergleich<'t>> {
                vec![self.into()]
            }
        }

        impl<'t> KurzNamen<'t> for ($type, Case) {
            #[inline]
            fn kurz_namen(self) -> Vec<Vergleich<'t>> {
                vec![self.into()]
            }
        }

        macro_rules! impl_into_iter {
            ($collection: ident) => {
                impl<'t> KurzNamen<'t> for $collection<$type> {
                    #[inline]
                    fn kurz_namen(self) -> Vec<Vergleich<'t>> {
                        self.into_iter().map(Into::into).collect()
                    }
                }

                impl<'t> KurzNamen<'t> for $collection<($type, Case)> {
                    #[inline]
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
    #[inline]
    fn kurz_namen(self) -> Vec<Vergleich<'t>> {
        self
    }
}

impl<'t, S: AsRef<str>> KurzNamen<'t> for &'t Vec<S> {
    #[inline]
    fn kurz_namen(self) -> Vec<Vergleich<'t>> {
        self.iter().map(|string| string.as_ref().into()).collect()
    }
}

impl<'t, T> Beschreibung<'t, T> {
    /// Erzeuge eine neue [`Beschreibung`].
    ///
    /// ## English synonym
    /// [`new`](Description::new)
    #[inline]
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

    /// Create a new [`Description`].
    ///
    /// ## Deutsches Synonym
    /// [`neu`](Beschreibung::neu)
    #[inline]
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

    /// Erzeuge eine neue [`Beschreibung`].
    ///
    /// ## English synonym
    /// [`new_with_language`](Description::new_with_language)
    #[inline]
    pub fn neu_mit_sprache(
        lang: impl LangNamen<'t>,
        kurz: impl KurzNamen<'t>,
        hilfe: Option<&'t str>,
        standard: Option<T>,
        sprache: Sprache,
    ) -> Beschreibung<'t, T> {
        Beschreibung::neu(sprache.lang_präfix, lang, sprache.kurz_präfix, kurz, hilfe, standard)
    }

    /// Create a new [`Description`].
    ///
    /// ## Deutsches Synonym
    /// [`neu_mit_sprache`](Description::neu_mit_sprache)
    #[inline]
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
/// [`Configuration`]
#[derive(Debug)]
#[must_use]
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
        /// Der Wert ist [`None`], wenn es sich um eine Flag die zu frühem beenden führt handelt.
        ///
        /// ## English
        /// Prefix and following infix to invert the flag argument.
        /// The value is [`None`] if it is a flag causing an early exit.
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
/// [`Konfiguration`]
pub type Configuration<'t> = Konfiguration<'t>;
