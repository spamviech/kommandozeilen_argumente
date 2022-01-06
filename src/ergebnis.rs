//! Ergebnis- und Fehler-Typ für parsen von Kommandozeilen-Argumenten.

use std::{ffi::OsString, fmt::Display};

use nonempty::NonEmpty;

/// Ergebnis des Parsen von Kommandozeilen-Argumenten.
#[derive(Debug)]
pub enum Ergebnis<T, E> {
    /// Erfolgreiches Parsen.
    Wert(T),
    /// Frühes Beenden durch zeigen der Nachrichten gewünscht.
    FrühesBeenden(NonEmpty<String>),
    /// Fehler beim Parsen der Kommandozeilen-Argumente.
    Fehler(NonEmpty<Fehler<E>>),
}

/// Fehlerquellen beim Parsen von Kommandozeilen-Argumenten
#[derive(Debug, Clone)]
pub enum Fehler<E> {
    /// Ein benötigtes Flag-Argument wurde nicht genannt.
    FehlendeFlag {
        /// Vollständiger Name.
        lang: String,
        /// Kurzform des Namen.
        kurz: Option<String>,
        /// Präfix zum invertieren.
        invertiere_präfix: String,
    },
    /// Ein benötigtes Wert-Argument wurde nicht genannt.
    FehlenderWert {
        /// Vollständiger Name.
        lang: String,
        /// Kurzform des Namen.
        kurz: Option<String>,
        /// Verwendete Meta-Variable für den Wert.
        meta_var: String,
    },
    /// Fehler beim Parsen des genannten Wertes.
    Fehler {
        /// Vollständiger Name.
        lang: String,
        /// Kurzform des Namen.
        kurz: Option<String>,
        /// Verwendete Meta-Variable für den Wert.
        meta_var: String,
        /// Beim Parsen aufgetretener Fehler.
        fehler: ParseFehler<E>,
    },
}

/// Mögliche Fehler-Quellen beim Parsen aus einem [OsStr].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFehler<E> {
    /// Die Konvertierung in ein &[str] ist fehlgeschlagen.
    InvaliderString(OsString),
    /// Fehler beim Parsen des Strings.
    ParseFehler(E),
}

impl<E: Display> Fehler<E> {
    /// Zeige den Fehler in Menschen-lesbarer Form an.
    #[inline(always)]
    pub fn fehlermeldung(&self) -> String {
        self.erstelle_fehlermeldung(
            "Fehlende Flag",
            "Fehlender Wert",
            "Parse-Fehler",
            "Invalider String",
        )
    }

    /// Show the error in a human readable form.
    #[inline(always)]
    pub fn error_message(&self) -> String {
        self.erstelle_fehlermeldung(
            "Missing Flag",
            "Missing Value",
            "Parse Error",
            "Invalid String",
        )
    }

    /// Zeige den Fehler in Menschen-lesbarer Form an.
    pub fn erstelle_fehlermeldung(
        &self,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
    ) -> String {
        match self {
            Fehler::FehlendeFlag { lang, kurz, invertiere_präfix } => {
                let mut fehlermeldung =
                    format!("{}: --[{}-]{}", fehlende_flag, invertiere_präfix, lang);
                if let Some(kurz) = kurz {
                    fehlermeldung.push_str(" | -");
                    fehlermeldung.push_str(kurz);
                }
                fehlermeldung
            }
            Fehler::FehlenderWert { lang, kurz, meta_var } => {
                let mut fehlermeldung = format!("{}: --{} {}", fehlender_wert, lang, meta_var);
                if let Some(kurz) = kurz {
                    fehlermeldung.push_str(" | -");
                    fehlermeldung.push_str(kurz);
                    fehlermeldung.push_str("[=| ]");
                    fehlermeldung.push_str(meta_var);
                }
                fehlermeldung
            }
            Fehler::Fehler { lang, kurz, meta_var, fehler } => {
                let (fehler_art, fehler_anzeige) = match fehler {
                    ParseFehler::InvaliderString(os_string) => {
                        (invalider_string, format!("{:?}", os_string))
                    }
                    ParseFehler::ParseFehler(fehler) => (parse_fehler, fehler.to_string()),
                };
                let mut fehlermeldung = format!("{}: --{} {}", fehler_art, lang, meta_var);
                if let Some(kurz) = kurz {
                    fehlermeldung.push_str(" | -");
                    fehlermeldung.push_str(kurz);
                    fehlermeldung.push_str("[=| ]");
                    fehlermeldung.push_str(meta_var);
                }
                fehlermeldung.push('\n');
                fehlermeldung.push_str(&fehler_anzeige);
                fehlermeldung
            }
        }
    }
}
