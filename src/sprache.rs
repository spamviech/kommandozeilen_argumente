//! Alle Strings, die zum erstellen von Hilfe-Text und Fehlermeldung notwendig sind.

/// Alle Strings, die zum erstellen von Hilfe-Text und Fehlermeldung notwendig sind.
#[derive(Debug, Clone, Copy)]
pub struct Sprache {
    /// Standard-Präfix zum invertieren einer Flag.
    pub invertiere_präfix: &'static str,
    /// Standard-Wert für die Meta-Variable im Hilfe-Text
    pub meta_var: &'static str,
    /// Meta-Beschreibung für Optionen im Hilfe-Text.
    pub optionen: &'static str,
    /// Beschreibung für Standard-Wert im Hilfe-Text.
    pub standard: &'static str,
    /// Beschreibung für mögliche Werte im Hilfe-Text.
    pub erlaubte_werte: &'static str,
    /// Beschreibung einer fehlenden Flag in einer Fehlermeldung.
    pub fehlende_flag: &'static str,
    /// Beschreibung eines fehlenden Wertes in einer Fehlermeldung.
    pub fehlender_wert: &'static str,
    /// Beschreibung eines Parse-Fehlers in einer Fehlermeldung.
    pub parse_fehler: &'static str,
    /// Beschreibung eines invaliden Strings in einer Fehlermeldung.
    pub invalider_string: &'static str,
    /// Beschreibung für ein nicht verwendetes Argument in einer Fehlermeldung.
    pub argument_nicht_verwendet: &'static str,
}

impl Sprache {
    /// Deutsche Strings.
    pub const DEUTSCH: Sprache = Sprache {
        invertiere_präfix: "kein",
        meta_var: "WERT",
        optionen: "OPTIONEN",
        standard: "Standard",
        erlaubte_werte: "Erlaubte Werte",
        fehlende_flag: "Fehlende Flag",
        fehlender_wert: "Fehlender Wert",
        parse_fehler: "Parse-Fehler",
        invalider_string: "Invalider String",
        argument_nicht_verwendet: "Nicht alle Argumente verwendet",
    };

    /// English Strings.
    pub const ENGLISH: Sprache = Sprache {
        invertiere_präfix: "no",
        meta_var: "VALUE",
        optionen: "OPTIONS",
        standard: "Default",
        erlaubte_werte: "Possible values",
        fehlende_flag: "Missing Flag",
        fehlender_wert: "Missing Value",
        parse_fehler: "Parse Error",
        invalider_string: "Invalid String",
        argument_nicht_verwendet: "Unused argument(s)",
    };
}
