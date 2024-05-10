//! Wert-Argumente.

use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
    str::FromStr,
};

use nonempty::NonEmpty;

use crate::{
    argumente::hilfe::Hilfe,
    beschreibung::{Beschreibung, Description, Name},
    dyn_to_owned::{Anzeige, Parse},
    ergebnis::{Ergebnis, Fehler, ParseFehler},
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
    fn varianten() -> Vec<Self>;

    /// All variants of the type.
    ///
    /// ## Deutsches Synonym
    /// [`varianten`](EnumArgument::varianten)
    #[inline]
    #[must_use]
    fn variants() -> Vec<Self> {
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
    fn parse_enum(arg: OsString) -> Result<Self, ParseFehler<String>>;
}

/// Es handelt sich um ein Wert-Argument.
///
/// ## English
/// It is a value argument.
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
    pub anzeige: Cow<'t, dyn Anzeige<'t, T>>,
}

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
            .finish()
    }
}

impl<'t, T: Display + FromStr> Wert<'t, T, <T as FromStr>::Err> {
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
        }
    }

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

/// Hilfsfunktion für [`Argumente::parse`]
fn zeige_elemente<'t, T: 't>(
    string: &mut String,
    #[allow(clippy::ptr_arg)] anzeige: &Cow<'_, dyn Anzeige<'_, T>>,
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
    /// Parse die übergebenen Argumente und erzeuge den zugehörigen Wert.
    ///
    /// ## English
    /// Parse the given arguments and return the corresponding value.
    #[inline]
    pub fn parse<I: Iterator<Item = Option<OsString>>>(
        self,
        args: I,
    ) -> (Ergebnis<'t, T, F>, Vec<Option<OsString>>) {
        let Wert { beschreibung, wert_infix, meta_var, mögliche_werte: _, parse, anzeige: _ } =
            self;
        let Beschreibung { name, hilfe: _, standard } = beschreibung;
        let mut nicht_verwendet = Vec::new();
        let mut iter = args.into_iter();
        let mut name_ohne_wert = None;
        // Wird nur mit den gleich benannten Variablen aufgerufen.
        #[allow(clippy::shadow_unrelated)]
        let parse_wert = |arg: &OsStr,
                          mut nicht_verwendet: Vec<_>,
                          name: Name<'t>,
                          wert_infix: Vergleich<'t>,
                          iter: I|
         -> (Ergebnis<'t, T, F>, Vec<Option<OsString>>) {
            nicht_verwendet.push(None);
            nicht_verwendet.extend(iter);
            let ergebnis = match parse(arg) {
                Ok(wert) => Ergebnis::Wert(wert),
                Err(fehler) => Ergebnis::Fehler(NonEmpty::singleton(Fehler::Fehler {
                    name,
                    wert_infix: wert_infix.string,
                    meta_var,
                    fehler,
                })),
            };
            (ergebnis, nicht_verwendet)
        };
        while let Some(arg_opt) = iter.next() {
            if let Some(arg) = &arg_opt {
                if let Some(_name_arg) = name_ohne_wert.take() {
                    return parse_wert(arg, nicht_verwendet, name, wert_infix, iter);
                } else if let Some(wert_opt) = name.parse_mit_wert(&wert_infix, arg) {
                    if let Some(wert_str) = wert_opt {
                        return parse_wert(&wert_str, nicht_verwendet, name, wert_infix, iter);
                    }
                    name_ohne_wert = Some(arg_opt);
                } else {
                    nicht_verwendet.push(arg_opt);
                }
            } else {
                if let Some(einzelner_name) = name_ohne_wert.take() {
                    nicht_verwendet.push(einzelner_name);
                }
                nicht_verwendet.push(arg_opt);
            }
        }
        let ergebnis = if let Some(wert) = standard {
            Ergebnis::Wert(wert)
        } else {
            Ergebnis::Fehler(NonEmpty::singleton(Fehler::FehlenderWert {
                name,
                wert_infix: wert_infix.string,
                meta_var,
            }))
        };
        (ergebnis, nicht_verwendet)
    }

    /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
    ///
    /// ## English
    /// Create the Message for the syntax of the arguments and the corresponding help text.
    #[inline]
    pub fn erzeuge_hilfe_text(&self, meta_standard: &str, meta_erlaubte_werte: &str) -> Hilfe<'_> {
        let Wert { beschreibung, wert_infix, meta_var, mögliche_werte, parse: _, anzeige } = self;
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
        let hilfe: Option<Cow<'_, str>> = match (hilfe, standard, mögliche_werte) {
            (None, None, None) => None,
            (None, None, Some(mögliche_werte)) => {
                let mut string = format!("[{meta_erlaubte_werte}: ");
                zeige_elemente(&mut string, &self.anzeige, mögliche_werte);
                string.push(']');
                Some(Cow::Owned(string))
            },
            (None, Some(standard), None) => {
                Some(Cow::Owned(format!("[{meta_standard}: {}]", anzeige(standard))))
            },
            (None, Some(standard), Some(mögliche_werte)) => {
                let mut string =
                    format!("[{meta_standard}: {}, {meta_erlaubte_werte}: ", anzeige(standard));
                zeige_elemente(&mut string, &self.anzeige, mögliche_werte);
                string.push(']');
                Some(Cow::Owned(string))
            },
            (Some(hilfe), None, None) => Some(Cow::Borrowed(hilfe)),
            (Some(hilfe), None, Some(mögliche_werte)) => {
                let mut string = format!("{hilfe} [{meta_erlaubte_werte}: ");
                zeige_elemente(&mut string, &self.anzeige, mögliche_werte);
                string.push(']');
                Some(Cow::Owned(string))
            },
            (Some(hilfe), Some(standard), None) => {
                Some(Cow::Owned(format!("{hilfe} [{meta_standard}: {}]", anzeige(standard))))
            },
            (Some(hilfe), Some(standard), Some(mögliche_werte)) => {
                let mut string = format!(
                    "{hilfe} [{meta_standard}: {}, {meta_erlaubte_werte}: ",
                    anzeige(standard)
                );
                zeige_elemente(&mut string, &self.anzeige, mögliche_werte);
                string.push(']');
                Some(Cow::Owned(string))
            },
        };
        Hilfe { syntax, hilfe }
    }
}
