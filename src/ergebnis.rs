//! Ergebnis- und Fehler-Typ für parsen von Kommandozeilen-Argumenten.

use std::{ffi::OsString, fmt::Display};

use nonempty::NonEmpty;

/// Ergebnis des Parsen von Kommandozeilen-Argumenten.
#[derive(Debug)]
pub enum ParseErgebnis<T, E> {
    /// Erfolgreiches Parsen.
    Wert(T),
    /// Frühes Beenden durch zeigen der Nachrichten gewünscht.
    FrühesBeenden(NonEmpty<String>),
    /// Fehler beim Parsen der Kommandozeilen-Argumente.
    Fehler(NonEmpty<ParseFehler<E>>),
}

/// Fehlerquellen beim Parsen von Kommandozeilen-Argumenten
#[derive(Debug, Clone)]
pub enum ParseFehler<E> {
    /// Ein benötigtes Flag-Argument wurde nicht genannt.
    FehlendeFlag {
        /// Vollständiger Name.
        lang: String,
        /// Kurzform des Namen.
        kurz: Option<String>,
        /// Präfix zum invertieren.
        invertiere_prefix: String,
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
    ParseFehler {
        /// Vollständiger Name.
        lang: String,
        /// Kurzform des Namen.
        kurz: Option<String>,
        /// Verwendete Meta-Variable für den Wert.
        meta_var: String,
        /// Beim Parsen aufgetretener Fehler.
        fehler: E,
    },
}

impl ParseFehler<OsString> {
    /// Versuche [ParseFehler::ParseFehler] von [OsString] nach [String] zu konvertieren.
    /// Dadurch kann der [ParseFehler] über [fehlermeldung] und [error_message] angezeigt werden.
    pub fn als_string(self) -> Result<ParseFehler<String>, ParseFehler<OsString>> {
        match self {
            ParseFehler::FehlendeFlag { lang, kurz, invertiere_prefix } => {
                Ok(ParseFehler::FehlendeFlag { lang, kurz, invertiere_prefix })
            }
            ParseFehler::FehlenderWert { lang, kurz, meta_var } => {
                Ok(ParseFehler::FehlenderWert { lang, kurz, meta_var })
            }
            ParseFehler::ParseFehler { lang, kurz, meta_var, fehler } => {
                match fehler.into_string() {
                    Ok(fehler) => Ok(ParseFehler::ParseFehler { lang, kurz, meta_var, fehler }),
                    Err(fehler) => Err(ParseFehler::ParseFehler { lang, kurz, meta_var, fehler }),
                }
            }
        }
    }
}

impl<E: Display> ParseFehler<E> {
    /// Zeige den Fehler in Menschen-lesbarer Form an.
    #[inline(always)]
    pub fn fehlermeldung(&self) -> String {
        self.erstelle_fehlermeldung("Fehlende Flag", "Fehlender Wert", "Parse-Fehler")
    }

    /// Show the error in a human readable form.
    #[inline(always)]
    pub fn error_message(&self) -> String {
        self.erstelle_fehlermeldung("Missing Flag", "Missing Value", "Parse Error")
    }

    /// Zeige den Fehler in Menschen-lesbarer Form an.
    pub fn erstelle_fehlermeldung(
        &self,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
    ) -> String {
        match self {
            ParseFehler::FehlendeFlag { lang, kurz, invertiere_prefix } => {
                let mut fehlermeldung =
                    format!("{}: --[{}-]{}", fehlende_flag, invertiere_prefix, lang);
                if let Some(kurz) = kurz {
                    fehlermeldung.push_str(" | -");
                    fehlermeldung.push_str(kurz);
                }
                fehlermeldung
            }
            ParseFehler::FehlenderWert { lang, kurz, meta_var } => {
                let mut fehlermeldung = format!("{}: --{} {}", fehlender_wert, lang, meta_var);
                if let Some(kurz) = kurz {
                    fehlermeldung.push_str(" | -");
                    fehlermeldung.push_str(kurz);
                    fehlermeldung.push_str("[=| ]");
                    fehlermeldung.push_str(meta_var);
                }
                fehlermeldung
            }
            ParseFehler::ParseFehler { lang, kurz, meta_var, fehler } => {
                let mut fehlermeldung = format!("{}: --{} {}", parse_fehler, lang, meta_var);
                if let Some(kurz) = kurz {
                    fehlermeldung.push_str(" | -");
                    fehlermeldung.push_str(kurz);
                    fehlermeldung.push_str("[=| ]");
                    fehlermeldung.push_str(meta_var);
                }
                fehlermeldung.push('\n');
                fehlermeldung.push_str(&fehler.to_string());
                fehlermeldung
            }
        }
    }
}
