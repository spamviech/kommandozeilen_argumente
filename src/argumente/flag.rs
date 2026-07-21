//! Flag-Argumente.

use std::{
    borrow::Cow,
    convert::identity,
    ffi::{OsStr, OsString},
    fmt::{self, Debug},
};

use nonempty::NonEmpty;

use crate::{
    argumente::{hilfe::Hilfe, ParseMergedShortFormsResult},
    beschreibung::{ArgumentInput, Beschreibung, Description, Name},
    dyn_to_owned::{Bool, Show},
    ergebnis::{Ergebnis, Fehler, SingeArgResult, ZwischenErgebnis},
    sprache::{Language, Sprache},
    unicode::Vergleich,
};

/// Es handelt sich um ein Flag-Argument.
///
/// ## English
/// It is a flag argument.
#[must_use]
pub struct Flag<'t, T> {
    /// Allgemeine Beschreibung des Arguments.
    ///
    /// ## English
    /// General description of the argument.
    pub beschreibung: Beschreibung<'t, T>,

    /// Präfix invertieren des Flag-Arguments.
    ///
    /// ## English
    /// Prefix to invert the flag argument.
    pub invertiere_präfix: Vergleich<'t>,

    /// Infix zum invertieren des Flag-Arguments.
    ///
    /// ## English
    /// Infix to invert the flag argument.
    pub invertiere_infix: Vergleich<'t>,

    /// Erzeuge einen Wert aus einer [`bool`].
    ///
    /// ## English
    /// Create a value from a [`bool`].
    pub konvertiere: Cow<'t, dyn Bool<'t, T>>,

    /// Anzeige eines Wertes (default value).
    ///
    /// ## English
    /// Display a value (default value).
    pub anzeige: Cow<'t, dyn Show<'t, T>>,
}

impl<T: Debug> Debug for Flag<'_, T> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Flag")
            .field("beschreibung", &self.beschreibung)
            .field("invertiere_präfix", &self.invertiere_präfix)
            .field("invertiere_infix", &self.invertiere_infix)
            .field("konvertiere", &"<closure>")
            .field("anzeige", &"<closure>")
            .finish()
    }
}

impl<'t> Flag<'t, bool> {
    /// Erzeuge ein Flag-Argument, dass mit einem "kein"-Präfix deaktiviert werden kann.
    ///
    /// ## English version
    /// [`new`](Flag::new)
    #[inline]
    pub fn neu(beschreibung: Beschreibung<'t, bool>) -> Self {
        Flag::neu_mit_sprache(beschreibung, Sprache::DEUTSCH)
    }

    /// Erzeuge ein Flag-Argument, dass mit dem konfigurierten Präfix deaktiviert werden kann.
    ///
    /// ## English synonym
    /// [`new_with_language`](Flag::new_with_language)
    #[inline]
    pub fn neu_mit_sprache(beschreibung: Beschreibung<'t, bool>, sprache: Sprache) -> Self {
        Flag {
            beschreibung,
            invertiere_präfix: Vergleich::from(sprache.invertiere_präfix),
            invertiere_infix: Vergleich::from(sprache.invertiere_infix),
            konvertiere: Cow::Borrowed(&identity),
            anzeige: Cow::Borrowed(&<bool as ToString>::to_string),
        }
    }

    /// Create a flag-argument which can be deactivated with a "no" prefix.
    ///
    /// ## Deutsche Version
    /// [`neu`](Flag::neu)
    #[inline]
    pub fn new(description: Description<'t, bool>) -> Self {
        Flag::new_with_language(description, Sprache::ENGLISH)
    }

    /// Create a flag-argument which can be deactivated with the configured prefix.
    ///
    /// ## Deutsches Synonym
    /// [`neu_mit_sprache`](Flag::neu_mit_sprache)
    #[inline]
    pub fn new_with_language(description: Description<'t, bool>, language: Language) -> Self {
        Flag::neu_mit_sprache(description, language)
    }
}

impl<'t, T> Flag<'t, T> {
    /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
    ///
    /// ## English
    /// Create the Message for the syntax of the arguments and the corresponding help text.
    #[inline]
    pub fn erzeuge_hilfe_text(&self, meta_standard: &str) -> Hilfe {
        let Flag { beschreibung, invertiere_präfix, invertiere_infix, konvertiere: _, anzeige } =
            self;
        let Beschreibung { name, hilfe, standard } = beschreibung;
        let Name { lang_präfix, lang, kurz_präfix, kurz } = name;
        let mut syntax = String::new();
        syntax.push_str(lang_präfix.as_str());
        syntax.push('[');
        syntax.push_str(invertiere_präfix.as_str());
        syntax.push_str(invertiere_infix.as_str());
        syntax.push(']');
        let NonEmpty { head, tail } = lang;
        Name::möglichkeiten_als_regex(head, tail.as_slice(), &mut syntax);
        if let Some((kurz_head, kurz_tail)) = kurz.split_first() {
            syntax.push_str(" | ");
            syntax.push_str(kurz_präfix.as_str());
            Name::möglichkeiten_als_regex(kurz_head, kurz_tail, &mut syntax);
        }
        let hilfe = match (hilfe, standard) {
            (None, None) => None,
            (None, Some(standard)) => Some(format!("{meta_standard}: {}", anzeige(standard))),
            (Some(hilfe), None) => Some(String::from(*hilfe)),
            (Some(hilfe), Some(standard)) => {
                let mut hilfe_mit_standard = (*hilfe).to_owned();
                hilfe_mit_standard.push(' ');
                hilfe_mit_standard.push_str(meta_standard);
                hilfe_mit_standard.push_str(": ");
                hilfe_mit_standard.push_str(&anzeige(standard));
                Some(hilfe_mit_standard)
            },
        };
        Hilfe { syntax, hilfe }
    }

    /// Konvertiere den Wert in einen String, unter Zuhilfenahme der jeweiligen `anzeige*`-Funktionen.
    ///
    /// ## English
    /// Convert the value to a string, using the respective `anzeige*`-function.
    #[inline]
    pub fn als_string_flag(&self) -> Flag<'_, String> {
        let Flag { beschreibung, invertiere_präfix, invertiere_infix, konvertiere, anzeige } = self;
        let konvertiere_boxed: Box<dyn '_ + Bool<'_, String>> =
            Box::new(|bool: bool| anzeige(&konvertiere(bool)));
        Flag {
            beschreibung: beschreibung.as_ref().konvertiere(&**anzeige),
            invertiere_präfix: invertiere_präfix.clone(),
            invertiere_infix: invertiere_infix.clone(),
            konvertiere: Cow::Owned(konvertiere_boxed),
            anzeige: Cow::Borrowed(&Clone::clone),
        }
    }
}

impl<T> Flag<'_, T> {
    /// Parse merged short form arguments.
    ///
    /// Rules to allow merging of short names:
    ///
    /// - All short names in the same string share the same (short) prefix.
    /// - Only short names consisting of a single [grapheme](https://docs.rs/unicode-segmentation/1.8.0/unicode_segmentation/trait.UnicodeSegmentation.html#tymethod.graphemes) participate.
    /// - At most one value argument per block.
    ///   It must be the last argument name in the string, optionally followed by \[a value-infix and\] the value sub-string.
    /// - Merging of short names must be allowed for this particular argument.
    #[inline]
    pub fn parse_merged_short_forms<F>(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'_, T, F> {
        let Flag { beschreibung, invertiere_präfix, invertiere_infix, konvertiere, anzeige } = self;
        todo!();
    }
}
