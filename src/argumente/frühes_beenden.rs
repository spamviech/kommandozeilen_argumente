//! Flag-Argumente, die zu frühen Beenden führen.

use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use either::Either;
use itertools::Itertools;
use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;
use void::Void;

use crate::{
    argumente::{Argumente, Arguments},
    beschreibung::{contains_str, Beschreibung, Description, Konfiguration, KurzNamen, LangNamen},
    ergebnis::{namen_regex_hinzufügen, Ergebnis},
    sprache::{Language, Sprache},
};

// TODO benenne [Argumente::konfigurationen], [Arguments::configurations] um eigene Hilfe zu erzeugen.
impl<T: 'static, E: 'static> Argumente<T, E> {
    /// Erzeuge `--version`- und `--hilfe`-Flags, die zu vorzeitigem Beenden führen.
    /// Wie [version_deutsch](Argumente::version_deutsch) und [hilfe](Argumente::hilfe)
    /// mit synchronisiertem Programmnamen und Version.
    #[inline(always)]
    pub fn hilfe_und_version(self, programm_name: &str, version: &str) -> Argumente<T, E> {
        self.hilfe_und_version_mit_sprache(programm_name, version, Sprache::DEUTSCH)
    }

    /// Create `--version` and `--help` flags causing an early exit.
    /// Similar to using [version_english](Argumente::version_english) and [help](Argumente::help)
    /// with a synchronised program name and version.
    #[inline(always)]
    pub fn help_and_version(self, program_name: &str, version: &str) -> Arguments<T, E> {
        self.hilfe_und_version_mit_sprache(program_name, version, Sprache::ENGLISH)
    }

    /// Erzeuge Flags, die zu vorzeitigem Beenden führen und Version, bzw. Hilfe-Text anzeigen.
    /// Wie [version_mit_sprache](Argumente::version_mit_sprache) und
    /// [hilfe_mit_sprache](Argumente::hilfe_mit_sprache) mit synchronisiertem
    /// Programmnamen und Version.
    #[inline(always)]
    pub fn hilfe_und_version_mit_sprache(
        self,
        programm_name: &str,
        version: &str,
        sprache: Sprache,
    ) -> Argumente<T, E> {
        self.version_mit_sprache(programm_name, version, sprache).hilfe_mit_sprache(
            programm_name,
            Some(version),
            sprache,
        )
    }

    /// Create `--version` and `--help` flags causing an early exit.
    /// Similar to using [version_english](Argumente::version_english) and [help](Argumente::help)
    /// with a synchronised program name and version.
    #[inline(always)]
    pub fn help_and_version_with_language(
        self,
        program_name: &str,
        version: &str,
        language: Language,
    ) -> Arguments<T, E> {
        self.hilfe_und_version_mit_sprache(program_name, version, language)
    }

    /// Erzeuge eine `--version`-Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei die konfigurierte Programm-Version.
    #[inline(always)]
    pub fn version_deutsch(self, programm_name: &str, version: &str) -> Argumente<T, E> {
        self.version_mit_sprache(programm_name, version, Sprache::DEUTSCH)
    }

    /// Erzeuge eine Flag, die zu vorzeitigem Beenden führt
    /// und die konfigurierte Programm-Version anzeigt.
    pub fn version_mit_namen(
        self,
        lang_namen: impl LangNamen,
        kurz_namen: impl KurzNamen,
        programm_name: &str,
        version: &str,
    ) -> Argumente<T, E> {
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
    #[inline(always)]
    pub fn version_english(self, program_name: &str, version: &str) -> Arguments<T, E> {
        self.version_mit_sprache(program_name, version, Sprache::ENGLISH)
    }

    /// Create a flag causing an early exit which shows the configured program version.
    pub fn version_with_names(
        self,
        long_names: impl LangNamen,
        short_names: impl KurzNamen,
        program_name: &str,
        version: &str,
    ) -> Arguments<T, E> {
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
    #[inline(always)]
    pub fn version_mit_sprache(
        self,
        programm_name: &str,
        version: &str,
        sprache: Sprache,
    ) -> Argumente<T, E> {
        self.version_mit_namen_und_sprache(
            sprache.version_lang,
            sprache.version_kurz,
            programm_name,
            version,
            sprache,
        )
    }

    /// Create a flag causing an early exit which shows the configured program version.
    #[inline(always)]
    pub fn version_with_language(
        self,
        program_name: &str,
        version: &str,
        language: Language,
    ) -> Arguments<T, E> {
        self.version_mit_sprache(program_name, version, language)
    }

    /// Erzeuge eine Flag, die zu vorzeitigem Beenden führt
    /// und die konfigurierte Programm-Version anzeigt.
    pub fn version_mit_namen_und_sprache(
        self,
        lang_namen: impl LangNamen,
        kurz_namen: impl KurzNamen,
        programm_name: &str,
        version: &str,
        sprache: Sprache,
    ) -> Argumente<T, E> {
        let beschreibung = Beschreibung::neu(
            lang_namen,
            kurz_namen,
            Some(sprache.version_beschreibung.to_owned()),
            None,
        );
        self.zeige_version(beschreibung, programm_name, version)
    }

    /// Create a flag causing an early exit which shows the configured program version.
    #[inline(always)]
    pub fn version_with_names_and_language(
        self,
        long_names: impl LangNamen,
        short_names: impl KurzNamen,
        program_name: &str,
        version: &str,
        language: Language,
    ) -> Arguments<T, E> {
        self.version_mit_namen_und_sprache(long_names, short_names, program_name, version, language)
    }

    /// Erzeuge eine Flag, die zu vorzeitigem Beenden führt
    /// und die konfigurierte Programm-Version anzeigt.
    #[inline(always)]
    pub fn zeige_version(
        self,
        beschreibung: Beschreibung<Void>,
        programm_name: &str,
        version: &str,
    ) -> Argumente<T, E> {
        self.frühes_beenden(beschreibung, format!("{} {}", programm_name, version))
    }

    /// Create a flag causing an early exit which shows the configured program version.
    #[inline(always)]
    pub fn show_version(
        self,
        description: Description<Void>,
        program_name: &str,
        version: &str,
    ) -> Arguments<T, E> {
        self.zeige_version(description, program_name, version)
    }

    /// Erzeuge eine `--hilfe`-Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei eine automatisch generierte Hilfe.
    #[inline(always)]
    pub fn hilfe(self, programm_name: &str, version: Option<&str>) -> Argumente<T, E> {
        self.hilfe_mit_sprache(programm_name, version, Sprache::DEUTSCH)
    }

    /// Erzeuge eine Flag, die zu vorzeitigem Beenden führt
    /// und eine automatisch generierte Hilfe anzeigt.
    #[inline(always)]
    pub fn hilfe_mit_namen(
        self,
        lang_namen: impl LangNamen,
        kurz_namen: impl KurzNamen,
        programm_name: &str,
        version: Option<&str>,
    ) -> Argumente<T, E> {
        self.hilfe_mit_namen_und_sprache(
            lang_namen,
            kurz_namen,
            programm_name,
            version,
            Sprache::DEUTSCH,
        )
    }

    /// Create a `--help` flag, causing an early exit.
    /// Shows an automatically created help text.
    #[inline(always)]
    pub fn help(self, program_name: &str, version: Option<&str>) -> Argumente<T, E> {
        self.hilfe_mit_sprache(program_name, version, Sprache::ENGLISH)
    }

    /// Create a flag causing an early exit which shows an automatically created help text.
    #[inline(always)]
    pub fn help_with_names(
        self,
        long_names: impl LangNamen,
        short_names: impl KurzNamen,
        program_name: &str,
        version: Option<&str>,
    ) -> Argumente<T, E> {
        self.hilfe_mit_namen_und_sprache(
            long_names,
            short_names,
            program_name,
            version,
            Sprache::ENGLISH,
        )
    }

    /// Erstelle eine Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei eine automatisch konfigurierte Hilfe an.
    #[inline(always)]
    pub fn hilfe_mit_sprache(
        self,
        programm_name: &str,
        version: Option<&str>,
        sprache: Sprache,
    ) -> Argumente<T, E> {
        self.hilfe_mit_namen_und_sprache(
            sprache.hilfe_lang,
            sprache.hilfe_kurz,
            programm_name,
            version,
            sprache,
        )
    }

    /// Create a flag causing an early exit which shows an automatically created help text.
    #[inline(always)]
    pub fn help_with_language(
        self,
        program_name: &str,
        version: Option<&str>,
        language: Language,
    ) -> Arguments<T, E> {
        self.hilfe_mit_sprache(program_name, version, language)
    }

    /// Erstelle eine Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei eine automatisch konfigurierte Hilfe an.
    #[inline(always)]
    pub fn hilfe_mit_namen_und_sprache(
        self,
        lang_namen: impl LangNamen,
        kurz_namen: impl KurzNamen,
        programm_name: &str,
        version: Option<&str>,
        sprache: Sprache,
    ) -> Argumente<T, E> {
        let beschreibung = Beschreibung::neu(
            lang_namen,
            kurz_namen,
            Some(sprache.hilfe_beschreibung.to_owned()),
            None,
        );
        self.erstelle_hilfe_mit_sprache(beschreibung, programm_name, version, sprache)
    }

    /// Create a flag causing an early exit which shows an automatically created help text.
    #[inline(always)]
    pub fn help_with_names_and_language(
        self,
        long_names: impl LangNamen,
        short_names: impl KurzNamen,
        program_name: &str,
        version: Option<&str>,
        language: Language,
    ) -> Arguments<T, E> {
        self.hilfe_mit_namen_und_sprache(long_names, short_names, program_name, version, language)
    }

    /// Erstelle eine Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei eine automatisch konfigurierte Hilfe an.
    #[inline(always)]
    pub fn erstelle_hilfe_mit_sprache(
        self,
        eigene_beschreibung: Beschreibung<Void>,
        programm_name: &str,
        version: Option<&str>,
        sprache: Sprache,
    ) -> Argumente<T, E> {
        self.erstelle_hilfe(
            eigene_beschreibung,
            programm_name,
            version,
            sprache.optionen,
            sprache.standard,
            sprache.erlaubte_werte,
        )
    }

    /// Create a flag causing an early exit which shows an automatically created help text.
    #[inline(always)]
    pub fn create_help_with_language(
        self,
        help_description: Description<Void>,
        program_name: &str,
        version: Option<&str>,
        language: Language,
    ) -> Arguments<T, E> {
        self.erstelle_hilfe_mit_sprache(help_description, program_name, version, language)
    }

    /// Erstelle eine Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei eine automatisch konfigurierte Hilfe an.
    #[inline(always)]
    pub fn erstelle_hilfe(
        self,
        eigene_beschreibung: Beschreibung<Void>,
        programm_name: &str,
        version: Option<&str>,
        optionen: &str,
        standard: &str,
        erlaubte_werte: &str,
    ) -> Argumente<T, E> {
        let hilfe_text = self.erstelle_hilfe_text_intern(
            Some(&eigene_beschreibung),
            programm_name,
            version,
            optionen,
            standard,
            erlaubte_werte,
        );
        self.frühes_beenden(eigene_beschreibung, hilfe_text)
    }

    /// Create a flag causing an early exit which shows an automatically created help text.
    #[inline(always)]
    pub fn create_help(
        self,
        help_description: Description<Void>,
        program_name: &str,
        version: Option<&str>,
        options: &str,
        default: &str,
        allowed_values: &str,
    ) -> Arguments<T, E> {
        self.erstelle_hilfe(
            help_description,
            program_name,
            version,
            options,
            default,
            allowed_values,
        )
    }

    /// Erstelle den Hilfe-Text für alle konfigurierten Argumente.
    #[inline(always)]
    pub fn hilfe_text(&self, programm_name: &str, version: Option<&str>) -> String {
        self.erstelle_hilfe_text_mit_sprache(programm_name, version, Sprache::DEUTSCH)
    }

    /// Create the help-text for all configured arguments.
    #[inline(always)]
    pub fn help_text(&self, program_name: &str, version: Option<&str>) -> String {
        self.erstelle_hilfe_text_mit_sprache(program_name, version, Sprache::ENGLISH)
    }

    /// Erstelle den Hilfe-Text für alle konfigurierten Argumente.
    #[inline(always)]
    pub fn erstelle_hilfe_text_mit_sprache(
        &self,
        programm_name: &str,
        version: Option<&str>,
        sprache: Sprache,
    ) -> String {
        self.erstelle_hilfe_text(
            programm_name,
            version,
            sprache.optionen,
            sprache.standard,
            sprache.erlaubte_werte,
        )
    }

    /// Create the help-text for all configured arguments.
    #[inline(always)]
    pub fn help_text_with_language(
        &self,
        programm_name: &str,
        version: Option<&str>,
        language: Language,
    ) -> String {
        self.erstelle_hilfe_text_mit_sprache(programm_name, version, language)
    }

    /// Erstelle den Hilfe-Text für alle konfigurierten Argumente.
    #[inline(always)]
    pub fn erstelle_hilfe_text(
        &self,
        programm_name: &str,
        version: Option<&str>,
        optionen: &str,
        standard: &str,
        erlaubte_werte: &str,
    ) -> String {
        self.erstelle_hilfe_text_intern(
            None,
            programm_name,
            version,
            optionen,
            standard,
            erlaubte_werte,
        )
    }

    /// Create the help-text for all configured arguments.
    #[inline(always)]
    pub fn create_help_text(
        &self,
        programm_name: &str,
        version: Option<&str>,
        options: &str,
        default: &str,
        allowed_values: &str,
    ) -> String {
        self.erstelle_hilfe_text(programm_name, version, options, default, allowed_values)
    }

    fn erstelle_hilfe_text_intern(
        &self,
        eigene_beschreibung: Option<&Beschreibung<Void>>,
        programm_name: &str,
        version: Option<&str>,
        optionen: &str,
        standard: &str,
        erlaubte_werte: &str,
    ) -> String {
        let current_exe = env::current_exe().ok();
        let exe_name = current_exe
            .as_ref()
            .map(PathBuf::as_path)
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .unwrap_or(programm_name);
        let mut name = programm_name.to_owned();
        if let Some(version) = version {
            name.push(' ');
            name.push_str(version);
        }
        let mut hilfe_text = format!("{}\n\n{} [{}]\n\n{}:\n", name, exe_name, optionen, optionen);
        let eigener_arg_string = eigene_beschreibung.map(|beschreibung| Konfiguration::Flag {
            beschreibung: beschreibung.clone().als_string_beschreibung().0,
            invertiere_präfix: None,
        });
        fn lang_regex(
            lang_namen: &NonEmpty<String>,
            invertiere_präfix_oder_meta_var: Either<&Option<String>, &String>,
        ) -> String {
            let mut lang_regex = "--".to_owned();
            match invertiere_präfix_oder_meta_var {
                Either::Left(invertiere_präfix) => {
                    if let Some(präfix) = invertiere_präfix {
                        lang_regex.push('[');
                        lang_regex.push_str(präfix);
                        lang_regex.push_str("]-");
                    }
                    namen_regex_hinzufügen(&mut lang_regex, &lang_namen.head, &lang_namen.tail);
                },
                Either::Right(meta_var) => {
                    namen_regex_hinzufügen(&mut lang_regex, &lang_namen.head, &lang_namen.tail);
                    lang_regex.push_str("(=| )");
                    lang_regex.push_str(meta_var);
                },
            }
            lang_regex
        }
        let none = None;
        let mut max_lang_regex_breite = 0;
        let mut lang_regex_vec = Vec::new();
        for arg_string in self.konfigurationen().chain(eigener_arg_string.iter()) {
            let (beschreibung, invertiere_präfix_oder_meta_var, mögliche_werte) = match arg_string {
                Konfiguration::Flag { beschreibung, invertiere_präfix } => {
                    (beschreibung, Either::Left(invertiere_präfix), &none)
                },
                Konfiguration::Wert { beschreibung, meta_var, mögliche_werte } => {
                    (beschreibung, Either::Right(meta_var), mögliche_werte)
                },
            };
            let lang_regex = lang_regex(&beschreibung.lang, invertiere_präfix_oder_meta_var);
            let lang_regex_breite = lang_regex.graphemes(true).count();
            max_lang_regex_breite = max_lang_regex_breite.max(lang_regex_breite);
            lang_regex_vec.push((
                lang_regex,
                lang_regex_breite,
                beschreibung,
                invertiere_präfix_oder_meta_var,
                mögliche_werte,
            ))
        }
        fn kurz_regex_hinzufügen(
            max_lang_regex_breite: usize,
            mut name_regex: String,
            lang_regex_breite: usize,
            kurz_namen: &Vec<String>,
            invertiere_präfix_oder_meta_var: Either<&Option<String>, &String>,
        ) -> String {
            if let Some((head, tail)) = kurz_namen.split_first() {
                let einrücken = " ".repeat(max_lang_regex_breite - lang_regex_breite);
                name_regex.push_str(&einrücken);
                name_regex.push_str(" | -");
                namen_regex_hinzufügen(&mut name_regex, head, tail);
                if let Either::Right(meta_var) = invertiere_präfix_oder_meta_var {
                    name_regex.push_str("[=| ]");
                    name_regex.push_str(meta_var);
                }
            }
            name_regex
        }
        let mut max_name_regex_breite = 0;
        let mut name_regex_vec = Vec::new();
        for (
            lang_regex,
            lang_regex_breite,
            beschreibung,
            invertiere_präfix_oder_meta_var,
            mögliche_werte,
        ) in lang_regex_vec
        {
            let name_regex = kurz_regex_hinzufügen(
                max_lang_regex_breite,
                lang_regex,
                lang_regex_breite,
                &beschreibung.kurz,
                invertiere_präfix_oder_meta_var,
            );
            let name_regex_breite = name_regex.graphemes(true).count();
            max_name_regex_breite = max_name_regex_breite.max(name_regex_breite);
            name_regex_vec.push((name_regex, name_regex_breite, beschreibung, mögliche_werte))
        }
        fn hilfe_zeile(
            standard: &str,
            erlaubte_werte: &str,
            max_name_regex_breite: usize,
            hilfe_text: &mut String,
            name_regex: String,
            name_regex_breite: usize,
            beschreibung: &Beschreibung<String>,
            mögliche_werte: &Option<NonEmpty<String>>,
        ) {
            hilfe_text.push_str("  ");
            hilfe_text.push_str(&name_regex);
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
        for (name_regex, name_regex_breite, beschreibung, mögliche_werte) in name_regex_vec {
            hilfe_zeile(
                standard,
                erlaubte_werte,
                max_name_regex_breite,
                &mut hilfe_text,
                name_regex,
                name_regex_breite,
                beschreibung,
                mögliche_werte,
            )
        }
        hilfe_text
    }

    /// Erstelle eine Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei die übergebene Nachricht an.
    pub fn frühes_beenden(
        self,
        beschreibung: Beschreibung<Void>,
        nachricht: String,
    ) -> Argumente<T, E> {
        let Argumente { mut konfigurationen, mut flag_kurzformen, parse } = self;
        let name_kurz = beschreibung.kurz.clone();
        let name_lang = beschreibung.lang.clone();
        let (beschreibung, _standard) = beschreibung.als_string_beschreibung();
        flag_kurzformen.extend(beschreibung.kurz.iter().cloned());
        konfigurationen.push(Konfiguration::Flag { beschreibung, invertiere_präfix: None });
        Argumente {
            konfigurationen,
            flag_kurzformen,
            parse: Box::new(move |args| {
                let name_kurz_existiert = !name_kurz.is_empty();
                let mut nicht_selbst_verwendet = Vec::new();
                let mut nachrichten = Vec::new();
                let mut zeige_nachricht = || nachrichten.push(nachricht.clone());
                for arg in args {
                    if let Some(string) = arg.and_then(OsStr::to_str) {
                        if let Some(lang) = string.strip_prefix("--") {
                            if contains_str!(&name_lang, lang) {
                                zeige_nachricht();
                                nicht_selbst_verwendet.push(None);
                                continue;
                            }
                        } else if name_kurz_existiert {
                            if let Some(kurz) = string.strip_prefix('-') {
                                if kurz
                                    .graphemes(true)
                                    .exactly_one()
                                    .map(|name| contains_str!(&name_kurz, name))
                                    .unwrap_or(false)
                                {
                                    zeige_nachricht();
                                    nicht_selbst_verwendet.push(None);
                                    continue;
                                }
                            }
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
                    _ => {
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
    #[inline(always)]
    pub fn early_exit(self, description: Description<Void>, message: String) -> Arguments<T, E> {
        self.frühes_beenden(description, message)
    }
}
