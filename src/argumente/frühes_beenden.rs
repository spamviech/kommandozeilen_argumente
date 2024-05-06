//! Flag-Argumente, die zu frühen Beenden führen.

use std::{
    borrow::Cow,
    convert::AsRef,
    env,
    ffi::{OsStr, OsString},
    path::Path,
};

use either::Either;
use itertools::Itertools;
use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;
use void::Void;

use crate::{
    argumente::{Argumente, Arguments},
    beschreibung::{
        contains_str, Beschreibung, Description, Konfiguration, KurzNamen, LangNamen, Name,
    },
    ergebnis::{namen_regex_hinzufügen, Ergebnis},
    sprache::{Language, Sprache},
    unicode::{Normalisiert, Vergleich},
};

impl<'t, T: 't, E: 't> Argumente<'t, T, E> {
    /// Erzeuge `--version`- und `--hilfe`-Flags, die zu vorzeitigem Beenden führen.
    /// Wie [`version_deutsch`](Argumente::version_deutsch) und [`hilfe`](Argumente::hilfe)
    /// mit synchronisiertem Programmnamen und Version.
    ///
    /// ## English version
    /// [`help_and_version`](Arguments::help_and_version)
    #[inline]
    pub fn hilfe_und_version(
        self,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        version: &str,
    ) -> Argumente<'t, T, E> {
        self.hilfe_und_version_mit_sprache(
            programm_name,
            programm_beschreibung,
            version,
            Sprache::DEUTSCH,
        )
    }

    /// Create `--version` and `--help` flags causing an early exit.
    /// Similar to using [`version_english`](Argumente::version_english) and [`help`](Argumente::help)
    /// with a synchronised program name and version.
    ///
    /// ## Deutsches Version
    /// [`hilfe_und_version`](Argumente::hilfe_und_version)
    #[inline]
    pub fn help_and_version(
        self,
        program_name: &str,
        program_description: Option<&str>,
        version: &str,
    ) -> Arguments<'t, T, E> {
        self.hilfe_und_version_mit_sprache(
            program_name,
            program_description,
            version,
            Sprache::ENGLISH,
        )
    }

    /// Erzeuge Flags, die zu vorzeitigem Beenden führen und Version, bzw. Hilfe-Text anzeigen.
    /// Wie [`version_mit_sprache`](Argumente::version_mit_sprache) und
    /// [`hilfe_mit_sprache`](Argumente::hilfe_mit_sprache) mit synchronisiertem
    /// Programmnamen und Version.
    ///
    /// ## English synonym
    /// [`help_and_version_with_language`](Arguments::help_and_version_with_language)
    #[inline]
    pub fn hilfe_und_version_mit_sprache(
        self,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        version: &str,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        self.version_mit_sprache(programm_name, version, sprache).hilfe_mit_sprache(
            programm_name,
            programm_beschreibung,
            Some(version),
            sprache,
        )
    }

    /// Create `--version` and `--help` flags causing an early exit.
    /// Similar to using [`version_english`](Argumente::version_english) and [`help`](Argumente::help)
    /// with a synchronised program name and version.
    ///
    /// ## Deutsches Synonym
    /// [`hilfe_und_version_mit_sprache`](Argumente::hilfe_und_version_mit_sprache)
    #[inline]
    pub fn help_and_version_with_language(
        self,
        program_name: &str,
        program_description: Option<&str>,
        version: &str,
        language: Language,
    ) -> Arguments<'t, T, E> {
        self.hilfe_und_version_mit_sprache(program_name, program_description, version, language)
    }

    /// Erzeuge eine `--version`-Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei die konfigurierte Programm-Version.
    ///
    /// ## English version
    /// [`version_english`](Arguments::version_english)
    #[inline]
    pub fn version_deutsch(self, programm_name: &str, version: &str) -> Argumente<'t, T, E> {
        self.version_mit_sprache(programm_name, version, Sprache::DEUTSCH)
    }

    /// Erzeuge eine Flag, die zu vorzeitigem Beenden führt
    /// und die konfigurierte Programm-Version anzeigt.
    ///
    /// ## English version
    /// [`version_with_names`](Arguments::version_with_names)
    #[inline]
    pub fn version_mit_namen(
        self,
        lang_namen: impl LangNamen<'t>,
        kurz_namen: impl KurzNamen<'t>,
        programm_name: &str,
        version: &str,
    ) -> Argumente<'t, T, E> {
        self.version_mit_namen_und_sprache(
            lang_namen,
            kurz_namen,
            programm_name,
            version,
            Sprache::DEUTSCH,
        )
    }

    /// Create a `--version` flag, causing an early exit.
    /// Shows the configured program version.
    ///
    /// ## Deutsches Version
    /// [`version_deutsch`](Argumente::version_deutsch)
    #[inline]
    pub fn version_english(self, program_name: &str, version: &str) -> Arguments<'t, T, E> {
        self.version_mit_sprache(program_name, version, Sprache::ENGLISH)
    }

    /// Create a flag causing an early exit which shows the configured program version.
    ///
    /// ## Deutsches Version
    /// [`version_mit_namen`](Argumente::version_mit_namen)
    #[inline]
    pub fn version_with_names(
        self,
        long_names: impl LangNamen<'t>,
        short_names: impl KurzNamen<'t>,
        program_name: &str,
        version: &str,
    ) -> Arguments<'t, T, E> {
        self.version_mit_namen_und_sprache(
            long_names,
            short_names,
            program_name,
            version,
            Sprache::ENGLISH,
        )
    }

    /// Erzeuge eine Flag, die zu vorzeitigem Beenden führt
    /// und die konfigurierte Programm-Version anzeigt.
    ///
    /// ## English synonym
    /// [`version_with_language`](Arguments::version_with_language)
    #[inline]
    pub fn version_mit_sprache(
        self,
        programm_name: &str,
        version: &str,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        self.version_mit_namen_und_sprache(
            sprache.version_lang,
            sprache.version_kurz,
            programm_name,
            version,
            sprache,
        )
    }

    /// Create a flag causing an early exit which shows the configured program version.
    ///
    /// ## Deutsches Synonym
    /// [`version_mit_sprache`](Argumente::version_mit_sprache)
    #[inline]
    pub fn version_with_language(
        self,
        program_name: &str,
        version: &str,
        language: Language,
    ) -> Arguments<'t, T, E> {
        self.version_mit_sprache(program_name, version, language)
    }

    /// Erzeuge eine Flag, die zu vorzeitigem Beenden führt
    /// und die konfigurierte Programm-Version anzeigt.
    ///
    /// ## English synonym
    /// [`version_with_names_and_language`](Arguments::version_with_names_and_language)
    #[inline]
    pub fn version_mit_namen_und_sprache(
        self,
        lang_namen: impl LangNamen<'t>,
        kurz_namen: impl KurzNamen<'t>,
        programm_name: &str,
        version: &str,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        let beschreibung = Beschreibung::neu_mit_sprache(
            lang_namen,
            kurz_namen,
            Some(sprache.version_beschreibung),
            None,
            sprache,
        );
        self.zeige_version(beschreibung, programm_name, version)
    }

    /// Create a flag causing an early exit which shows the configured program version.
    ///
    /// ## Deutsches Synonym
    /// [`version_mit_namen_und_sprache`](Argumente::version_mit_namen_und_sprache)
    #[inline]
    pub fn version_with_names_and_language(
        self,
        long_names: impl LangNamen<'t>,
        short_names: impl KurzNamen<'t>,
        program_name: &str,
        version: &str,
        language: Language,
    ) -> Arguments<'t, T, E> {
        self.version_mit_namen_und_sprache(long_names, short_names, program_name, version, language)
    }

    /// Erzeuge eine Flag, die zu vorzeitigem Beenden führt
    /// und die konfigurierte Programm-Version anzeigt.
    ///
    /// ## English synonym
    /// [`show_version`](Arguments::show_version)
    #[inline]
    pub fn zeige_version(
        self,
        beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        version: &str,
    ) -> Argumente<'t, T, E> {
        self.frühes_beenden(beschreibung, format!("{programm_name} {version}"))
    }

    /// Create a flag causing an early exit which shows the configured program version.
    ///
    /// ## Deutsches Synonym
    /// [`zeige_version`](Argumente::zeige_version)
    #[inline]
    pub fn show_version(
        self,
        description: Description<'t, Void>,
        program_name: &str,
        version: &str,
    ) -> Arguments<'t, T, E> {
        self.zeige_version(description, program_name, version)
    }

    /// Erzeuge eine `--hilfe`-Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei eine automatisch generierte Hilfe.
    ///
    /// ## English version
    /// [`help`](Arguments::help)
    #[inline]
    pub fn hilfe(
        self,
        programm_name: &str,
        program_description: Option<&str>,
        version: Option<&str>,
    ) -> Argumente<'t, T, E> {
        self.hilfe_mit_sprache(programm_name, program_description, version, Sprache::DEUTSCH)
    }

    /// Erzeuge eine Flag, die zu vorzeitigem Beenden führt
    /// und eine automatisch generierte Hilfe anzeigt.
    ///
    /// ## English version
    /// [`help_with_names`](Arguments::help_with_names)
    #[inline]
    pub fn hilfe_mit_namen(
        self,
        lang_namen: impl LangNamen<'t>,
        kurz_namen: impl KurzNamen<'t>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        version: Option<&str>,
    ) -> Argumente<'t, T, E> {
        self.hilfe_mit_namen_und_sprache(
            lang_namen,
            kurz_namen,
            programm_name,
            programm_beschreibung,
            version,
            Sprache::DEUTSCH,
        )
    }

    /// Create a `--help` flag, causing an early exit.
    /// Shows an automatically created help text.
    ///
    /// ## Deutsches Version
    /// [`hilfe`](Argumente::hilfe)
    #[inline]
    pub fn help(
        self,
        program_name: &str,
        program_description: Option<&str>,
        version: Option<&str>,
    ) -> Argumente<'t, T, E> {
        self.hilfe_mit_sprache(program_name, program_description, version, Sprache::ENGLISH)
    }

    /// Create a flag causing an early exit which shows an automatically created help text.
    ///
    /// ## Deutsches Version
    /// [`hilfe_mit_namen`](Argumente::hilfe_mit_namen)
    #[inline]
    pub fn help_with_names(
        self,
        long_names: impl LangNamen<'t>,
        short_names: impl KurzNamen<'t>,
        program_name: &str,
        program_description: Option<&str>,
        version: Option<&str>,
    ) -> Argumente<'t, T, E> {
        self.hilfe_mit_namen_und_sprache(
            long_names,
            short_names,
            program_name,
            program_description,
            version,
            Sprache::ENGLISH,
        )
    }

    /// Erstelle eine Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei eine automatisch konfigurierte Hilfe an.
    ///
    /// ## English synonym
    /// [`help_with_language`](Arguments::help_with_language)
    #[inline]
    pub fn hilfe_mit_sprache(
        self,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        version: Option<&str>,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        self.hilfe_mit_namen_und_sprache(
            sprache.hilfe_lang,
            sprache.hilfe_kurz,
            programm_name,
            programm_beschreibung,
            version,
            sprache,
        )
    }

    /// Create a flag causing an early exit which shows an automatically created help text.
    ///
    /// ## Deutsches Synonym
    /// [`hilfe_mit_sprache`](Argumente::hilfe_mit_sprache)
    #[inline]
    pub fn help_with_language(
        self,
        program_name: &str,
        program_description: Option<&str>,
        version: Option<&str>,
        language: Language,
    ) -> Arguments<'t, T, E> {
        self.hilfe_mit_sprache(program_name, program_description, version, language)
    }

    /// Erstelle eine Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei eine automatisch konfigurierte Hilfe an.
    ///
    /// ## English synonym
    /// [`help_with_names_and_language`](Arguments::help_with_names_and_language)
    #[inline]
    pub fn hilfe_mit_namen_und_sprache(
        self,
        lang_namen: impl LangNamen<'t>,
        kurz_namen: impl KurzNamen<'t>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        version: Option<&str>,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        let beschreibung = Beschreibung::neu_mit_sprache(
            lang_namen,
            kurz_namen,
            Some(sprache.hilfe_beschreibung),
            None,
            sprache,
        );
        self.erstelle_hilfe_mit_sprache(
            beschreibung,
            programm_name,
            programm_beschreibung,
            version,
            sprache,
        )
    }

    /// Create a flag causing an early exit which shows an automatically created help text.
    ///
    /// ## Deutsches Synonym
    /// [`hilfe_mit_namen_und_sprache`](Argumente::hilfe_mit_namen_und_sprache)
    #[inline]
    pub fn help_with_names_and_language(
        self,
        long_names: impl LangNamen<'t>,
        short_names: impl KurzNamen<'t>,
        program_name: &str,
        program_description: Option<&str>,
        version: Option<&str>,
        language: Language,
    ) -> Arguments<'t, T, E> {
        self.hilfe_mit_namen_und_sprache(
            long_names,
            short_names,
            program_name,
            program_description,
            version,
            language,
        )
    }

    /// Erstelle eine Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei eine automatisch konfigurierte Hilfe an.
    ///
    /// ## English synonym
    /// [`create_help_with_language`](Arguments::create_help_with_language)
    #[inline]
    pub fn erstelle_hilfe_mit_sprache(
        self,
        eigene_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        version: Option<&str>,
        sprache: Sprache,
    ) -> Argumente<'t, T, E> {
        self.erstelle_hilfe(
            eigene_beschreibung,
            programm_name,
            programm_beschreibung,
            version,
            sprache.optionen,
            sprache.standard,
            sprache.erlaubte_werte,
        )
    }

    /// Create a flag causing an early exit which shows an automatically created help text.
    ///
    /// ## Deutsches Synonym
    /// [`erstelle_hilfe_mit_sprache`](Argumente::erstelle_hilfe_mit_sprache)
    #[inline]
    pub fn create_help_with_language(
        self,
        help_description: Description<'t, Void>,
        program_name: &str,
        program_description: Option<&str>,
        version: Option<&str>,
        language: Language,
    ) -> Arguments<'t, T, E> {
        self.erstelle_hilfe_mit_sprache(
            help_description,
            program_name,
            program_description,
            version,
            language,
        )
    }

    /// Erstelle eine Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei eine automatisch konfigurierte Hilfe an.
    ///
    /// ## English synonym
    /// [`create_help`](Arguments::create_help)
    #[inline]
    pub fn erstelle_hilfe(
        self,
        eigene_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        version: Option<&str>,
        optionen: &str,
        standard: &str,
        erlaubte_werte: &str,
    ) -> Argumente<'t, T, E> {
        let hilfe_text = self.erstelle_hilfe_text_intern(
            Some(&eigene_beschreibung),
            programm_name,
            programm_beschreibung,
            version,
            optionen,
            standard,
            erlaubte_werte,
        );
        self.frühes_beenden(eigene_beschreibung, hilfe_text)
    }

    /// Create a flag causing an early exit which shows an automatically created help text.
    ///
    /// ## Deutsches Synonym
    /// [`erstelle_hilfe`](Argumente::erstelle_hilfe)
    #[inline]
    pub fn create_help(
        self,
        help_description: Description<'t, Void>,
        program_name: &str,
        program_description: Option<&str>,
        version: Option<&str>,
        options: &str,
        default: &str,
        allowed_values: &str,
    ) -> Arguments<'t, T, E> {
        self.erstelle_hilfe(
            help_description,
            program_name,
            program_description,
            version,
            options,
            default,
            allowed_values,
        )
    }

    /// Erstelle den Hilfe-Text für alle konfigurierten Argumente.
    ///
    /// ## English version
    /// [`help_text`](Arguments::help_text)
    #[inline]
    #[must_use]
    pub fn hilfe_text(
        &self,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        version: Option<&str>,
    ) -> String {
        self.erstelle_hilfe_text_mit_sprache(
            programm_name,
            programm_beschreibung,
            version,
            Sprache::DEUTSCH,
        )
    }

    /// Create the help-text for all configured arguments.
    ///
    /// ## Deutsches Version
    /// [`hilfe_text`](Argumente::hilfe_text)
    #[inline]
    #[must_use]
    pub fn help_text(
        &self,
        program_name: &str,
        program_description: Option<&str>,
        version: Option<&str>,
    ) -> String {
        self.erstelle_hilfe_text_mit_sprache(
            program_name,
            program_description,
            version,
            Sprache::ENGLISH,
        )
    }

    /// Erstelle den Hilfe-Text für alle konfigurierten Argumente.
    ///
    /// ## English synonym
    /// [`create_help_text_with_language`](Arguments:[create_help_text_with_language)
    #[inline]
    #[must_use]
    pub fn erstelle_hilfe_text_mit_sprache(
        &self,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        version: Option<&str>,
        sprache: Sprache,
    ) -> String {
        self.erstelle_hilfe_text(
            programm_name,
            programm_beschreibung,
            version,
            sprache.optionen,
            sprache.standard,
            sprache.erlaubte_werte,
        )
    }

    /// Create the help-text for all configured arguments.
    ///
    /// ## Deutsches Synonym
    /// [`erstelle_hilfe_text_mit_sprache`](Argumente::erstelle_hilfe_text_mit_sprache)
    #[inline]
    #[must_use]
    pub fn create_help_text_with_language(
        &self,
        programm_name: &str,
        program_description: Option<&str>,
        version: Option<&str>,
        language: Language,
    ) -> String {
        self.erstelle_hilfe_text_mit_sprache(programm_name, program_description, version, language)
    }

    /// Erstelle den Hilfe-Text für alle konfigurierten Argumente.
    ///
    /// ## English synonym
    /// [`create_help_text`](Arguments::create_help_text)
    #[inline]
    #[must_use]
    pub fn erstelle_hilfe_text(
        &self,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        version: Option<&str>,
        optionen: &str,
        standard: &str,
        erlaubte_werte: &str,
    ) -> String {
        self.erstelle_hilfe_text_intern(
            None,
            programm_name,
            programm_beschreibung,
            version,
            optionen,
            standard,
            erlaubte_werte,
        )
    }

    /// Create the help-text for all configured arguments.
    ///
    /// ## Deutsches Synonym
    /// [`erstelle_hilfe_text`](Argumente::erstelle_hilfe_text)
    #[inline]
    #[must_use]
    pub fn create_help_text(
        &self,
        programm_name: &str,
        program_description: Option<&str>,
        version: Option<&str>,
        options: &str,
        default: &str,
        allowed_values: &str,
    ) -> String {
        self.erstelle_hilfe_text(
            programm_name,
            program_description,
            version,
            options,
            default,
            allowed_values,
        )
    }

    /// Hilfsfunktion für [`Argumente::erstelle_hilfe_text_intern`].
    fn lang_regex(
        lang_präfix: &Vergleich<'_>,
        lang_namen: &NonEmpty<Vergleich<'_>>,
        flag_oder_wert: Either<&Option<(Vergleich<'_>, Vergleich<'_>)>, (&Vergleich<'_>, &str)>,
    ) -> String {
        let mut lang_regex = lang_präfix.as_ref().to_owned();
        match flag_oder_wert {
            Either::Left(invertiere_präfix_infix) => {
                if let Some((präfix, infix)) = invertiere_präfix_infix {
                    lang_regex.push('[');
                    lang_regex.push_str(präfix.as_ref());
                    lang_regex.push(']');
                    lang_regex.push_str(infix.as_ref());
                }
                namen_regex_hinzufügen(&mut lang_regex, &lang_namen.head, &lang_namen.tail);
            },
            Either::Right((wert_infix, meta_var)) => {
                namen_regex_hinzufügen(&mut lang_regex, &lang_namen.head, &lang_namen.tail);
                lang_regex.push('(');
                lang_regex.push_str(wert_infix.as_ref());
                lang_regex.push_str("| )");
                lang_regex.push_str(meta_var);
            },
        }
        lang_regex
    }

    /// Hilfsfunktion für [`Argumente::erstelle_hilfe_text_intern`].
    fn kurz_regex_hinzufügen(
        max_lang_regex_breite: usize,
        mut name_regex: String,
        lang_regex_breite: usize,
        kurz_präfix: &Vergleich<'_>,
        kurz_namen: &[Vergleich<'_>],
        flag_oder_wert: Either<&Option<(Vergleich<'_>, Vergleich<'_>)>, (&Vergleich<'_>, &str)>,
    ) -> String {
        if let Some((head, tail)) = kurz_namen.split_first() {
            #[allow(clippy::arithmetic_side_effects)]
            let einrücken = " ".repeat(max_lang_regex_breite - lang_regex_breite);
            name_regex.push_str(&einrücken);
            name_regex.push_str(" | ");
            name_regex.push_str(kurz_präfix.as_ref());
            namen_regex_hinzufügen(&mut name_regex, head, tail);
            if let Either::Right((wert_infix, meta_var)) = flag_oder_wert {
                name_regex.push('[');
                name_regex.push_str(wert_infix.as_ref());
                name_regex.push_str("| ]");
                name_regex.push_str(meta_var.as_ref());
            }
        }
        name_regex
    }

    /// Hilfsfunktion für [`Argumente::erstelle_hilfe_text_intern`].
    #[allow(clippy::too_many_arguments)]
    fn hilfe_zeile(
        standard: &str,
        erlaubte_werte: &str,
        max_name_regex_breite: usize,
        hilfe_text: &mut String,
        name_regex: &str,
        name_regex_breite: usize,
        beschreibung: &Beschreibung<'_, String>,
        mögliche_werte: &Option<NonEmpty<String>>,
    ) {
        hilfe_text.push_str("  ");
        hilfe_text.push_str(name_regex);
        #[allow(clippy::arithmetic_side_effects)]
        let einrücken = " ".repeat(2 + max_name_regex_breite - name_regex_breite);
        hilfe_text.push_str(&einrücken);
        if let Some(hilfe) = &beschreibung.hilfe {
            hilfe_text.push_str(hilfe);
        }
        if let Some(werte) = mögliche_werte {
            if beschreibung.hilfe.is_some() {
                hilfe_text.push(' ');
            }
            hilfe_text.push('[');
            hilfe_text.push_str(erlaubte_werte);
            hilfe_text.push_str(": ");
            hilfe_text.push_str(&werte.head);
            for wert in &werte.tail {
                hilfe_text.push_str(", ");
                hilfe_text.push_str(wert);
            }
            if beschreibung.standard.is_some() {
                hilfe_text.push_str(" | ");
            } else {
                hilfe_text.push(']');
            }
        }
        if let Some(standard_wert) = &beschreibung.standard {
            if !mögliche_werte.is_some() {
                if beschreibung.hilfe.is_some() {
                    hilfe_text.push(' ');
                }
                hilfe_text.push('[');
            }
            hilfe_text.push_str(standard);
            hilfe_text.push_str(": ");
            hilfe_text.push_str(standard_wert);
            hilfe_text.push(']');
        }
        hilfe_text.push('\n');
    }

    /// Implementierung für [`erstelle_hilfe`](Argumente::erstelle_hilfe) und
    /// [`erstelle_hilfe_text`](Argumente::erstelle_hilfe_text).
    #[allow(clippy::too_many_arguments)]
    fn erstelle_hilfe_text_intern(
        &self,
        eigene_beschreibung: Option<&Beschreibung<'_, Void>>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        version: Option<&str>,
        optionen: &str,
        standard: &str,
        erlaubte_werte: &str,
    ) -> String {
        let current_exe = env::current_exe().ok();
        let exe_name = current_exe
            .as_deref()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .unwrap_or(programm_name);
        let mut name = programm_name.to_owned();
        if let Some(version) = version {
            name.push(' ');
            name.push_str(version);
        }
        let programm_beschreibung = programm_beschreibung
            .map(|beschreibung| format!("\n{beschreibung}"))
            .unwrap_or_default();
        let mut hilfe_text =
            format!("{name}{programm_beschreibung}\n\n{exe_name} [{optionen}]\n\n{optionen}:\n");
        let eigener_arg_string = eigene_beschreibung.map(|beschreibung| Konfiguration::Flag {
            beschreibung: beschreibung.clone().als_string_beschreibung().0,
            invertiere_präfix_infix: None,
        });
        let none = None;
        let mut max_lang_regex_breite = 0;
        let mut lang_regex_vec = Vec::new();
        for arg_string in self.konfigurationen().chain(eigener_arg_string.iter()) {
            let (beschreibung, flag_oder_wert, mögliche_werte) = match arg_string {
                Konfiguration::Flag { beschreibung, invertiere_präfix_infix } => {
                    (beschreibung, Either::Left(invertiere_präfix_infix), &none)
                },
                Konfiguration::Wert { beschreibung, wert_infix, meta_var, mögliche_werte } => {
                    (beschreibung, Either::Right((wert_infix, *meta_var)), mögliche_werte)
                },
            };
            let lang_regex = Self::lang_regex(
                &beschreibung.name.lang_präfix,
                &beschreibung.name.lang,
                flag_oder_wert,
            );
            let lang_regex_breite = lang_regex.graphemes(true).count();
            max_lang_regex_breite = max_lang_regex_breite.max(lang_regex_breite);
            lang_regex_vec.push((
                lang_regex,
                lang_regex_breite,
                beschreibung,
                flag_oder_wert,
                mögliche_werte,
            ));
        }
        let mut max_name_regex_breite = 0;
        let mut name_regex_vec = Vec::new();
        for (lang_regex, lang_regex_breite, beschreibung, flag_oder_wert, mögliche_werte) in
            lang_regex_vec
        {
            let name_regex = Self::kurz_regex_hinzufügen(
                max_lang_regex_breite,
                lang_regex,
                lang_regex_breite,
                &beschreibung.name.kurz_präfix,
                &beschreibung.name.kurz,
                flag_oder_wert,
            );
            let name_regex_breite = name_regex.graphemes(true).count();
            max_name_regex_breite = max_name_regex_breite.max(name_regex_breite);
            name_regex_vec.push((name_regex, name_regex_breite, beschreibung, mögliche_werte));
        }
        for (name_regex, name_regex_breite, beschreibung, mögliche_werte) in name_regex_vec {
            Self::hilfe_zeile(
                standard,
                erlaubte_werte,
                max_name_regex_breite,
                &mut hilfe_text,
                &name_regex,
                name_regex_breite,
                beschreibung,
                mögliche_werte,
            );
        }
        hilfe_text
    }

    /// Erstelle eine Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei die übergebene Nachricht an.
    ///
    /// ## English synonym
    /// [`early_exit`](Arguments::early_exit)
    #[inline]
    pub fn frühes_beenden(
        self,
        beschreibung: Beschreibung<'t, Void>,
        nachricht: impl Into<Cow<'t, str>>,
    ) -> Argumente<'t, T, E> {
        let Argumente { mut konfigurationen, mut flag_kurzformen, parse } = self;
        let name_lang_präfix = beschreibung.name.lang_präfix.clone();
        let name_lang = beschreibung.name.lang.clone();
        let name_kurz_präfix = beschreibung.name.kurz_präfix.clone();
        let name_kurz = beschreibung.name.kurz.clone();
        let (beschreibung_string, _standard) = beschreibung.als_string_beschreibung();
        flag_kurzformen
            .entry(beschreibung_string.name.kurz_präfix.clone())
            .or_insert(Vec::new())
            .extend(beschreibung_string.name.kurz.iter().cloned());
        konfigurationen.push(Konfiguration::Flag {
            beschreibung: beschreibung_string,
            invertiere_präfix_infix: None,
        });
        let nachricht_cow = nachricht.into();
        Argumente {
            konfigurationen,
            flag_kurzformen,
            parse: Box::new(move |args| {
                let name_kurz_existiert = !name_kurz.is_empty();
                let mut nicht_selbst_verwendet = Vec::new();
                let mut nachrichten: Vec<Cow<'t, str>> = Vec::new();
                let mut zeige_nachricht = || nachrichten.push(nachricht_cow.clone());
                for arg in args {
                    if let Some(string) = arg.as_ref().and_then(|os_string| os_string.to_str()) {
                        let normalisiert = Normalisiert::neu(string);
                        if let Some(lang_str) = name_lang_präfix.strip_als_präfix(&normalisiert) {
                            if contains_str(&name_lang, lang_str) {
                                zeige_nachricht();
                                nicht_selbst_verwendet.push(None);
                                continue;
                            }
                        } else if name_kurz_existiert {
                            if let Some(kurz_str) =
                                name_kurz_präfix.strip_als_präfix(&normalisiert)
                            {
                                if kurz_str
                                    .graphemes(true)
                                    .exactly_one()
                                    .map(|name| contains_str(&name_kurz, name))
                                    .unwrap_or(false)
                                {
                                    zeige_nachricht();
                                    nicht_selbst_verwendet.push(None);
                                    continue;
                                }
                            }
                        } else {
                            // kein match für "{lang_präfix}.*" und Argument hat keine kurz_namen
                        }
                    }
                    nicht_selbst_verwendet.push(arg);
                }
                let (ergebnis, nicht_verwendet) = parse(nicht_selbst_verwendet);
                let finales_ergebnis = match ergebnis {
                    Ergebnis::FrühesBeenden(mut frühes_beenden) => {
                        frühes_beenden.tail.extend(nachrichten);
                        Ergebnis::FrühesBeenden(frühes_beenden)
                    },
                    Ergebnis::Wert(_) | Ergebnis::Fehler(_) => {
                        if let Some(frühes_beenden) = NonEmpty::from_vec(nachrichten) {
                            Ergebnis::FrühesBeenden(frühes_beenden)
                        } else {
                            ergebnis
                        }
                    },
                };
                (finales_ergebnis, nicht_verwendet)
            }),
        }
    }

    /// Create a flag which causes an early exit and shows the given message.
    ///
    /// ## Deutsches Synonym
    /// [`frühes_beenden`](Argumente::frühes_beenden)
    #[inline]
    pub fn early_exit(
        self,
        description: Description<'t, Void>,
        message: impl Into<Cow<'t, str>>,
    ) -> Arguments<'t, T, E> {
        self.frühes_beenden(description, message)
    }
}

/// Es handelt sich um ein Flag-Argument, das zu frühem beenden führt.
///
/// ## English
/// It is a flag argument, causing an early exit.
#[derive(Debug)]
pub struct FrühesBeenden<'t> {
    /// Allgemeine Beschreibung des Arguments.
    ///
    /// ## English
    /// General description of the argument.
    pub beschreibung: Beschreibung<'t, Void>,

    /// Die angezeigte Nachricht.
    ///
    /// ## English
    /// The message.
    pub nachricht: Cow<'t, str>,
}

impl<'t> FrühesBeenden<'t> {
    /// Parse die übergebenen Argumente und erzeuge den zugehörigen Wert.
    ///
    /// ## English
    /// Parse the given arguments and return the corresponding value.
    #[inline]
    pub fn parse<F>(
        self,
        args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, (), F>, Vec<Option<OsString>>) {
        let FrühesBeenden { beschreibung, nachricht } = self;
        let Beschreibung { name, hilfe: _, standard } = beschreibung;
        let mut nicht_verwendet = Vec::new();
        let mut iter = args.into_iter();
        while let Some(arg_opt) = iter.next() {
            if let Some(arg) = &arg_opt {
                if name.parse_frühes_beenden(arg) {
                    nicht_verwendet.push(None);
                    nicht_verwendet.extend(iter);
                    return (
                        Ergebnis::FrühesBeenden(NonEmpty::singleton(nachricht)),
                        nicht_verwendet,
                    );
                }
            }
            nicht_verwendet.push(arg_opt);
        }
        let ergebnis =
            if let Some(wert) = standard { void::unreachable(wert) } else { Ergebnis::Wert(()) };
        (ergebnis, nicht_verwendet)
    }

    /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
    ///
    /// ## English
    /// Create the Message for the syntax of the arguments and the corresponding help text.
    #[inline]
    pub fn erzeuge_hilfe_text(&self) -> (String, Option<Cow<'_, str>>) {
        let FrühesBeenden { beschreibung, nachricht: _ } = self;
        let Beschreibung { name, hilfe, standard } = beschreibung;
        let Name { lang_präfix, lang, kurz_präfix, kurz } = name;
        let mut syntax = String::new();
        syntax.push_str(lang_präfix.as_str());
        let NonEmpty { head, tail } = lang;
        Name::möglichkeiten_als_regex(head, tail.as_slice(), &mut syntax);
        if let Some((kurz_head, kurz_tail)) = kurz.split_first() {
            syntax.push_str(" | ");
            syntax.push_str(kurz_präfix.as_str());
            Name::möglichkeiten_als_regex(kurz_head, kurz_tail, &mut syntax);
        }
        if let Some(void) = standard {
            void::unreachable(*void)
        }
        let hilfe_text = hilfe.map(Cow::Borrowed);
        (syntax, hilfe_text)
    }
}
