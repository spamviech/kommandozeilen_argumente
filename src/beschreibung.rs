//! Beschreibung eines Arguments.

use std::{convert::AsRef, fmt::Display};

use nonempty::NonEmpty;

use crate::unicode::{Case, Normalisiert};

// TODO besseren Namen (heh) finden, wird auch für invertiere_präfix verwendet
/// Lang- oder Kurz-Name eines Kommandozeilen-Arguments.
///
/// ## English
/// Long or short name of a command line argument.
pub type Name<'t> = (Normalisiert<'t>, Case);

// TODO erwähne verschmelzen von Flag-Kurzformen?
// TODO Lang/Kurz-Präfix
// TODO case_sensitive für (jeden?) Namen
/// Beschreibung eines Kommandozeilen-Arguments.
///
/// ## English synonym
/// [Description]
#[derive(Debug, Clone)]
pub struct Beschreibung<'t, T> {
    /// Voller Name, wird nach zwei Minus angegeben "--<lang>".
    ///
    /// ## English
    /// Full Name, given after two minus characters "--<lang>"
    pub lang: NonEmpty<Name<'t>>,

    /// Kurzer Name, wird nach einem Minus angegeben "-<kurz>".
    /// Kurznamen länger als ein [Grapheme](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// werden nicht unterstützt.
    ///
    /// ## English
    /// Short name, given after one minus character "-<kurz>"
    /// Short names longer than a [Grapheme](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// are not supported.
    pub kurz: Vec<Name<'t>>,

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
        let Beschreibung { lang, kurz, hilfe, standard } = self;
        let standard_string = standard.as_ref().map(anzeige);
        (Beschreibung { lang, kurz, hilfe, standard: standard_string }, standard)
    }

    /// Konvertiere eine [Beschreibung] zu einem anderen Typ.
    ///
    /// ## English synonym
    /// [convert](Description::convert)
    pub fn konvertiere<S>(self, konvertiere: impl FnOnce(T) -> S) -> Beschreibung<'t, S> {
        let Beschreibung { lang, kurz, hilfe, standard } = self;
        Beschreibung { lang, kurz, hilfe, standard: standard.map(konvertiere) }
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

pub(crate) fn contains_str<'t>(iter: impl Iterator<Item = &'t Name<'t>>, gesucht: &str) -> bool {
    iter.any(|(element, case)| element.eq(gesucht, *case))
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
    fn lang_namen(self) -> NonEmpty<Name<'t>>;
}

macro_rules! impl_lang_namen {
    ($type: ty) => {
        impl<'t> LangNamen<'t> for $type {
            fn lang_namen(self) -> NonEmpty<Name<'t>> {
                NonEmpty::singleton((Normalisiert::neu(self), Case::Insensitive))
            }
        }

        impl<'t> LangNamen<'t> for ($type, Case) {
            fn lang_namen(self) -> NonEmpty<Name<'t>> {
                let (s, case) = self;
                NonEmpty::singleton((Normalisiert::neu(s), case))
            }
        }

        impl<'t> LangNamen<'t> for NonEmpty<$type> {
            fn lang_namen(self) -> NonEmpty<Name<'t>> {
                let NonEmpty { head, tail } = self;
                NonEmpty {
                    head: (Normalisiert::neu(head), Case::Insensitive),
                    tail: tail
                        .into_iter()
                        .map(|s| (Normalisiert::neu(s), Case::Insensitive))
                        .collect(),
                }
            }
        }

        impl<'t> LangNamen<'t> for NonEmpty<($type, Case)> {
            fn lang_namen(self) -> NonEmpty<Name<'t>> {
                let NonEmpty { head: (h_s, h_case), tail } = self;
                NonEmpty {
                    head: (Normalisiert::neu(h_s), h_case),
                    tail: tail.into_iter().map(|(s, case)| (Normalisiert::neu(s), case)).collect(),
                }
            }
        }
    };
}

impl_lang_namen! {String}
impl_lang_namen! {&'t str}

impl<'t> LangNamen<'t> for Normalisiert<'t> {
    fn lang_namen(self) -> NonEmpty<Name<'t>> {
        NonEmpty::singleton((self, Case::Insensitive))
    }
}

impl<'t> LangNamen<'t> for Name<'t> {
    fn lang_namen(self) -> NonEmpty<Name<'t>> {
        NonEmpty::singleton(self)
    }
}

impl<'t> LangNamen<'t> for NonEmpty<Normalisiert<'t>> {
    fn lang_namen(self) -> NonEmpty<Name<'t>> {
        let NonEmpty { head, tail } = self;
        NonEmpty {
            head: (head, Case::Insensitive),
            tail: tail.into_iter().map(|n| (n, Case::Insensitive)).collect(),
        }
    }
}

impl<'t> LangNamen<'t> for NonEmpty<Name<'t>> {
    fn lang_namen(self) -> NonEmpty<Name<'t>> {
        self
    }
}

impl<'t, S: AsRef<str>> LangNamen<'t> for &'t NonEmpty<S> {
    fn lang_namen(self) -> NonEmpty<Name<'t>> {
        let NonEmpty { head, tail } = self;
        NonEmpty {
            head: (Normalisiert::neu(head.as_ref()), Case::Insensitive),
            tail: tail
                .into_iter()
                .map(|s| (Normalisiert::neu(s.as_ref()), Case::Insensitive))
                .collect(),
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
    fn kurz_namen(self) -> Vec<Name<'t>>;
}

macro_rules! impl_kurz_namen {
    ($type: ty) => {
        impl<'t> KurzNamen<'t> for $type {
            fn kurz_namen(self) -> Vec<Name<'t>> {
                vec![(Normalisiert::neu(self), Case::Insensitive)]
            }
        }

        impl<'t> KurzNamen<'t> for ($type, Case) {
            fn kurz_namen(self) -> Vec<Name<'t>> {
                let (s, case) = self;
                vec![(Normalisiert::neu(s), case)]
            }
        }

        macro_rules! impl_into_iter {
            ($collection: ident) => {
                impl<'t> KurzNamen<'t> for $collection<$type> {
                    fn kurz_namen(self) -> Vec<Name<'t>> {
                        self.into_iter()
                            .map(|s| (Normalisiert::neu(s), Case::Insensitive))
                            .collect()
                    }
                }

                impl<'t> KurzNamen<'t> for $collection<($type, Case)> {
                    fn kurz_namen(self) -> Vec<Name<'t>> {
                        self.into_iter().map(|(s, case)| (Normalisiert::neu(s), case)).collect()
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

impl<'t> KurzNamen<'t> for Normalisiert<'t> {
    fn kurz_namen(self) -> Vec<Name<'t>> {
        vec![(self, Case::Insensitive)]
    }
}

impl<'t> KurzNamen<'t> for Name<'t> {
    fn kurz_namen(self) -> Vec<Name<'t>> {
        vec![self]
    }
}

macro_rules! impl_kurz_namen_into_iter {
    ($collection: ident) => {
        impl<'t> KurzNamen<'t> for $collection<Normalisiert<'t>> {
            fn kurz_namen(self) -> Vec<Name<'t>> {
                self.into_iter().map(|s| (s, Case::Insensitive)).collect()
            }
        }

        impl<'t> KurzNamen<'t> for $collection<Name<'t>> {
            fn kurz_namen(self) -> Vec<Name<'t>> {
                self.into_iter().collect()
            }
        }
    };
}

impl_kurz_namen_into_iter! {Option}
impl_kurz_namen_into_iter! {NonEmpty}

impl<'t> KurzNamen<'t> for Vec<Normalisiert<'t>> {
    fn kurz_namen(self) -> Vec<Name<'t>> {
        self.into_iter().map(|s| (s, Case::Insensitive)).collect()
    }
}

impl<'t> KurzNamen<'t> for Vec<Name<'t>> {
    fn kurz_namen(self) -> Vec<Name<'t>> {
        self
    }
}

impl<'t, S: AsRef<str>> KurzNamen<'t> for &'t Vec<S> {
    fn kurz_namen(self) -> Vec<Name<'t>> {
        self.into_iter().map(|s| (Normalisiert::neu(s.as_ref()), Case::Insensitive)).collect()
    }
}

impl<'t, T> Beschreibung<'t, T> {
    /// Erzeuge eine neue [Beschreibung].
    ///
    /// ## English synonym
    /// [new](Description::new)
    pub fn neu(
        lang: impl LangNamen<'t>,
        kurz: impl KurzNamen<'t>,
        hilfe: Option<&'t str>,
        standard: Option<T>,
    ) -> Beschreibung<'t, T> {
        Beschreibung { lang: lang.lang_namen(), kurz: kurz.kurz_namen(), hilfe, standard }
    }

    /// Create a new [Description].
    ///
    /// ## Deutsches Synonym
    /// [neu](Beschreibung::neu)
    #[inline(always)]
    pub fn new(
        long: impl LangNamen<'t>,
        short: impl KurzNamen<'t>,
        help: Option<&'t str>,
        default: Option<T>,
    ) -> Description<'t, T> {
        Beschreibung::neu(long, short, help, default)
    }
}

// TODO Invertiere-Flag-Infix, Wert-Infix
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

        /// Präfix zum invertieren des Flag-Arguments.
        ///
        /// ## English
        /// Prefix to invert the flag argument.
        invertiere_präfix: Option<Name<'t>>,
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
