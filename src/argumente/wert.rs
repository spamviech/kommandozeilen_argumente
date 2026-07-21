//! Wert-Argumente.

use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
    str::FromStr,
};

use either::Either;
use nonempty::NonEmpty;

use crate::{
    argumente::{hilfe::Hilfe, ParseMergedShortFormsResult},
    beschreibung::{AdjustedMergedShortNames, ArgumentInput, Beschreibung, Description, Name},
    dyn_to_owned::{Parse, Show},
    ergebnis::{
        Ergebnis, Fehler, KommentierterParseFehler, ParseFehler, SingeArgResult, ZwischenErgebnis,
    },
    sprache::{Language, Sprache},
    unicode::Vergleich,
};

#[cfg(any(feature = "derive", all(doc, not(doctest))))]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use kommandozeilen_argumente_derive::EnumArgument;

/// Trait für Typen mit einer festen Anzahl an Werten und Methode zum Parsen.
/// Gedacht für Summen-Typen ohne extra Daten (nur Unit-Varianten).
///
/// Mit aktiviertem `derive`-Feature kann die Implementierung
/// [`automatisch erzeugt werden`](derive@EnumArgument).
///
/// ## English
/// Trait for types with a fixed number of values and a parse method.
/// Intended for sum-types without extra data (only unit variants).
///
/// With activated `derive`-feature, the implementation can be
/// [`created automatically`](derive@EnumArgument).
pub trait EnumArgument: Sized {
    /// Alle Varianten des Typs.
    ///
    /// ## English synonym
    /// [`variants`](EnumArgument::variants)
    #[must_use]
    fn varianten() -> Option<NonEmpty<Self>>;

    /// All variants of the type.
    ///
    /// ## Deutsches Synonym
    /// [`varianten`](EnumArgument::varianten)
    #[inline]
    #[must_use]
    fn variants() -> Option<NonEmpty<Self>> {
        Self::varianten()
    }

    /// Versuche einen Wert ausgehend vom übergebenen [`OsString`] zu erzeugen.
    ///
    /// ### Fehler
    ///
    /// Parsen wegen dem zurückgegebenen [`ParseFehler`] fehlgeschlagen.
    ///
    /// ## English
    /// Try to parse a value from the given [`OsString`].
    ///
    /// ### Errors
    ///
    /// Parsing failed with the given [`ParseError`].
    fn parse_enum(arg: &OsStr) -> Result<Self, ParseFehler<String>>;
}

/// Es handelt sich um ein Wert-Argument.
///
/// ## English synonym
/// [`Value`]
#[must_use]
pub struct Wert<'t, T, Fehler> {
    /// Allgemeine Beschreibung des Arguments.
    ///
    /// ## English
    /// General description of the argument.
    pub beschreibung: Beschreibung<'t, T>,

    /// Infix um einen Wert im selben Argument wie den Namen anzugeben.
    ///
    /// ## English
    /// Infix to give a value in the same argument as the name.
    pub wert_infix: Vergleich<'t>,

    /// Meta-Variable im Hilfe-Text.
    ///
    /// ## English
    /// Meta-variable used in the help-text.
    pub meta_var: &'t str,

    /// Erlaubten Werte zur Anzeige im Hilfe-Text.
    ///
    /// ## English
    /// Allowed values, used in the help text.
    pub mögliche_werte: Option<NonEmpty<T>>,

    /// Parse einen Wert aus einem [`OsString`].
    ///
    /// ## English
    /// Parse a value from an [`OsString`].
    pub parse: Cow<'t, dyn Parse<'t, T, Fehler>>,

    /// Anzeige eines Wertes (standard/mögliche Werte).
    ///
    /// ## English
    /// Display a value (default/possible values).
    pub anzeige: Cow<'t, dyn Show<'t, T>>,

    /// Anzeige eines Fehlers.
    ///
    /// ## English
    /// Display an error.
    pub anzeige_fehler: Cow<'t, dyn Show<'t, Fehler>>,
}

/// It is a value argument.
///
/// ## Deutsche Synonym
/// [`Wert`]
pub type Value<'t, T, Fehler> = Wert<'t, T, Fehler>;

impl<T: Debug, Fehler> Debug for Wert<'_, T, Fehler> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Wert")
            .field("beschreibung", &self.beschreibung)
            .field("wert_infix", &self.wert_infix)
            .field("meta_var", &self.meta_var)
            .field("mögliche_werte", &self.mögliche_werte)
            .field("parse", &"<closure>")
            .field("anzeige", &"<closure>")
            .field("anzeige_fehler", &"<closure>")
            .finish()
    }
}

impl<'t, T: Display + FromStr> Wert<'t, T, <T as FromStr>::Err>
where
    <T as FromStr>::Err: Display,
{
    /// Erzeuge ein Wert-Argument, ausgehend von der [`FromStr`]-Implementierung.
    ///
    /// ## English synonym
    /// [`new`](Wert::new)
    #[inline]
    pub fn neu(beschreibung: Beschreibung<'t, T>, mögliche_werte: Option<NonEmpty<T>>) -> Self {
        Wert::neu_mit_sprache(beschreibung, mögliche_werte, Sprache::DEUTSCH)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der [`FromStr`]-Implementierung.
    ///
    /// ## English synonym
    /// [`new_with_language`](Wert::new_with_language)
    #[inline]
    pub fn neu_mit_sprache(
        beschreibung: Beschreibung<'t, T>,
        mögliche_werte: Option<NonEmpty<T>>,
        sprache: Sprache,
    ) -> Self {
        Wert {
            beschreibung,
            wert_infix: Vergleich::from(sprache.wert_infix),
            meta_var: sprache.meta_var,
            mögliche_werte,
            parse: Cow::Borrowed(&|os_str: &OsStr| {
                if let Some(string) = os_str.to_str() {
                    string.parse().map_err(ParseFehler::ParseFehler)
                } else {
                    Err(ParseFehler::InvaliderString(OsString::from(os_str)))
                }
            }),
            anzeige: Cow::Borrowed(&<T as ToString>::to_string),
            anzeige_fehler: Cow::Borrowed(&<<T as FromStr>::Err as ToString>::to_string),
        }
    }
}

impl<'t, T: Display + FromStr> Value<'t, T, <T as FromStr>::Err>
where
    <T as FromStr>::Err: Display,
{
    /// Create a value-argument, based on the [`FromStr`]-implementation.
    ///
    /// ## Deutsches Synonym
    /// [`neu`](Wert::neu)
    #[inline]
    pub fn new(description: Description<'t, T>, possible_values: Option<NonEmpty<T>>) -> Self {
        Wert::new_with_language(description, possible_values, Language::ENGLISH)
    }

    /// Create a value-argument, based on the [`FromStr`]-implementation.
    ///
    /// ## Deutsches Synonym
    /// [`neu_mit_sprache`](Wert::neu_mit_sprache)
    #[inline]
    pub fn new_with_language(
        description: Description<'t, T>,
        possible_values: Option<NonEmpty<T>>,
        language: Language,
    ) -> Self {
        Wert::neu_mit_sprache(description, possible_values, language)
    }
}

impl<'t, T: Display + EnumArgument> Wert<'t, T, String> {
    /// Erzeuge ein Wert-Argument, ausgehend von der [`FromStr`]-Implementierung.
    ///
    /// ## English synonym
    /// [`new_enum`](Wert::new_enum)
    #[inline]
    pub fn neu_enum(beschreibung: Beschreibung<'t, T>) -> Self {
        Wert::neu_enum_mit_sprache(beschreibung, Sprache::DEUTSCH)
    }

    /// Erzeuge ein Wert-Argument, ausgehend von der [`FromStr`]-Implementierung.
    ///
    /// ## English synonym
    /// [`new_enum_with_language`](Wert::new_enum_with_language)
    #[inline]
    pub fn neu_enum_mit_sprache(beschreibung: Beschreibung<'t, T>, sprache: Sprache) -> Self {
        Wert {
            beschreibung,
            wert_infix: Vergleich::from(sprache.wert_infix),
            meta_var: sprache.meta_var,
            mögliche_werte: EnumArgument::varianten(),
            parse: Cow::Borrowed(&EnumArgument::parse_enum),
            anzeige: Cow::Borrowed(&<T as ToString>::to_string),
            anzeige_fehler: Cow::Borrowed(&Clone::clone),
        }
    }
}

impl<'t, T: Display + EnumArgument> Value<'t, T, String> {
    /// Create a value-argument, based on the [`EnumArgument`]-implementation.
    ///
    /// ## Deutsches Synonym
    /// [`neu_enum`](Wert::neu_enum)
    #[inline]
    pub fn new_enum(description: Description<'t, T>) -> Self {
        Wert::new_enum_with_language(description, Language::ENGLISH)
    }

    /// Create a value-argument, based on the [`EnumArgument`]-implementation.
    ///
    /// ## Deutsches Synonym
    /// [`neu_enum_mit_sprache`](Wert::neu_enum_mit_sprache)
    #[inline]
    pub fn new_enum_with_language(description: Description<'t, T>, language: Language) -> Self {
        Wert::neu_enum_mit_sprache(description, language)
    }
}

/// Hilfsfunktion für [`Argumente::parse`]
fn zeige_elemente<'t, T: 't>(
    string: &mut String,
    #[allow(clippy::ptr_arg)] anzeige: &Cow<'_, dyn Show<'_, T>>,
    elemente: impl IntoIterator<Item = &'t T>,
) {
    let mut erstes = true;
    for element in elemente {
        if erstes {
            erstes = false;
        } else {
            string.push_str(", ");
        }
        string.push_str(&anzeige(element));
    }
}

impl<'t, T, F> Wert<'t, T, F> {
    /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
    ///
    /// ## English
    /// Create the Message for the syntax of the arguments and the corresponding help text.
    #[inline]
    pub fn erzeuge_hilfe_text(&self, meta_standard: &str, meta_erlaubte_werte: &str) -> Hilfe {
        let Wert {
            beschreibung,
            wert_infix,
            meta_var,
            mögliche_werte,
            parse: _,
            anzeige,
            anzeige_fehler: _,
        } = self;
        let Beschreibung { name, hilfe, standard } = beschreibung;
        let Name { lang_präfix, lang, kurz_präfix, kurz } = name;
        let mut syntax = String::new();
        syntax.push_str(lang_präfix.as_str());
        let NonEmpty { head, tail } = lang;
        Name::möglichkeiten_als_regex(head, tail.as_slice(), &mut syntax);
        syntax.push_str("( |");
        syntax.push_str(wert_infix.as_str());
        syntax.push(')');
        syntax.push_str(meta_var);
        if let Some((kurz_head, kurz_tail)) = kurz.split_first() {
            syntax.push_str(" | ");
            syntax.push_str(kurz_präfix.as_str());
            Name::möglichkeiten_als_regex(kurz_head, kurz_tail, &mut syntax);
            syntax.push_str("[ |");
            syntax.push_str(wert_infix.as_str());
            syntax.push(']');
            syntax.push_str(meta_var);
        }
        // TODO a lot of code duplication...
        let hilfe = match (hilfe, standard, mögliche_werte) {
            (None, None, None) => None,
            (None, None, Some(mögliche_werte)) => {
                let mut string = format!("[{meta_erlaubte_werte}: ");
                zeige_elemente(&mut string, &self.anzeige, mögliche_werte);
                string.push(']');
                Some(string)
            },
            (None, Some(standard), None) => {
                Some(format!("[{meta_standard}: {}]", anzeige(standard)))
            },
            (None, Some(standard), Some(mögliche_werte)) => {
                let mut string =
                    format!("[{meta_standard}: {}, {meta_erlaubte_werte}: ", anzeige(standard));
                zeige_elemente(&mut string, &self.anzeige, mögliche_werte);
                string.push(']');
                Some(string)
            },
            (Some(hilfe), None, None) => Some(String::from(*hilfe)),
            (Some(hilfe), None, Some(mögliche_werte)) => {
                let mut string = format!("{hilfe} [{meta_erlaubte_werte}: ");
                zeige_elemente(&mut string, &self.anzeige, mögliche_werte);
                string.push(']');
                Some(string)
            },
            (Some(hilfe), Some(standard), None) => {
                Some(format!("{hilfe} [{meta_standard}: {}]", anzeige(standard)))
            },
            (Some(hilfe), Some(standard), Some(mögliche_werte)) => {
                let mut string = format!(
                    "{hilfe} [{meta_standard}: {}, {meta_erlaubte_werte}: ",
                    anzeige(standard)
                );
                zeige_elemente(&mut string, &self.anzeige, mögliche_werte);
                string.push(']');
                Some(string)
            },
        };
        Hilfe { syntax, hilfe }
    }

    /// Konvertiere den Wert und Fehler der parse-Funktion in einen String,
    /// unter Zuhilfenahme der jeweiligen `anzeige*`-Funktionen.
    ///
    /// ## English
    /// Convert the value and error of the parse-function to a string, using the respective `anzeige*`-function.
    #[inline]
    pub fn als_string_wert(&self) -> Wert<'_, String, String> {
        let Wert {
            beschreibung,
            wert_infix,
            meta_var,
            mögliche_werte,
            parse,
            anzeige,
            anzeige_fehler,
        } = self;
        let parse_boxed: Box<dyn '_ + Parse<'_, String, String>> =
            Box::new(|os_str: &OsStr| match parse(os_str) {
                Ok(wert) => Ok(anzeige(&wert)),
                Err(parse_fehler) => {
                    Err(parse_fehler.konvertiere(|fehler| anzeige_fehler(&fehler)))
                },
            });
        Wert {
            beschreibung: beschreibung.as_ref().konvertiere(&**anzeige),
            wert_infix: wert_infix.clone(),
            meta_var,
            mögliche_werte: NonEmpty::from_vec(
                mögliche_werte.as_ref().into_iter().flatten().map(&**anzeige).collect(),
            ),
            parse: Cow::Owned(parse_boxed),
            anzeige: Cow::Borrowed(&Clone::clone),
            anzeige_fehler: Cow::Borrowed(&Clone::clone),
        }
    }
}

impl<T, F> Wert<'_, T, F> {
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
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'_, T, F> {
        let Wert {
            beschreibung,
            wert_infix,
            meta_var,
            mögliche_werte,
            parse,
            anzeige,
            anzeige_fehler,
        } = self;
        todo!();
    }
}
