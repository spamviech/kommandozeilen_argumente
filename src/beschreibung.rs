//! Beschreibung eines Arguments.

use std::fmt::Display;
use std::ops::Deref;

use nonempty::NonEmpty;

// TODO english doc for fields
/// Beschreibung eines Kommandozeilen-Arguments.
#[derive(Debug, Clone)]
pub struct Beschreibung<T> {
    /// Voller Name, wird nach zwei Minus angegeben "--<lang>".
    pub lang: NonEmpty<String>,
    /// Kurzer Name, wird nach einem Minus angegeben "-<kurz>".
    /// Kurznamen länger als ein [Grapheme](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// werden nicht unterstützt.
    pub kurz: Vec<String>,
    /// Im automatischen Hilfetext angezeigte Beschreibung.
    pub hilfe: Option<String>,
    /// Standard-Wert falls kein passendes Kommandozeilen-Argument verwendet wurde.
    pub standard: Option<T>,
}

/// Description of a command line argument.
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
    pub fn konvertiere<S>(self, konvertiere: impl FnOnce(T) -> S) -> Beschreibung<S> {
        let Beschreibung { lang, kurz, hilfe, standard } = self;
        Beschreibung { lang, kurz, hilfe, standard: standard.map(konvertiere) }
    }

    /// Convert a [Description] to a different type.
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

// TODO english doc, english alias?
/// Mindestens ein String als Definition für den vollen Namen.
pub trait LangNamen {
    /// Konvertiere in ein [NonEmpty].
    fn lang_namen(self) -> NonEmpty<String>;
}

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

// TODO english doc, english alias?
/// Beliebige Anzahl an Strings für den kurzen Namen.
pub trait KurzNamen {
    /// Konvertiere in einen [Vec].
    fn kurz_namen(self) -> Vec<String>;
}

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
    pub fn neu(
        lang: impl LangNamen,
        kurz: impl KurzNamen,
        hilfe: Option<String>,
        standard: Option<T>,
    ) -> Beschreibung<T> {
        Beschreibung { lang: lang.lang_namen(), kurz: kurz.kurz_namen(), hilfe, standard }
    }

    /// Create a new [Description].
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

// TODO english doc for variants, fields
/// Konfiguration eines Kommandozeilen-Arguments.
#[derive(Debug)]
pub enum Konfiguration {
    /// Es handelt sich um ein Flag-Argument.
    Flag {
        /// Allgemeine Beschreibung des Arguments.
        beschreibung: Beschreibung<String>,
        /// Präfix zum invertieren des Flag-Arguments.
        invertiere_präfix: Option<String>,
    },
    /// Es handelt sich um ein Wert-Argument.
    Wert {
        /// Allgemeine Beschreibung des Arguments.
        beschreibung: Beschreibung<String>,
        /// Meta-Variable im Hilfe-Text.
        meta_var: String,
        /// String-Darstellung der erlaubten Werte.
        mögliche_werte: Option<NonEmpty<String>>,
    },
}

/// Configuration of a command line argument.
pub type Configuration = Konfiguration;
