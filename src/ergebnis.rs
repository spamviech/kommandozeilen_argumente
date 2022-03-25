//! Ergebnis- und Fehler-Typ für parsen von Kommandozeilen-Argumenten.

use std::{borrow::Cow, ffi::OsString, fmt::Display, iter};

use either::Either;
use nonempty::NonEmpty;

use crate::{
    sprache::{Language, Sprache},
    unicode::Normalisiert,
};

/// Ergebnis des Parsen von Kommandozeilen-Argumenten.
///
/// ## English synonym
/// [Result]
#[derive(Debug)]
pub enum Ergebnis<'t, T, E> {
    /// Erfolgreiches Parsen.
    ///
    /// ## English
    /// Successful parsing.
    Wert(T),
    /// Frühes Beenden durch zeigen der Nachrichten gewünscht.
    ///
    /// ## English
    /// Request an early exit, showing the given messages.
    FrühesBeenden(NonEmpty<Cow<'t, str>>),
    /// Fehler beim Parsen der Kommandozeilen-Argumente.
    ///
    /// ## English
    /// Error while parsing command line arguments.
    Fehler(NonEmpty<Fehler<'t, E>>),
}

/// Result when parsing command line arguments.
///
/// ## Deutsches Synonym
/// [Ergebnis]
pub type Result<'t, T, E> = Ergebnis<'t, T, E>;

impl<'t, T, E> Ergebnis<'t, T, E> {
    /// Konvertiere einen erfolgreich geparsten Wert mit der spezifizierten Funktion.
    ///
    /// ## English synonym
    /// [convert](Result::convert)
    pub fn konvertiere<S>(self, f: impl FnOnce(T) -> S) -> Ergebnis<'t, S, E> {
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
    pub fn convert<S>(self, f: impl FnOnce(T) -> S) -> Ergebnis<'t, S, E> {
        self.konvertiere(f)
    }
}

/// Alle Namen eines Arguments.
///
/// ## English synonym
/// [Names]
#[derive(Debug, Clone)]
pub struct Namen<'t> {
    /// Präfix vor dem LangNamen.
    ///
    /// ## English
    /// Prefix before the long name.
    pub lang_präfix: Normalisiert<'t>,

    /// Vollständiger Name.
    ///
    /// ## English
    /// Full name.
    pub lang: NonEmpty<Normalisiert<'t>>,

    /// Präfix vor dem KurzNamen.
    ///
    /// ## English
    /// Prefix before the short name.
    pub kurz_präfix: Normalisiert<'t>,

    /// Kurzform des Namen.
    ///
    /// ## English
    /// Short form of the name.
    pub kurz: Vec<Normalisiert<'t>>,
}

/// All names of an argument.
///
/// ## Deutsches Synonym
/// [Namen]
pub type Names<'t> = Namen<'t>;

/// Fehlerquellen beim Parsen von Kommandozeilen-Argumenten.
///
/// ## English synonym
/// [Result]
#[derive(Debug, Clone)]
pub enum Fehler<'t, E> {
    /// Ein benötigtes Flag-Argument wurde nicht genannt.
    ///
    /// ## English
    /// A required flag argument is missing.
    FehlendeFlag {
        /// Alle Namen des Flag-Arguments.
        ///
        /// ## English
        /// All names of the flag argument.
        namen: Namen<'t>,

        /// Präfix zum invertieren des Flag-Arguments.
        ///
        /// ## English
        /// Prefix to invert the flag argument.
        invertiere_präfix: Normalisiert<'t>,

        /// Auf Präfix folgendes Infix zum invertieren des Flag-Arguments.
        ///
        /// ## English
        /// Infix following the prefix to invert the flag argument.
        invertiere_infix: Normalisiert<'t>,
    },
    /// Ein benötigtes Wert-Argument wurde nicht genannt.
    ///
    /// ## English
    /// A required value argument is missing.
    FehlenderWert {
        /// Alle Namen des Wert-Arguments.
        ///
        /// ## English
        /// All names of the value argument.
        namen: Namen<'t>,

        /// Infix um einen Wert im selben Argument wie den Namen anzugeben.
        ///
        /// ## English
        /// Infix to give a value in the same argument as the name.
        wert_infix: Normalisiert<'t>,

        /// Verwendete Meta-Variable für den Wert.
        ///
        /// ## English
        /// Used Meta-variable of the value.
        meta_var: &'t str,
    },
    /// Fehler beim Parsen des genannten Wertes.
    ///
    /// ## English
    /// Error while parsing the value.
    Fehler {
        /// Alle Namen des Wert-Arguments.
        ///
        /// ## English
        /// All names of the value argument.
        namen: Namen<'t>,

        /// Infix um einen Wert im selben Argument wie den Namen anzugeben.
        ///
        /// ## English
        /// Infix to give a value in the same argument as the name.
        wert_infix: Normalisiert<'t>,

        /// Verwendete Meta-Variable für den Wert.
        ///
        /// ## English
        /// Used Meta-variable of the value.
        meta_var: &'t str,

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
pub type Error<'t, E> = Fehler<'t, E>;

pub(crate) fn namen_regex_hinzufügen<S: AsRef<str>>(string: &mut String, head: &S, tail: &[S]) {
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
        string.push_str(name.as_ref());
    }
    if !tail.is_empty() {
        string.push(')')
    }
}

/// Mögliche Fehler-Quellen beim Parsen aus einem [OsStr].
///
/// ## English synonym
/// [ParseError]
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(single_use_lifetimes)]
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

/// Possible errors when parsing an [OsStr].
///
/// ## Deutsches Synonym
/// [ParseFehler]
pub type ParseError<E> = ParseFehler<E>;

impl<E: Display> Fehler<'_, E> {
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
        self.create_error_message_with_language(Language::ENGLISH)
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
        self.erstelle_fehlermeldung_mit_sprache(language.sprache())
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
            Namen { lang_präfix, lang, kurz_präfix, kurz }: &Namen<'_>,
            flag_oder_wert: Either<
                (&Normalisiert<'_>, &Normalisiert<'_>),
                (&Normalisiert<'_>, &str),
            >,
        ) -> String {
            let mut fehlermeldung = format!("{fehler_beschreibung}: ");
            fehlermeldung.push_str(lang_präfix.as_ref());
            match flag_oder_wert {
                Either::Left((invertiere_präfix, invertiere_infix)) => {
                    fehlermeldung.push('[');
                    fehlermeldung.push_str(invertiere_präfix.as_ref());
                    fehlermeldung.push_str(invertiere_infix.as_ref());
                    fehlermeldung.push(']');
                    namen_regex_hinzufügen(&mut fehlermeldung, &lang.head, &lang.tail);
                },
                Either::Right((wert_infix, meta_var)) => {
                    namen_regex_hinzufügen(&mut fehlermeldung, &lang.head, &lang.tail);
                    fehlermeldung.push_str("( |");
                    fehlermeldung.push_str(wert_infix.as_ref());
                    fehlermeldung.push(')');
                    fehlermeldung.push_str(meta_var);
                },
            }
            if let Some((head, tail)) = kurz.split_first() {
                fehlermeldung.push_str(" | ");
                fehlermeldung.push_str(kurz_präfix.as_ref());
                namen_regex_hinzufügen(&mut fehlermeldung, head, tail);
                if let Either::Right((wert_infix, meta_var)) = flag_oder_wert {
                    fehlermeldung.push_str("[ |");
                    fehlermeldung.push_str(wert_infix.as_ref());
                    fehlermeldung.push(']');
                    fehlermeldung.push_str(meta_var);
                }
            }
            fehlermeldung
        }
        match self {
            Fehler::FehlendeFlag { namen, invertiere_präfix, invertiere_infix } => fehlermeldung(
                fehlende_flag,
                namen,
                Either::Left((invertiere_präfix, invertiere_infix)),
            ),
            Fehler::FehlenderWert { namen, wert_infix, meta_var } => {
                fehlermeldung(fehlender_wert, namen, Either::Right((wert_infix, meta_var)))
            },
            Fehler::Fehler { namen, wert_infix, meta_var, fehler } => {
                let (fehler_art, fehler_anzeige) = match fehler {
                    ParseFehler::InvaliderString(os_string) => {
                        (invalider_string, format!("{:?}", os_string))
                    },
                    ParseFehler::ParseFehler(fehler) => (parse_fehler, fehler.to_string()),
                };
                let mut fehlermeldung =
                    fehlermeldung(fehler_art, namen, Either::Right((wert_infix, meta_var)));
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
