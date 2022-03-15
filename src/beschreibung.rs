//! Beschreibung eines Arguments.

use std::{convert::AsRef, fmt::Display};

use nonempty::NonEmpty;

use crate::unicode::Normalisiert;

// TODO erwähne verschmelzen von Flag-Kurzformen?
// TODO Lang/Kurz-Präfix
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
    pub lang: NonEmpty<Normalisiert<'t>>,
    /// Kurzer Name, wird nach einem Minus angegeben "-<kurz>".
    /// Kurznamen länger als ein [Grapheme](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// werden nicht unterstützt.
    ///
    /// ## English
    /// Short name, given after one minus character "-<kurz>"
    /// Short names longer than a [Grapheme](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// are not supported.
    pub kurz: Vec<Normalisiert<'t>>,
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

macro_rules! contains_str {
    ($collection: expr, $gesucht: expr) => {
        $collection.iter().any(|element| element.as_ref() == $gesucht)
    };
}
pub(crate) use contains_str;

/// Mindestens ein String als Definition für den vollen Namen.
///
/// ## English
/// At least one String as definition for the full name.
pub trait LangNamen<'t> {
    /// Konvertiere in ein [NonEmpty].
    ///
    /// ## English
    /// Convert into a [NonEmpty].
    fn lang_namen(self) -> NonEmpty<Normalisiert<'t>>;
}

impl<'t> LangNamen<'t> for String {
    fn lang_namen(self) -> NonEmpty<Normalisiert<'t>> {
        NonEmpty::singleton(Normalisiert::neu(self))
    }
}

impl<'t> LangNamen<'t> for &'t str {
    fn lang_namen(self) -> NonEmpty<Normalisiert<'t>> {
        NonEmpty::singleton(Normalisiert::neu(self))
    }
}

impl<'t> LangNamen<'t> for NonEmpty<String> {
    fn lang_namen(self) -> NonEmpty<Normalisiert<'t>> {
        let NonEmpty { head, tail } = self;
        NonEmpty {
            head: Normalisiert::neu(head),
            tail: tail.into_iter().map(Normalisiert::neu).collect(),
        }
    }
}

impl<'t> LangNamen<'t> for NonEmpty<&'t str> {
    fn lang_namen(self) -> NonEmpty<Normalisiert<'t>> {
        let NonEmpty { head, tail } = self;
        NonEmpty {
            head: Normalisiert::neu(head),
            tail: tail.into_iter().map(Normalisiert::neu).collect(),
        }
    }
}

impl<'t, S: AsRef<str>> LangNamen<'t> for &'t NonEmpty<S> {
    fn lang_namen(self) -> NonEmpty<Normalisiert<'t>> {
        let NonEmpty { head, tail } = self;
        NonEmpty {
            head: Normalisiert::neu(head),
            tail: tail.into_iter().map(Normalisiert::neu).collect(),
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
    fn kurz_namen(self) -> Vec<Normalisiert<'t>>;
}

impl<'t> KurzNamen<'t> for Option<String> {
    fn kurz_namen(self) -> Vec<Normalisiert<'t>> {
        self.into_iter().map(Normalisiert::neu).collect()
    }
}

impl<'t> KurzNamen<'t> for Option<&'t str> {
    fn kurz_namen(self) -> Vec<Normalisiert<'t>> {
        self.into_iter().map(Normalisiert::neu).collect()
    }
}

impl<'t> KurzNamen<'t> for String {
    fn kurz_namen(self) -> Vec<Normalisiert<'t>> {
        vec![Normalisiert::neu(self)]
    }
}

impl<'t> KurzNamen<'t> for &'t str {
    fn kurz_namen(self) -> Vec<Normalisiert<'t>> {
        vec![Normalisiert::neu(self)]
    }
}

impl<'t> KurzNamen<'t> for NonEmpty<String> {
    fn kurz_namen(self) -> Vec<Normalisiert<'t>> {
        self.into_iter().map(Normalisiert::neu).collect()
    }
}

impl<'t, S: AsRef<str>> KurzNamen<'t> for &'t Vec<S> {
    fn kurz_namen(self) -> Vec<Normalisiert<'t>> {
        self.into_iter().map(Normalisiert::neu).collect()
    }
}

impl<'t> KurzNamen<'t> for Vec<String> {
    fn kurz_namen(self) -> Vec<Normalisiert<'t>> {
        self.into_iter().map(Normalisiert::neu).collect()
    }
}

impl<'t> KurzNamen<'t> for Vec<&'t str> {
    fn kurz_namen(self) -> Vec<Normalisiert<'t>> {
        self.into_iter().map(Normalisiert::neu).collect()
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
        hilfe: Option<impl 't + AsRef<str>>,
        standard: Option<T>,
    ) -> Beschreibung<'t, T> {
        Beschreibung {
            lang: lang.lang_namen(),
            kurz: kurz.kurz_namen(),
            hilfe: hilfe.map(|h| h.as_ref()),
            standard,
        }
    }

    /// Create a new [Description].
    ///
    /// ## Deutsches Synonym
    /// [neu](Beschreibung::neu)
    #[inline(always)]
    pub fn new(
        long: impl LangNamen<'t>,
        short: impl KurzNamen<'t>,
        help: Option<impl 't + AsRef<str>>,
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
        invertiere_präfix: Option<Normalisiert<'t>>,
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
