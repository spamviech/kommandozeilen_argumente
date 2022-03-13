//! Beschreibung eines Arguments.

use std::fmt::Display;
use std::ops::Deref;

use nonempty::NonEmpty;

// TODO erwähne verschmelzen von Flag-Kurzformen?
/// Beschreibung eines Kommandozeilen-Arguments.
///
/// ## English synonym
/// [Description]
#[derive(Debug, Clone)]
pub struct Beschreibung<T> {
    /// Voller Name, wird nach zwei Minus angegeben "--<lang>".
    ///
    /// ## English
    /// Full Name, given after two minus characters "--<lang>"
    pub lang: NonEmpty<String>,
    /// Kurzer Name, wird nach einem Minus angegeben "-<kurz>".
    /// Kurznamen länger als ein [Grapheme](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// werden nicht unterstützt.
    ///
    /// ## English
    /// Short name, given after one minus character "-<kurz>"
    /// Short names longer than a [Grapheme](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// are not supported.
    pub kurz: Vec<String>,
    /// Im automatischen Hilfetext angezeigte Beschreibung.
    ///
    /// ## English
    /// Description shown in the automatically created help text.
    pub hilfe: Option<String>,
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
pub type Description<T> = Beschreibung<T>;

impl<T: Display> Beschreibung<T> {
    #[inline(always)]
    pub(crate) fn als_string_beschreibung(self) -> (Beschreibung<String>, Option<T>) {
        self.als_string_beschreibung_allgemein(ToString::to_string)
    }
}

impl<T> Beschreibung<T> {
    pub(crate) fn als_string_beschreibung_allgemein(
        self,
        anzeige: impl Fn(&T) -> String,
    ) -> (Beschreibung<String>, Option<T>) {
        let Beschreibung { lang, kurz, hilfe, standard } = self;
        let standard_string = standard.as_ref().map(anzeige);
        (Beschreibung { lang, kurz, hilfe, standard: standard_string }, standard)
    }

    /// Konvertiere eine [Beschreibung] zu einem anderen Typ.
    ///
    /// ## English synonym
    /// [convert](Description::convert)
    pub fn konvertiere<S>(self, konvertiere: impl FnOnce(T) -> S) -> Beschreibung<S> {
        let Beschreibung { lang, kurz, hilfe, standard } = self;
        Beschreibung { lang, kurz, hilfe, standard: standard.map(konvertiere) }
    }

    /// Convert a [Description] to a different type.
    ///
    /// ## Deutsches Synonym
    /// [konvertiere](Beschreibung::konvertiere)
    #[inline(always)]
    pub fn convert<S>(self, convert: impl FnOnce(T) -> S) -> Description<S> {
        self.konvertiere(convert)
    }
}

macro_rules! contains_str {
    ($collection: expr, $gesucht: expr) => {
        $collection.iter().any(|element| element == $gesucht)
    };
}
pub(crate) use contains_str;

/// Mindestens ein String als Definition für den vollen Namen.
///
/// ## English synonym
/// [LongNames]
pub trait LangNamen: Sized {
    /// Konvertiere in ein [NonEmpty].
    ///
    /// ## English synonym
    /// [long_names](LongNames::long_names)
    fn lang_namen(self) -> NonEmpty<String>;

    /// Convert into a [NonEmpty].
    ///
    /// ## Deutsches Synonym
    /// [lang_namen](LangNamen::lang_namen)
    #[inline(always)]
    fn long_names(self) -> NonEmpty<String> {
        self.lang_namen()
    }
}

#[cfg(all(doc, not(doctest)))]
/// At least one String as definition for the full name.
///
/// ## Deutsches Synonym
/// [LangNamen]
pub trait LongNames = LangNamen;

impl LangNamen for String {
    fn lang_namen(self) -> NonEmpty<String> {
        NonEmpty::singleton(self)
    }
}

impl LangNamen for &str {
    fn lang_namen(self) -> NonEmpty<String> {
        NonEmpty::singleton(self.to_owned())
    }
}

impl LangNamen for NonEmpty<String> {
    fn lang_namen(self) -> NonEmpty<String> {
        self
    }
}

impl<S: Deref<Target = str>> LangNamen for &NonEmpty<S> {
    fn lang_namen(self) -> NonEmpty<String> {
        let NonEmpty { head, tail } = self;
        NonEmpty {
            head: head.deref().to_owned(),
            tail: tail.iter().map(|s| s.deref().to_owned()).collect(),
        }
    }
}

/// Beliebige Anzahl an Strings für den kurzen Namen.
///
/// ## English synonym
/// [ShortNames]
pub trait KurzNamen: Sized {
    /// Konvertiere in einen [Vec].
    ///
    /// ## English synonym
    /// [short_names](ShortNames::short_names)
    fn kurz_namen(self) -> Vec<String>;

    /// Convert into a [Vec].
    ///
    /// ## Deutsches Synonym
    /// [kurz_namen](LangNamen::kurz_namen)
    #[inline(always)]
    fn short_names(self) -> Vec<String> {
        self.kurz_namen()
    }
}

#[cfg(all(doc, not(doctest)))]
/// Arbitrary amount of strings for the short name.
///
/// ## Deutsches Synonym
/// [LangNamen]
pub trait ShortNames = KurzNamen;

impl KurzNamen for Option<String> {
    fn kurz_namen(self) -> Vec<String> {
        self.into_iter().collect()
    }
}

impl KurzNamen for String {
    fn kurz_namen(self) -> Vec<String> {
        vec![self]
    }
}

impl KurzNamen for &str {
    fn kurz_namen(self) -> Vec<String> {
        vec![self.to_owned()]
    }
}

impl KurzNamen for NonEmpty<String> {
    fn kurz_namen(self) -> Vec<String> {
        self.into()
    }
}

impl<S: Deref<Target = str>> KurzNamen for &Vec<S> {
    fn kurz_namen(self) -> Vec<String> {
        self.iter().map(|s| s.deref().to_owned()).collect()
    }
}

impl KurzNamen for Vec<String> {
    fn kurz_namen(self) -> Vec<String> {
        self
    }
}

impl<T> Beschreibung<T> {
    /// Erzeuge eine neue [Beschreibung].
    ///
    /// ## English synonym
    /// [new](Description::new)
    pub fn neu(
        lang: impl LangNamen,
        kurz: impl KurzNamen,
        hilfe: Option<String>,
        standard: Option<T>,
    ) -> Beschreibung<T> {
        Beschreibung { lang: lang.lang_namen(), kurz: kurz.kurz_namen(), hilfe, standard }
    }

    /// Create a new [Description].
    ///
    /// ## Deutsches Synonym
    /// [neu](Beschreibung::neu)
    #[inline(always)]
    pub fn new(
        long: impl LangNamen,
        short: impl KurzNamen,
        help: Option<String>,
        default: Option<T>,
    ) -> Description<T> {
        Beschreibung::neu(long, short, help, default)
    }
}

/// Konfiguration eines Kommandozeilen-Arguments.
///
/// ## English synonym
/// [Configuration]
#[derive(Debug)]
pub enum Konfiguration {
    /// Es handelt sich um ein Flag-Argument.
    ///
    /// ## English
    /// It is a flag argument.
    Flag {
        /// Allgemeine Beschreibung des Arguments.
        ///
        /// ## English
        /// General description of the argument.
        beschreibung: Beschreibung<String>,
        /// Präfix zum invertieren des Flag-Arguments.
        ///
        /// ## English
        /// Prefix to invert the flag argument.
        invertiere_präfix: Option<String>,
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
        beschreibung: Beschreibung<String>,
        /// Meta-Variable im Hilfe-Text.
        ///
        /// ## English
        /// Meta-variable used in the help-text.
        meta_var: String,
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
pub type Configuration = Konfiguration;
