//! Ergebnis- und Fehler-Typ für parsen von Kommandozeilen-Argumenten.

use std::{ffi::OsString, fmt::Display, iter};

use either::Either;
use nonempty::NonEmpty;

use crate::sprache::{Language, Sprache};

/// Ergebnis des Parsen von Kommandozeilen-Argumenten.
///
/// ## English synonym
/// [Result]
#[derive(Debug)]
pub enum Ergebnis<T, E> {
    /// Erfolgreiches Parsen.
    ///
    /// ## English
    /// Successful parsing.
    Wert(T),
    /// Frühes Beenden durch zeigen der Nachrichten gewünscht.
    ///
    /// ## English
    /// Request an early exit, showing the given messages.
    FrühesBeenden(NonEmpty<String>),
    /// Fehler beim Parsen der Kommandozeilen-Argumente.
    ///
    /// ## English
    /// Error while parsing command line arguments.
    Fehler(NonEmpty<Fehler<E>>),
}

/// Result when parsing command line arguments.
///
/// ## Deutsches Synonym
/// [Ergebnis]
pub type Result<T, E> = Ergebnis<T, E>;

impl<T, E> Ergebnis<T, E> {
    /// Konvertiere einen erfolgreich geparsten Wert mit der spezifizierten Funktion.
    ///
    /// ## English synonym
    /// [convert](Result::convert)
    pub fn konvertiere<S>(self, f: impl FnOnce(T) -> S) -> Ergebnis<S, E> {
        match self {
            Ergebnis::Wert(t) => Ergebnis::Wert(f(t)),
            Ergebnis::FrühesBeenden(nachrichten) => Ergebnis::FrühesBeenden(nachrichten),
            Ergebnis::Fehler(fehler) => Ergebnis::Fehler(fehler),
        }
    }

    /// Convert a successfully parsed value using the specified function.
    ///
    /// ## Deutsches Synonym
    /// [konvertiere](Ergebnis::konvertiere)
    #[inline(always)]
    pub fn convert<S>(self, f: impl FnOnce(T) -> S) -> Ergebnis<S, E> {
        self.konvertiere(f)
    }
}

/// Fehlerquellen beim Parsen von Kommandozeilen-Argumenten.
///
/// ## English synonym
/// [Result]
#[derive(Debug, Clone)]
pub enum Fehler<E> {
    /// Ein benötigtes Flag-Argument wurde nicht genannt.
    ///
    /// ## English
    /// A required flag argument is missing.
    FehlendeFlag {
        /// Vollständiger Name.
        ///
        /// ## English
        /// Full name.
        lang: NonEmpty<String>,
        /// Kurzform des Namen.
        ///
        /// ## English
        /// Short form of the name.
        kurz: Vec<String>,
        /// Präfix zum invertieren.
        ///
        /// ## English
        /// Prefix to invert the flag.
        invertiere_präfix: String,
    },
    /// Ein benötigtes Wert-Argument wurde nicht genannt.
    ///
    /// ## English
    /// A required value argument is missing.
    FehlenderWert {
        /// Vollständiger Name.
        ///
        /// ## English
        /// Full name.
        lang: NonEmpty<String>,
        /// Kurzform des Namen.
        ///
        /// ## English
        /// Short form of the name.
        kurz: Vec<String>,
        /// Verwendete Meta-Variable für den Wert.
        ///
        /// ## English
        /// Used Meta-variable of the value.
        meta_var: String,
    },
    /// Fehler beim Parsen des genannten Wertes.
    ///
    /// ## English
    /// Error while parsing the value.
    Fehler {
        /// Vollständiger Name.
        ///
        /// ## English
        /// Full name
        lang: NonEmpty<String>,
        /// Kurzform des Namen.
        ///
        /// ## English
        /// Short form of the name.
        kurz: Vec<String>,
        /// Verwendete Meta-Variable für den Wert.
        ///
        /// ## English
        /// Used Meta-variable of the value.
        meta_var: String,
        /// Beim Parsen aufgetretener Fehler.
        ///
        /// ## English
        /// Reported error from parsing.
        fehler: ParseFehler<E>,
    },
}

/// Possible errors when parsing command line arguments.
///
/// ## Deutsches Synonym
/// [Fehler]
pub type Error<E> = Fehler<E>;

pub(crate) fn namen_regex_hinzufügen(string: &mut String, head: &String, tail: &[String]) {
    if !tail.is_empty() {
        string.push('(')
    }
    let mut first = true;
    for name in iter::once(head).chain(tail) {
        if first {
            first = false;
        } else {
            string.push_str("|");
        }
        string.push_str(name);
    }
    if !tail.is_empty() {
        string.push(')')
    }
}

/// Mögliche Fehler-Quellen beim Parsen aus einem [OsStr](std::ffi::OsStr).
///
/// ## English synonym
/// [ParseError]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFehler<E> {
    /// Die Konvertierung in ein [&str](str) ist fehlgeschlagen.
    ///
    /// ## English
    /// Conversion to a [&str](str) failed.
    InvaliderString(OsString),
    /// Fehler beim Parsen des Strings.
    ///
    /// ## English
    /// Error while parsing the string.
    ParseFehler(E),
}

/// Possible errors when parsing an [OsStr](std::ffi::OsStr).
///
/// ## Deutsches Synonym
/// [ParseFehler]
pub type ParseError<E> = ParseFehler<E>;

impl<E: Display> Fehler<E> {
    /// Zeige den Fehler in Menschen-lesbarer Form an.
    ///
    /// ## English version
    /// [error_message](Error::error_message)
    #[inline(always)]
    pub fn fehlermeldung(&self) -> String {
        self.erstelle_fehlermeldung_mit_sprache(Sprache::DEUTSCH)
    }

    /// Show the error in a human readable form.
    ///
    /// ## Deutsches Version
    /// [fehlermeldung](Fehler::fehlermeldung)
    #[inline(always)]
    pub fn error_message(&self) -> String {
        self.erstelle_fehlermeldung_mit_sprache(Language::ENGLISH)
    }

    /// Zeige den [Fehler] in Menschen-lesbarer Form an.
    ///
    /// ## English synonym
    /// [create_error_message_with_language](Error::create_error_message_with_language)
    #[inline(always)]
    pub fn erstelle_fehlermeldung_mit_sprache(&self, sprache: Sprache) -> String {
        self.erstelle_fehlermeldung(
            sprache.fehlende_flag,
            sprache.fehlender_wert,
            sprache.parse_fehler,
            sprache.invalider_string,
        )
    }

    /// Show the [Error] in human readable form.
    ///
    /// ## Deutsches Synonym
    /// [erstelle_fehlermeldung_mit_sprache](Fehler::erstelle_fehlermeldung_mit_sprache)
    #[inline(always)]
    pub fn create_error_message_with_language(&self, language: Language) -> String {
        self.erstelle_fehlermeldung_mit_sprache(language)
    }

    /// Zeige den Fehler in Menschen-lesbarer Form an.
    ///
    /// ## English synonym
    /// [create_error_message](Error::create_error_message)
    pub fn erstelle_fehlermeldung(
        &self,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
    ) -> String {
        fn fehlermeldung(
            fehler_beschreibung: &str,
            lang: &NonEmpty<String>,
            kurz: &Vec<String>,
            meta_var_oder_invertiere_präfix: Either<&String, &String>,
        ) -> String {
            let mut fehlermeldung = format!("{}: ", fehler_beschreibung);
            fehlermeldung.push_str("--");
            match meta_var_oder_invertiere_präfix {
                Either::Left(invertiere_präfix) => {
                    fehlermeldung.push('[');
                    fehlermeldung.push_str(invertiere_präfix);
                    fehlermeldung.push_str("-]");
                    namen_regex_hinzufügen(&mut fehlermeldung, &lang.head, &lang.tail);
                },
                Either::Right(meta_var) => {
                    namen_regex_hinzufügen(&mut fehlermeldung, &lang.head, &lang.tail);
                    fehlermeldung.push_str("( |=)");
                    fehlermeldung.push_str(meta_var);
                },
            }
            if let Some((head, tail)) = kurz.split_first() {
                fehlermeldung.push_str(" | -");
                namen_regex_hinzufügen(&mut fehlermeldung, head, tail);
                if let Either::Right(meta_var) = meta_var_oder_invertiere_präfix {
                    fehlermeldung.push_str("[ |=]");
                    fehlermeldung.push_str(meta_var);
                }
            }
            fehlermeldung
        }
        match self {
            Fehler::FehlendeFlag { lang, kurz, invertiere_präfix } => {
                fehlermeldung(fehlende_flag, lang, kurz, Either::Left(invertiere_präfix))
            },
            Fehler::FehlenderWert { lang, kurz, meta_var } => {
                fehlermeldung(fehlender_wert, lang, kurz, Either::Right(meta_var))
            },
            Fehler::Fehler { lang, kurz, meta_var, fehler } => {
                let (fehler_art, fehler_anzeige) = match fehler {
                    ParseFehler::InvaliderString(os_string) => {
                        (invalider_string, format!("{:?}", os_string))
                    },
                    ParseFehler::ParseFehler(fehler) => (parse_fehler, fehler.to_string()),
                };
                let mut fehlermeldung =
                    fehlermeldung(fehler_art, lang, kurz, Either::Right(meta_var));
                fehlermeldung.push('\n');
                fehlermeldung.push_str(&fehler_anzeige);
                fehlermeldung
            },
        }
    }

    /// Show the [Error] in human readable form.
    ///
    /// ## Deutsches Synonym
    /// [erstelle_fehlermeldung](Fehler::erstelle_fehlermeldung)
    #[inline(always)]
    pub fn create_error_message(
        &self,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
    ) -> String {
        self.erstelle_fehlermeldung(missing_flag, missing_value, parse_error, invalid_string)
    }
}
