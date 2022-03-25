//! Alle Strings, die zum erstellen von Hilfe-Text und Fehlermeldung notwendig sind.

/// Alle Strings, die zum erstellen von Hilfe-Text und Fehlermeldung notwendig sind.
///
/// ## English synonym
/// [Language]
#[derive(Debug, Clone, Copy)]
pub struct Sprache {
    /// Standard-Präfix für LangNamen.
    ///
    /// ## English
    /// Default prefix for long names.
    pub lang_präfix: &'static str,

    /// Standard-Präfix für KurzNamen.
    ///
    /// ## English
    /// Default prefix for short names.
    pub kurz_präfix: &'static str,

    /// Standard-Präfix zum invertieren einer Flag.
    ///
    /// ## English
    /// Default prefix to invert a flag.
    pub invertiere_präfix: &'static str,

    /// Standard-Infix nach dem Präfix zum invertieren einer Flag.
    ///
    /// ## English
    /// Default infix after the prefix to invert a flag.
    pub invertiere_infix: &'static str,

    /// Standard-Infix um einen Wert im selben Argument wie den Namen anzugeben.
    ///
    /// ## English
    /// Default infix to give a value in the same argument as the name.
    pub wert_infix: &'static str,

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

impl From<Language> for Sprache {
    fn from(input: Language) -> Self {
        input.sprache()
    }
}

impl Sprache {
    /// Convert to the english representation.
    ///
    /// ## Deutsch
    /// Konvertiere in die englische Repräsentation.
    pub const fn language(self) -> Language {
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
        } = self;
        Language {
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
        }
    }

    /// Deutsche Strings.
    pub const DEUTSCH: Sprache = Sprache {
        lang_präfix: "--",
        kurz_präfix: "-",
        invertiere_präfix: "kein",
        invertiere_infix: "-",
        wert_infix: "=",
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

    /// Englische Strings.
    pub const ENGLISCH: Sprache = Language::ENGLISH.sprache();
}

/// All strings required to produce help text and error message.
///
/// ## Deutsches Synonym
/// [Sprache]
#[derive(Debug, Clone, Copy)]
pub struct Language {
    /// Default prefix for long names.
    ///
    /// ## Deutsch
    /// Standard-Präfix für LangNamen.
    pub long_prefix: &'static str,

    /// Default prefix for short names.
    ///
    /// ## Deutsch
    ///Standard-Präfix für KurzNamen.
    pub short_prefix: &'static str,

    /// Default prefix to invert a flag.
    ///
    /// ## Deutsch
    /// Standard-Präfix zum invertieren einer Flag.
    pub invert_prefix: &'static str,

    /// Default infix after the prefix to invert a flag.
    ///
    /// ## Deutsch
    /// Standard-Infix nach dem Präfix zum invertieren einer Flag.
    pub invert_infix: &'static str,

    /// Default infix to give a value in the same argument as the name.
    ///
    /// ## Deutsch
    /// Standard-Infix um einen Wert im selben Argument wie den Namen anzugeben.
    pub value_infix: &'static str,

    /// Default-value for the meta-variable in the help text.
    ///
    /// ## Deutsch
    /// Standard-Wert für die Meta-Variable im Hilfe-Text
    pub meta_var: &'static str,

    /// Meta-description for options in the help text.
    ///
    /// ## Deutsch
    /// Meta-Beschreibung für Optionen im Hilfe-Text.
    pub options: &'static str,

    /// Description for the default value in the help text.
    ///
    /// ## Deutsch
    /// Beschreibung für Standard-Wert im Hilfe-Text.
    pub default: &'static str,

    /// Description for possible values in the help text.
    ///
    /// ## Deutsch
    /// Beschreibung für mögliche Werte im Hilfe-Text.
    pub allowed_values: &'static str,

    /// Description for a missing flag in an error message.
    ///
    /// ## Deutsch
    /// Beschreibung einer fehlenden Flag in einer Fehlermeldung.
    pub missing_flag: &'static str,

    /// Description for a missing value in an error message.
    ///
    /// ## Deutsch
    /// Beschreibung eines fehlenden Wertes in einer Fehlermeldung.
    pub missing_value: &'static str,

    /// Description for a parse error in an error message.
    ///
    /// ## Deutsch
    /// Beschreibung eines Parse-Fehlers in einer Fehlermeldung.
    pub parse_error: &'static str,

    /// Description for an invalid String in an error message.
    ///
    /// ## Deutsch
    /// Beschreibung eines invaliden Strings in einer Fehlermeldung.
    pub invalid_string: &'static str,

    /// Description for an unused argument in an error message.
    ///
    /// ## Deutsch
    /// Beschreibung für ein nicht verwendetes Argument in einer Fehlermeldung.
    pub unused_argument: &'static str,

    /// Description for the help flag in the automatically created help text.
    ///
    /// ## Deutsch
    /// Beschreibung für die Hilfe-Flag im automatisch erzeugten Hilfe-Text.
    pub help_description: &'static str,

    /// Long name for the help flag.
    ///
    /// ## Deutsch
    /// Lang-Name für die Hilfe-Flag.
    pub help_long: &'static str,

    /// Short name for the help flag.
    ///
    /// ## Deutsch
    /// Kurz-Name für die Hilfe-Flag.
    pub help_short: &'static str,

    /// Description for the version flag in the automatically created help text.
    ///
    /// ## Deutsch
    /// Beschreibung für die Version-Flag im automatisch erzeugten Hilfe-Text.
    pub version_description: &'static str,

    /// Long name for the version flag.
    ///
    /// ## Deutsch
    /// Lang-Name für die Version-Flag.
    pub version_long: &'static str,

    /// Short name for the version flag.
    ///
    /// ## Deutsch
    /// Kurz-Name für die Version-Flag.
    pub version_short: &'static str,
}

impl From<Sprache> for Language {
    fn from(input: Sprache) -> Self {
        input.language()
    }
}

impl Language {
    /// Konvertiere in die deutsche Repräsentation.
    ///
    /// ## English
    /// Convert to the german representation.
    pub const fn sprache(self) -> Sprache {
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
        } = self;
        Sprache {
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
        }
    }

    /// German Strings.
    pub const GERMAN: Language = Sprache::DEUTSCH.language();

    /// English Strings.
    pub const ENGLISH: Language = Language {
        long_prefix: "--",
        short_prefix: "-",
        invert_prefix: "no",
        invert_infix: "-",
        value_infix: "=",
        meta_var: "VALUE",
        options: "OPTIONS",
        default: "Default",
        allowed_values: "Allowed values",
        missing_flag: "Missing Flag",
        missing_value: "Missing Value",
        parse_error: "Parse Error",
        invalid_string: "Invalid String",
        unused_argument: "Unused argument(s)",
        help_description: "Show this text.",
        help_long: "hilfe",
        help_short: "h",
        version_description: "Show the current version.",
        version_long: "version",
        version_short: "v",
    };
}
