//! Flag-Argumente, die zu frühen Beenden führen.

use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
};

use nonempty::NonEmpty;
use void::Void;

use crate::{
    argumente::{hilfe::Hilfe, ParseMergedShortFormsResult},
    beschreibung::{ArgumentInput, Beschreibung, Description, Name},
    ergebnis::{Ergebnis, SingeArgResult, ZwischenErgebnis},
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

impl FrühesBeenden<'_> {
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
    ) -> ParseMergedShortFormsResult<'_> {
        let FrühesBeenden { beschreibung, nachricht } = self;
        todo!();
    }
}
