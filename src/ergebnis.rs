//! Ergebnis- und Fehler-Typ für parsen von Kommandozeilen-Argumenten.

use std::fmt::Display;

use nonempty::NonEmpty;

#[derive(Debug)]
pub enum ParseErgebnis<T, E> {
    Wert(T),
    FrühesBeenden(NonEmpty<String>),
    Fehler(NonEmpty<ParseFehler<E>>),
}

#[derive(Debug, Clone)]
pub enum ParseFehler<E> {
    FehlendeFlag { lang: String, kurz: Option<String>, invertiere_prefix: String },
    FehlenderWert { lang: String, kurz: Option<String>, meta_var: String },
    ParseFehler { lang: String, kurz: Option<String>, meta_var: String, fehler: E },
}

impl<E: Display> ParseFehler<E> {
    #[inline(always)]
    pub fn fehlermeldung(&self) -> String {
        self.erstelle_fehlermeldung("Fehlende Flag", "Fehlender Wert", "Parse-Fehler")
    }

    #[inline(always)]
    pub fn error_message(&self) -> String {
        self.erstelle_fehlermeldung("Missing Flag", "Missing Value", "Parse Error")
    }

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
