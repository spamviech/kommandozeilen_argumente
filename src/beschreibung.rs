//! Beschreibung eines Arguments.

use std::{convert::AsRef, fmt::Display};

use nonempty::NonEmpty;
use unicode_segmentation::{GraphemeCursor, Graphemes};

use crate::{
    sprache::{Language, Sprache},
    unicode::{Case, Compare, Normalisiert, Vergleich},
};

/// Beschreibung eines Kommandozeilen-Arguments.
///
/// ## English synonym
/// [Description]
#[derive(Debug, Clone)]
pub struct Beschreibung<'t, T> {
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
        let Beschreibung { lang_präfix, lang, kurz_präfix, kurz, hilfe, standard } = self;
        let standard_str = standard.as_ref().map(anzeige);
        (
            Beschreibung { lang_präfix, lang, kurz_präfix, kurz, hilfe, standard: standard_str },
            standard,
        )
    }

    /// Konvertiere eine [Beschreibung] zu einem anderen Typ.
    ///
    /// ## English synonym
    /// [convert](Description::convert)
    pub fn konvertiere<S>(self, konvertiere: impl FnOnce(T) -> S) -> Beschreibung<'t, S> {
        let Beschreibung { lang_präfix, lang, kurz_präfix, kurz, hilfe, standard } = self;
        Beschreibung {
            lang_präfix,
            lang,
            kurz_präfix,
            kurz,
            hilfe,
            standard: standard.map(konvertiere),
        }
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
    iter: impl Iterator<Item = &'t Vergleich<'t>>,
    gesucht: &str,
) -> bool {
    iter.any(|ziel| ziel.eq(gesucht))
}

pub(crate) fn contains_prefix<'t>(
    iter: impl 't + Iterator<Item = &'t Vergleich<'t>>,
    input: &'t str,
) -> impl 't + Iterator<Item = Graphemes<'t>> {
    iter.filter_map(move |ziel| ziel.strip_als_präfix(input))
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
            lang_präfix: lang_präfix.into(),
            lang: lang.lang_namen(),
            kurz_präfix: kurz_präfix.into(),
            kurz: kurz.kurz_namen(),
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
        /// Prefix an following infix to invert the flag argument.
        /// The value is [None] if it is a flag causing an early exit.
        invertiere_präfix_infix: Option<Vergleich<'t>>,
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
