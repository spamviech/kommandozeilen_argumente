//! Flag arguments that cause early exits.

use std::{borrow::Cow, ffi::OsString};

use nonempty::NonEmpty;
use void::Void;

use crate::{
    arguments::{
        ParseMergedShortFormsResult,
        help::{Help, Hilfe},
    },
    description::{Beschreibung, Description, Name},
};

/// A flag argument that causes an early exit.
///
/// ## Deutsch
/// Ein Flag-Argument, das zu einem frühen Beenden führt.
#[derive(Debug, Clone)]
#[must_use]
pub struct EarlyExit<'t> {
    /// General description of the argument.
    pub description: Description<'t, Void>,
    /// Message displayed when the argument is parsed.
    pub message: Cow<'t, str>,
}

/// German mirror of [`EarlyExit`].
#[derive(Debug, Clone)]
#[must_use]
pub struct FrühesBeenden<'t> {
    /// Allgemeine Beschreibung des Arguments.
    pub beschreibung: Beschreibung<'t, Void>,
    /// Die beim Parsen des Arguments angezeigte Nachricht.
    pub nachricht: Cow<'t, str>,
}

impl<'t> From<EarlyExit<'t>> for FrühesBeenden<'t> {
    #[inline]
    fn from(EarlyExit { description, message }: EarlyExit<'t>) -> Self {
        Self { beschreibung: description.into(), nachricht: message }
    }
}

impl<'t> From<FrühesBeenden<'t>> for EarlyExit<'t> {
    #[inline]
    fn from(FrühesBeenden { beschreibung, nachricht }: FrühesBeenden<'t>) -> Self {
        Self { description: beschreibung.into(), message: nachricht }
    }
}

impl<'t> EarlyExit<'t> {
    /// Creates a flag that causes an early exit and displays `message`.
    #[inline]
    pub fn new(description: Description<'t, Void>, message: impl Into<Cow<'t, str>>) -> Self {
        Self { description, message: message.into() }
    }

    /// Creates this argument's syntax and help text.
    #[inline]
    pub fn create_help_text(&self) -> Help {
        create_help_text(&self.description.name, self.description.help, self.description.default)
    }
}

impl<'t> FrühesBeenden<'t> {
    /// Erstellt eine Flag, die ein frühes Beenden auslöst und `nachricht` anzeigt.
    #[inline]
    pub fn neu(beschreibung: Beschreibung<'t, Void>, nachricht: impl Into<Cow<'t, str>>) -> Self {
        EarlyExit::new(beschreibung.into(), nachricht).into()
    }

    /// Erzeugt Syntax und Hilfetext für dieses Argument.
    #[inline]
    pub fn erzeuge_hilfe_text(&self) -> Hilfe {
        create_help_text(
            &self.beschreibung.name,
            self.beschreibung.hilfe,
            self.beschreibung.standard,
        )
        .into()
    }
}

fn create_help_text(name: &Name<'_>, help: Option<&str>, default: Option<Void>) -> Help {
    let Name { long_prefix, long, short_prefix, short } = name;
    let mut syntax = String::new();
    syntax.push_str(long_prefix.as_str());
    let NonEmpty { head, tail } = long;
    Name::alternatives_as_regex(head, tail.as_slice(), &mut syntax);
    if let Some((short_head, short_tail)) = short.split_first() {
        syntax.push_str(" | ");
        syntax.push_str(short_prefix.as_str());
        Name::alternatives_as_regex(short_head, short_tail, &mut syntax);
    }
    if let Some(value) = default {
        void::unreachable(value);
    }
    Help { syntax, help: help.map(String::from) }
}

impl EarlyExit<'_> {
    /// Parses merged short-form arguments.
    #[inline]
    pub fn parse_merged_short_forms<T, F>(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'_, T, F> {
        let _ = (self, args);
        todo!()
    }
}

impl FrühesBeenden<'_> {
    /// Parst zusammengefasste kurze Argumentformen.
    #[inline]
    pub fn parse_merged_short_forms<T, F>(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'_, T, F> {
        let _ = (self, args);
        todo!()
    }
}
