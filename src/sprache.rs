//! Alle Strings, die zum erstellen von Hilfe-Text und Fehlermeldung notwendig sind.

/// Alle Strings, die zum erstellen von Hilfe-Text und Fehlermeldung notwendig sind.
///
/// ## English synonym
/// [Language]
#[derive(Debug, Clone, Copy)]
pub struct Sprache {
    /// Standard-Präfix zum invertieren einer Flag.
    ///
    /// ## English
    /// Default prefix to invert a flag.
    pub invertiere_präfix: &'static str,
    /// Standard-Wert für die Meta-Variable im Hilfe-Text
    ///
    /// ## English
    /// Default-value for the meta-variable in the help text.
    pub meta_var: &'static str,
    /// Meta-Beschreibung für Optionen im Hilfe-Text.
    ///
    /// ## English
    /// Meta-description for options in the help text.
    pub optionen: &'static str,
    /// Beschreibung für Standard-Wert im Hilfe-Text.
    ///
    /// ## English
    /// Description for the default value in the help text.
    pub standard: &'static str,
    /// Beschreibung für mögliche Werte im Hilfe-Text.
    ///
    /// ## English
    /// Description for possible values in the help text.
    pub erlaubte_werte: &'static str,
    /// Beschreibung einer fehlenden Flag in einer Fehlermeldung.
    ///
    /// ## English
    /// Description for a missing flag in an error message.
    pub fehlende_flag: &'static str,
    /// Beschreibung eines fehlenden Wertes in einer Fehlermeldung.
    ///
    /// ## English
    /// Description for a missing value in an error message.
    pub fehlender_wert: &'static str,
    /// Beschreibung eines Parse-Fehlers in einer Fehlermeldung.
    ///
    /// ## English
    /// Description for a parse error in an error message.
    pub parse_fehler: &'static str,
    /// Beschreibung eines invaliden Strings in einer Fehlermeldung.
    ///
    /// ## English
    /// Description for an invalid String in an error message.
    pub invalider_string: &'static str,
    /// Beschreibung für ein nicht verwendetes Argument in einer Fehlermeldung.
    ///
    /// ## English
    /// Description for an unused argument in an error message.
    pub argument_nicht_verwendet: &'static str,
    /// Beschreibung für die Hilfe-Flag im automatisch erzeugten Hilfe-Text.
    ///
    /// ## English
    /// Description for the help flag in the automatically created help text.
    pub hilfe_beschreibung: &'static str,
    /// Lang-Name für die Hilfe-Flag.
    ///
    /// ## English
    /// Long name for the help flag.
    pub hilfe_lang: &'static str,
    /// Kurz-Name für die Hilfe-Flag.
    ///
    /// ## English
    /// Short name for the help flag.
    pub hilfe_kurz: &'static str,
    /// Beschreibung für die Version-Flag im automatisch erzeugten Hilfe-Text.
    ///
    /// ## English
    /// Description for the version flag in the automatically created help text.
    pub version_beschreibung: &'static str,
    /// Lang-Name für die Version-Flag.
    ///
    /// ## English
    /// Long name for the version flag.
    pub version_lang: &'static str,
    /// Kurz-Name für die Version-Flag.
    ///
    /// ## English
    /// Short name for the version flag.
    pub version_kurz: &'static str,
}

/// All strings required to produce help text and error message.
///
/// ## Deutsches Synonym
/// [Sprache]
pub type Language = Sprache;

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
        hilfe_beschreibung: "Zeige diesen Text an.",
        hilfe_lang: "hilfe",
        hilfe_kurz: "h",
        version_beschreibung: "Zeige die aktuelle Version an.",
        version_lang: "version",
        version_kurz: "v",
    };

    /// English Strings.
    pub const ENGLISH: Language = Sprache {
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
        hilfe_beschreibung: "Show this text.",
        hilfe_lang: "hilfe",
        hilfe_kurz: "h",
        version_beschreibung: "Show the current version.",
        version_lang: "version",
        version_kurz: "v",
    };
}
