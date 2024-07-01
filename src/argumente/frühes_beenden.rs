//! Flag-Argumente, die zu frühen Beenden führen.

use std::borrow::Cow;

use nonempty::NonEmpty;
use void::Void;

use crate::{
    argumente::hilfe::Hilfe,
    beschreibung::{ArgumentInput, Beschreibung, Description, Name},
    ergebnis::Ergebnis,
};

/// Es handelt sich um ein Flag-Argument, das zu frühem beenden führt.
///
/// ## English synonym
/// [`EarlyExit`]
#[derive(Debug, Clone)]
#[must_use]
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

/// It is a flag argument, causing an early exit.
///
/// ## Deutsches Synonym
/// [`FrühesBeenden`]
pub type EarlyExit<'t> = FrühesBeenden<'t>;

impl<'t> FrühesBeenden<'t> {
    /// Erstelle eine Flag, die zu vorzeitigem Beenden führt.
    /// Zeige dabei die übergebene Nachricht an.
    ///
    /// ## English synonym
    /// [`new`](FrühesBeenden::new)
    #[inline]
    pub fn neu(beschreibung: Beschreibung<'t, Void>, nachricht: impl Into<Cow<'t, str>>) -> Self {
        FrühesBeenden { beschreibung, nachricht: nachricht.into() }
    }

    /// Create a flag which causes an early exit and shows the given message.
    ///
    /// ## Deutsches Synonym
    /// [`neu`](FrühesBeenden::neu)
    #[inline]
    pub fn new(description: Description<'t, Void>, message: impl Into<Cow<'t, str>>) -> Self {
        FrühesBeenden::neu(description, message)
    }

    /// Parse die übergebenen Argumente und erzeuge den zugehörigen Wert.
    ///
    /// ## English
    /// Parse the given arguments and return the corresponding value.
    #[inline]
    pub fn parse<F>(
        self,
        args: impl Iterator<Item = Option<ArgumentInput>>,
    ) -> (Ergebnis<'t, (), F>, Vec<Option<ArgumentInput>>) {
        let FrühesBeenden { beschreibung, nachricht } = self;
        let Beschreibung { name, hilfe: _, standard } = beschreibung;
        let mut nicht_verwendet = Vec::new();
        let mut iter = args.into_iter();
        while let Some(arg_opt) = iter.next() {
            if let Some(arg) = &arg_opt {
                if let Some(angepasstes_arg) = name.parse_frühes_beenden(arg) {
                    nicht_verwendet.push(angepasstes_arg);
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
    pub fn erzeuge_hilfe_text(&self) -> Hilfe {
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
        let hilfe = hilfe.map(String::from);
        Hilfe { syntax, hilfe }
    }
}
