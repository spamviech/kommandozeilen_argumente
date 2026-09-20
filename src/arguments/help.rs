//! Types and traits for rendering argument help text.

use nonempty::NonEmpty;

use crate::argumente::einzelargument::EinzelArgument;

/// Text representation of an argument in help output.
///
/// ## Deutsch
/// Darstellung eines Arguments im Hilfe-Text.
#[derive(Debug, Clone)]
pub struct Help {
    /// Syntax for enabling/disabling the flag or setting the argument value.
    pub syntax: String,
    /// Help text for the argument.
    pub help: Option<String>,
}

/// Deutsche Spiegelung von [`Help`].
#[derive(Debug, Clone)]
pub struct Hilfe {
    /// Darstellen der Syntax zum (de)aktivieren der Flag/setzen des Argument-Wertes.
    pub syntax: String,
    /// Hilfe-Text für das Argument.
    pub hilfe: Option<String>,
}

impl From<Help> for Hilfe {
    #[inline]
    fn from(Help { syntax, help }: Help) -> Self {
        Self { syntax, hilfe: help }
    }
}

impl From<Hilfe> for Help {
    #[inline]
    fn from(Hilfe { syntax, hilfe }: Hilfe) -> Self {
        Self { syntax, help: hilfe }
    }
}

/// A help-text entry for an argument or several alternatives.
///
/// ## Deutsch
/// Darstellung eines Arguments oder mehrerer Alternativen im Hilfe-Text.
#[derive(Debug, Clone)]
pub enum Alternatives {
    /// A single argument's help text.
    Single(Help),
    /// Several alternative arguments.
    Alternatives(Box<NonEmpty<Alternatives>>),
    /// A fixed value without an associated argument, omitted from help output.
    Empty,
}

/// Deutsche Spiegelung von [`Alternatives`].
#[derive(Debug, Clone)]
pub enum Alternativen {
    /// Darstellung eines einzelnen Arguments im Hilfe-Text.
    EinzelArgument(Hilfe),
    /// Mehrere als Alternativen geparste Argumente.
    Alternativen(Box<NonEmpty<Alternativen>>),
    /// Fester Wert ohne assoziiertes Argument. Wird nicht im Hilfetext angezeigt.
    Leer,
}

impl From<Alternatives> for Alternativen {
    #[inline]
    fn from(alternatives: Alternatives) -> Self {
        match alternatives {
            Alternatives::Single(help) => Self::EinzelArgument(help.into()),
            Alternatives::Alternatives(alternatives) => {
                Self::Alternativen(Box::new(alternatives.map(Into::into)))
            },
            Alternatives::Empty => Self::Leer,
        }
    }
}

impl From<Alternativen> for Alternatives {
    #[inline]
    fn from(alternatives: Alternativen) -> Self {
        match alternatives {
            Alternativen::EinzelArgument(hilfe) => Self::Single(hilfe.into()),
            Alternativen::Alternativen(alternativen) => {
                Self::Alternatives(Box::new(alternativen.map(Into::into)))
            },
            Alternativen::Leer => Self::Empty,
        }
    }
}

/// Trait to simulate a rank-2 function.
pub trait CreateHelpText {
    /// Creates syntax and help text for an argument.
    fn create_help_text(
        &self,
        arg: EinzelArgument<'_, String, String>,
        meta_default: &str,
        meta_possible_values: &str,
    ) -> Hilfe;
}

/// Standard variant for creating help text for a single argument, e.g.:
/// `  --hilfe | -h    Zeige diesen Text an.`
#[derive(Debug, Clone, Copy)]
pub struct Standard;

#[allow(deprecated)]
impl CreateHelpText for Standard {
    #[inline]
    fn create_help_text(
        &self,
        arg: EinzelArgument<'_, String, String>,
        meta_default: &str,
        meta_possible_values: &str,
    ) -> Hilfe {
        arg.erzeuge_hilfe_text(meta_default, meta_possible_values)
    }
}

/// Default variant for creating help text for a single argument, e.g.:
/// `  --help | -h    Show this text.`
#[derive(Debug, Clone, Copy)]
pub struct Default;

#[allow(deprecated)]
impl CreateHelpText for Default {
    #[inline]
    fn create_help_text(
        &self,
        arg: EinzelArgument<'_, String, String>,
        meta_default: &str,
        meta_possible_values: &str,
    ) -> Hilfe {
        Standard.create_help_text(arg, meta_default, meta_possible_values)
    }
}
