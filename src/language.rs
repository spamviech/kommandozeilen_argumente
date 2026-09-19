//! Strings required to create help text and error messages.

/// All strings required to create help text and error messages.
#[derive(Debug, Clone, Copy)]
pub struct Language {
    /// Default prefix for long names.
    pub long_prefix: &'static str,
    /// Default prefix for short names.
    pub short_prefix: &'static str,
    /// Default prefix for inverting a flag.
    pub invert_prefix: &'static str,
    /// Default infix after the inversion prefix.
    pub invert_infix: &'static str,
    /// Default infix between a value name and its value.
    pub value_infix: &'static str,
    /// Default meta-variable in help text.
    pub meta_var: &'static str,
    /// Meta-description for options in help text.
    pub options: &'static str,
    /// Description for a default value in help text.
    pub default: &'static str,
    /// Description for possible values in help text.
    pub allowed_values: &'static str,
    /// Description for a missing flag error.
    pub missing_flag: &'static str,
    /// Description for a missing value error.
    pub missing_value: &'static str,
    /// Description for a parse error.
    pub parse_error: &'static str,
    /// Description for an invalid string error.
    pub invalid_string: &'static str,
    /// Description for an unused argument error.
    pub unused_argument: &'static str,
    /// Description for the generated help argument.
    pub help_description: &'static str,
    /// Long name for the generated help argument.
    pub help_long: &'static str,
    /// Short name for the generated help argument.
    pub help_short: &'static str,
    /// Description for the generated version argument.
    pub version_description: &'static str,
    /// Long name for the generated version argument.
    pub version_long: &'static str,
    /// Short name for the generated version argument.
    pub version_short: &'static str,
    /// Prefix before an argument syntax display.
    pub syntax_prefix: &'static str,
    /// Padding character for syntax displays.
    pub syntax_padding: char,
    /// Prefix for argument alternatives.
    pub alternative_prefix: &'static str,
    /// Separator for argument alternatives.
    pub alternative_separator: char,
}

impl Language {
    /// Converts German-named language strings to their English-primary type.
    #[must_use]
    pub const fn from_sprache(sprache: Sprache) -> Self {
        let Sprache {
            lang_präfix,
            kurz_präfix,
            invertiere_präfix,
            invertiere_infix,
            wert_infix,
            meta_var,
            optionen,
            standard,
            erlaubte_werte,
            fehlende_flag,
            fehlender_wert,
            parse_fehler,
            invalider_string,
            argument_nicht_verwendet,
            hilfe_beschreibung,
            hilfe_lang,
            hilfe_kurz,
            version_beschreibung,
            version_lang,
            version_kurz,
            syntax_präfix,
            syntax_padding,
            alternative_präfix,
            alternative_trennzeichen,
        } = sprache;
        Self {
            long_prefix: lang_präfix,
            short_prefix: kurz_präfix,
            invert_prefix: invertiere_präfix,
            invert_infix: invertiere_infix,
            value_infix: wert_infix,
            meta_var,
            options: optionen,
            default: standard,
            allowed_values: erlaubte_werte,
            missing_flag: fehlende_flag,
            missing_value: fehlender_wert,
            parse_error: parse_fehler,
            invalid_string: invalider_string,
            unused_argument: argument_nicht_verwendet,
            help_description: hilfe_beschreibung,
            help_long: hilfe_lang,
            help_short: hilfe_kurz,
            version_description: version_beschreibung,
            version_long: version_lang,
            version_short: version_kurz,
            syntax_prefix: syntax_präfix,
            syntax_padding,
            alternative_prefix: alternative_präfix,
            alternative_separator: alternative_trennzeichen,
        }
    }

    /// German strings.
    pub const GERMAN: Self = Self {
        long_prefix: "--",
        short_prefix: "-",
        invert_prefix: "kein",
        invert_infix: "-",
        value_infix: "=",
        meta_var: "WERT",
        options: "OPTIONEN",
        default: "Standard",
        allowed_values: "Erlaubte Werte",
        missing_flag: "Fehlende Flag",
        missing_value: "Fehlender Wert",
        parse_error: "Parse-Fehler",
        invalid_string: "Invalider String",
        unused_argument: "Nicht alle Argumente verwendet",
        help_description: "Zeige diesen Text an.",
        help_long: "hilfe",
        help_short: "h",
        version_description: "Zeige die aktuelle Version an.",
        version_long: "version",
        version_short: "v",
        syntax_prefix: "  ",
        syntax_padding: ' ',
        alternative_prefix: "| ",
        alternative_separator: '-',
    };

    /// English strings.
    pub const ENGLISH: Self = Self {
        long_prefix: "--",
        short_prefix: "-",
        invert_prefix: "no",
        invert_infix: "-",
        value_infix: "=",
        meta_var: "VALUE",
        options: "OPTIONS",
        default: "Default",
        allowed_values: "Possible values",
        missing_flag: "Missing Flag",
        missing_value: "Missing Value",
        parse_error: "Parse Error",
        invalid_string: "Invalid String",
        unused_argument: "Unused argument(s)",
        help_description: "Show this text.",
        help_long: "help",
        help_short: "h",
        version_description: "Show the current version.",
        version_long: "version",
        version_short: "v",
        syntax_prefix: "  ",
        syntax_padding: ' ',
        alternative_prefix: "| ",
        alternative_separator: '-',
    };
}

/// Alle Strings, die zum Erstellen von Hilfe-Text und Fehlermeldung notwendig sind.
#[derive(Debug, Clone, Copy)]
pub struct Sprache {
    /// Standard-Präfix für Lang-Namen.
    pub lang_präfix: &'static str,
    /// Standard-Präfix für Kurz-Namen.
    pub kurz_präfix: &'static str,
    /// Standard-Präfix zum Invertieren einer Flag.
    pub invertiere_präfix: &'static str,
    /// Standard-Infix nach dem Präfix zum Invertieren einer Flag.
    pub invertiere_infix: &'static str,
    /// Standard-Infix um einen Wert im selben Argument wie den Namen anzugeben.
    pub wert_infix: &'static str,
    /// Standard-Wert für die Meta-Variable im Hilfe-Text.
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
    /// Beschreibung für die Hilfe-Flag im automatisch erzeugten Hilfe-Text.
    pub hilfe_beschreibung: &'static str,
    /// Lang-Name für die Hilfe-Flag.
    pub hilfe_lang: &'static str,
    /// Kurz-Name für die Hilfe-Flag.
    pub hilfe_kurz: &'static str,
    /// Beschreibung für die Version-Flag im automatisch erzeugten Hilfe-Text.
    pub version_beschreibung: &'static str,
    /// Lang-Name für die Version-Flag.
    pub version_lang: &'static str,
    /// Kurz-Name für die Version-Flag.
    pub version_kurz: &'static str,
    /// Präfix vor der Syntax-Darstellung für ein Argument.
    pub syntax_präfix: &'static str,
    /// Füllzeichen für Syntax-Beschreibungen.
    pub syntax_padding: char,
    /// Präfix für Argument-Alternativen.
    pub alternative_präfix: &'static str,
    /// Trennzeichen zwischen Argument-Alternativen.
    pub alternative_trennzeichen: char,
}

impl Sprache {
    /// Konvertiert englisch-primäre Sprach-Strings in ihren deutsch benannten Spiegeltyp.
    #[must_use]
    pub const fn from_language(language: Language) -> Self {
        let Language {
            long_prefix,
            short_prefix,
            invert_prefix,
            invert_infix,
            value_infix,
            meta_var,
            options,
            default,
            allowed_values,
            missing_flag,
            missing_value,
            parse_error,
            invalid_string,
            unused_argument,
            help_description,
            help_long,
            help_short,
            version_description,
            version_long,
            version_short,
            syntax_prefix,
            syntax_padding,
            alternative_prefix,
            alternative_separator,
        } = language;
        Self {
            lang_präfix: long_prefix,
            kurz_präfix: short_prefix,
            invertiere_präfix: invert_prefix,
            invertiere_infix: invert_infix,
            wert_infix: value_infix,
            meta_var,
            optionen: options,
            standard: default,
            erlaubte_werte: allowed_values,
            fehlende_flag: missing_flag,
            fehlender_wert: missing_value,
            parse_fehler: parse_error,
            invalider_string: invalid_string,
            argument_nicht_verwendet: unused_argument,
            hilfe_beschreibung: help_description,
            hilfe_lang: help_long,
            hilfe_kurz: help_short,
            version_beschreibung: version_description,
            version_lang: version_long,
            version_kurz: version_short,
            syntax_präfix: syntax_prefix,
            syntax_padding,
            alternative_präfix: alternative_prefix,
            alternative_trennzeichen: alternative_separator,
        }
    }

    /// Deutsche Strings.
    pub const DEUTSCH: Self = Self::from_language(Language::GERMAN);
    /// Englische Strings.
    pub const ENGLISH: Self = Self::from_language(Language::ENGLISH);
}

impl From<Language> for Sprache {
    fn from(language: Language) -> Self {
        Self::from_language(language)
    }
}

impl From<Sprache> for Language {
    fn from(sprache: Sprache) -> Self {
        Self::from_sprache(sprache)
    }
}
