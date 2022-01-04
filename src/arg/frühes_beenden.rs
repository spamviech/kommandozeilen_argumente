//! Flag-Argumente, die zu frühen Beenden führen.

use std::{
    env,
    ffi::OsStr,
    iter,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;
use void::Void;

use crate::{
    arg::{Arg, ArgString},
    beschreibung::Beschreibung,
    ergebnis::ParseErgebnis,
};

impl<T: 'static, E: 'static> Arg<T, E> {
    pub fn version_deutsch(self, programm_name: &str, version: &str) -> Arg<T, E> {
        let beschreibung = Beschreibung {
            lang: "version".to_owned(),
            kurz: Some("v".to_owned()),
            hilfe: Some("Zeigt die aktuelle Version an.".to_owned()),
            standard: None,
        };
        self.zeige_version(beschreibung, programm_name, version)
    }

    pub fn version_english(self, program_name: &str, version: &str) -> Arg<T, E> {
        let beschreibung = Beschreibung {
            lang: "version".to_owned(),
            kurz: Some("v".to_owned()),
            hilfe: Some("Show the current version.".to_owned()),
            standard: None,
        };
        self.zeige_version(beschreibung, program_name, version)
    }

    pub fn zeige_version(
        self,
        beschreibung: Beschreibung<Void>,
        programm_name: &str,
        version: &str,
    ) -> Arg<T, E> {
        self.frühes_beenden(beschreibung, format!("{} {}", programm_name, version))
    }

    pub fn hilfe(
        self,
        programm_name: &str,
        version: Option<&str>,
        name_regex_breite: usize,
    ) -> Arg<T, E> {
        let beschreibung = Beschreibung {
            lang: "hilfe".to_owned(),
            kurz: Some("h".to_owned()),
            hilfe: Some("Zeigt diesen Text an.".to_owned()),
            standard: None,
        };
        self.erstelle_hilfe(
            beschreibung,
            programm_name,
            version,
            "OPTIONEN",
            "standard",
            "Erlaubte Werte",
            name_regex_breite,
        )
    }

    pub fn hilfe_und_version(
        self,
        programm_name: &str,
        version: &str,
        name_regex_breite: usize,
    ) -> Arg<T, E> {
        self.version_deutsch(programm_name, version).hilfe(
            programm_name,
            Some(version),
            name_regex_breite,
        )
    }

    pub fn help(
        self,
        program_name: &str,
        version: Option<&str>,
        name_regex_width: usize,
    ) -> Arg<T, E> {
        let beschreibung = Beschreibung {
            lang: "help".to_owned(),
            kurz: Some("h".to_owned()),
            hilfe: Some("Show this text.".to_owned()),
            standard: None,
        };
        self.erstelle_hilfe(
            beschreibung,
            program_name,
            version,
            "OPTIONS",
            "default",
            "Possible values",
            name_regex_width,
        )
    }

    pub fn help_and_version(
        self,
        program_name: &str,
        version: &str,
        name_regex_breite: usize,
    ) -> Arg<T, E> {
        self.version_english(program_name, version).help(
            program_name,
            Some(version),
            name_regex_breite,
        )
    }

    pub fn erstelle_hilfe(
        self,
        eigene_beschreibung: Beschreibung<Void>,
        programm_name: &str,
        version: Option<&str>,
        optionen: &str,
        standard: &str,
        erlaubte_werte: &str,
        name_regex_breite: usize,
    ) -> Arg<T, E> {
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
        let eigener_arg_string = ArgString::Flag {
            beschreibung: eigene_beschreibung.clone().als_string_beschreibung().0,
            invertiere_prefix: None,
        };
        fn hilfe_zeile(
            standard: &str,
            erlaubte_werte: &str,
            name_regex_breite: usize,
            hilfe_text: &mut String,
            name_regex: &String,
            beschreibung: &Beschreibung<String>,
            mögliche_werte: &Option<NonEmpty<String>>,
        ) {
            hilfe_text.push_str("  ");
            hilfe_text.push_str(name_regex);
            let bisherige_breite = 2 + name_regex.graphemes(true).count();
            let einrücken = " ".repeat(name_regex_breite.saturating_sub(bisherige_breite));
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
        for beschreibung in self.beschreibungen.iter().chain(iter::once(&eigener_arg_string)) {
            match beschreibung {
                ArgString::Flag { beschreibung, invertiere_prefix } => {
                    let mut name_regex = "--".to_owned();
                    if let Some(prefix) = invertiere_prefix {
                        name_regex.push('[');
                        name_regex.push_str(prefix);
                        name_regex.push_str("]-");
                    }
                    name_regex.push_str(&beschreibung.lang);
                    if let Some(kurz) = &beschreibung.kurz {
                        name_regex.push_str(" | -");
                        name_regex.push_str(kurz);
                    }
                    hilfe_zeile(
                        standard,
                        erlaubte_werte,
                        name_regex_breite,
                        &mut hilfe_text,
                        &name_regex,
                        beschreibung,
                        &None,
                    );
                }
                ArgString::Wert { beschreibung, meta_var, mögliche_werte } => {
                    let mut name_regex = "--".to_owned();
                    name_regex.push_str(&beschreibung.lang);
                    name_regex.push_str("(=| )");
                    name_regex.push_str(meta_var);
                    if let Some(kurz) = &beschreibung.kurz {
                        name_regex.push_str(" | -");
                        name_regex.push_str(kurz);
                        name_regex.push_str("[=| ]");
                        name_regex.push_str(meta_var);
                    }
                    hilfe_zeile(
                        standard,
                        erlaubte_werte,
                        name_regex_breite,
                        &mut hilfe_text,
                        &name_regex,
                        beschreibung,
                        mögliche_werte,
                    );
                }
            }
        }
        self.frühes_beenden(eigene_beschreibung, hilfe_text)
    }

    pub fn frühes_beenden(self, beschreibung: Beschreibung<Void>, nachricht: String) -> Arg<T, E> {
        let Arg { mut beschreibungen, mut flag_kurzformen, parse } = self;
        let name_kurz = beschreibung.kurz.clone();
        let name_lang = beschreibung.lang.clone();
        let (beschreibung, _standard) = beschreibung.als_string_beschreibung();
        if let Some(kurz) = &beschreibung.kurz {
            flag_kurzformen.push(kurz.clone())
        }
        beschreibungen.push(ArgString::Flag { beschreibung, invertiere_prefix: None });
        Arg {
            beschreibungen,
            flag_kurzformen,
            parse: Box::new(move |args| {
                let name_kurz_str = name_kurz.as_ref().map(String::as_str);
                let name_kurz_existiert = name_kurz_str.is_some();
                let mut nicht_verwendet = Vec::new();
                let mut nachrichten = Vec::new();
                let mut zeige_nachricht = || nachrichten.push(nachricht.clone());
                for arg in args {
                    if let Some(string) = arg.and_then(OsStr::to_str) {
                        if let Some(lang) = string.strip_prefix("--") {
                            if lang == name_lang {
                                zeige_nachricht();
                                nicht_verwendet.push(None);
                                continue;
                            }
                        } else if name_kurz_existiert {
                            if let Some(kurz) = string.strip_prefix('-') {
                                if kurz.graphemes(true).exactly_one().ok() == name_kurz_str {
                                    zeige_nachricht();
                                    nicht_verwendet.push(None);
                                    continue;
                                }
                            }
                        }
                    }
                    nicht_verwendet.push(arg);
                }
                if let Some(frühes_beenden) = NonEmpty::from_vec(nachrichten) {
                    (ParseErgebnis::FrühesBeenden(frühes_beenden), nicht_verwendet)
                } else {
                    parse(nicht_verwendet)
                }
            }),
        }
    }
}
