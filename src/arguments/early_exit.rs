//! Flag arguments that cause early exits.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ffi::OsString,
};

use nonempty::NonEmpty;
use void::Void;

#[cfg(feature = "german")]
use crate::{arguments::help::Hilfe, description::Beschreibung};
use crate::{
    arguments::{
        help::Help, parsing::ParsedEarlyExit, parsing::ParsedLongForms,
        parsing::ParsedMergedShortForms, parsing::ParsedShortForms,
    },
    description::{ArgumentInput, Description, Name},
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
#[cfg(feature = "german")]
pub struct FrühesBeenden<'t> {
    /// Allgemeine Beschreibung des Arguments.
    pub beschreibung: Beschreibung<'t, Void>,
    /// Die beim Parsen des Arguments angezeigte Nachricht.
    pub nachricht: Cow<'t, str>,
}

#[cfg(feature = "german")]
impl<'t> From<EarlyExit<'t>> for FrühesBeenden<'t> {
    #[inline]
    fn from(EarlyExit { description, message }: EarlyExit<'t>) -> Self {
        Self { beschreibung: description.into(), nachricht: message }
    }
}

#[cfg(feature = "german")]
impl<'t> From<FrühesBeenden<'t>> for EarlyExit<'t> {
    #[inline]
    fn from(FrühesBeenden { beschreibung, nachricht }: FrühesBeenden<'t>) -> Self {
        Self { description: beschreibung.into(), message: nachricht }
    }
}

/// Creates help text from an early-exit argument's metadata.
#[must_use]
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

impl<'t> EarlyExit<'t> {
    /// Creates a flag that causes an early exit and displays `message`.
    #[inline]
    pub fn new(description: Description<'t, Void>, message: impl Into<Cow<'t, str>>) -> Self {
        Self { description, message: message.into() }
    }

    /// Creates this argument's syntax and help text.
    #[inline]
    #[must_use]
    pub fn create_help_text(&self) -> Help {
        create_help_text(&self.description.name, self.description.help, self.description.default)
    }

    /// Parses standalone short-form arguments.
    #[inline]
    pub fn parse_short_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedShortForms<'t> {
        let Self { description, message } = self;
        let mut early_exits = Vec::new();
        let remaining = args
            .filter_map(|argument| match argument {
                ArgumentInput::Unchanged(argument) => {
                    let input = Cow::Owned(argument.to_string_lossy().into_owned());
                    if let Some(name) = description.name.parse_short_early_exit(&argument) {
                        early_exits.push(ParsedEarlyExit {
                            name: Cow::Owned(name.into()),
                            message: message.clone(),
                            input,
                        });
                        None
                    } else {
                        Some(ArgumentInput::Unchanged(argument))
                    }
                },
                adjusted @ ArgumentInput::AdjustedMergedShortNames(_) => Some(adjusted),
            })
            .collect();
        ParsedShortForms {
            early_exits,
            flags: Vec::new(),
            missing_values: HashSet::default(),
            values: HashMap::default(),
            remaining,
            alternative_branches: Vec::new(),
        }
    }

    /// Parses long-form arguments.
    #[inline]
    pub fn parse_long_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedLongForms<'t> {
        let Self { description, message } = self;
        let mut early_exits = Vec::new();
        let remaining = args
            .filter_map(|argument| match argument {
                ArgumentInput::Unchanged(argument) => {
                    let input = Cow::Owned(argument.to_string_lossy().into_owned());
                    description.name.parse_long_early_exit(&argument).map_or_else(
                        || Some(ArgumentInput::Unchanged(argument)),
                        |name| {
                            early_exits.push(ParsedEarlyExit {
                                name: Cow::Owned(name.into()),
                                message: message.clone(),
                                input,
                            });
                            None
                        },
                    )
                },
                adjusted @ ArgumentInput::AdjustedMergedShortNames(_) => Some(adjusted),
            })
            .collect();
        ParsedLongForms {
            early_exits,
            flags: Vec::new(),
            missing_values: HashSet::default(),
            values: HashMap::default(),
            remaining,
            alternative_branches: Vec::new(),
        }
    }

    /// Parses merged short-form arguments.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParsedMergedShortForms<'t> {
        self.parse_merged_short_inputs(args.map(ArgumentInput::Unchanged))
    }

    /// Parses merged short-form arguments while retaining adjusted-block remainders.
    #[inline]
    pub(crate) fn parse_merged_short_inputs(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedMergedShortForms<'t> {
        let Self { description, message } = self;
        let mut early_exits = Vec::new();
        let remaining = args
            .filter_map(|argument| {
                let input = match &argument {
                    ArgumentInput::Unchanged(argument) => {
                        Cow::Owned(argument.to_string_lossy().into_owned())
                    },
                    ArgumentInput::AdjustedMergedShortNames(_) => Cow::Borrowed(""),
                };
                if let Some(parsed) = description.name.parse_early_exit_merge_short_forms(&argument)
                {
                    early_exits.push(ParsedEarlyExit {
                        name: Cow::Owned(parsed.name.into()),
                        message: message.clone(),
                        input,
                    });
                    parsed.remaining
                } else {
                    Some(argument)
                }
            })
            .collect();
        ParsedMergedShortForms {
            early_exits,
            flags: Vec::new(),
            missing_values: HashSet::default(),
            values: HashMap::default(),
            remaining,
            alternative_branches: Vec::new(),
        }
    }
}

#[cfg(feature = "german")]
impl<'t> FrühesBeenden<'t> {
    /// Erstellt eine Flag, die ein frühes Beenden auslöst und `nachricht` anzeigt.
    #[inline]
    pub fn neu(beschreibung: Beschreibung<'t, Void>, nachricht: impl Into<Cow<'t, str>>) -> Self {
        EarlyExit::new(beschreibung.into(), nachricht).into()
    }

    /// Erzeugt Syntax und Hilfetext für dieses Argument.
    #[inline]
    #[must_use]
    pub fn erzeuge_hilfe_text(&self) -> Hilfe {
        EarlyExit::from(self.clone()).create_help_text().into()
    }

    /// Parst eigenständige kurze Argumentformen.
    #[inline]
    pub fn parse_short_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedShortForms<'t> {
        EarlyExit::from(self.clone()).parse_short_form(args)
    }

    /// Parst lange Argumentformen.
    #[inline]
    pub fn parse_long_form(
        &self,
        args: impl Iterator<Item = ArgumentInput>,
    ) -> ParsedLongForms<'t> {
        EarlyExit::from(self.clone()).parse_long_form(args)
    }

    /// Parst zusammengefasste kurze Argumentformen.
    #[inline]
    pub fn parse_merged_short_forms(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParsedMergedShortForms<'t> {
        EarlyExit::from(self.clone()).parse_merged_short_forms(args)
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::indexing_slicing, reason = "Easier assert_eq! for the first element.")]

    use std::ffi::OsString;

    use void::Void;

    use crate::{ArgumentInput, arguments::early_exit::EarlyExit, description::Description};

    const EMPTY_REMAINING: [ArgumentInput; 0] = [];

    #[test]
    fn parses_merged_short_early_exit() {
        let early_exit = EarlyExit::new(
            Description::new("--", "help", "-", "h", None, None::<Void>),
            "help message",
        );
        let result = early_exit.parse_merged_short_forms([OsString::from("-h")].into_iter());

        assert_eq!(result.early_exits.len(), 1);
        assert_eq!(result.early_exits[0].name, "h");
        assert_eq!(result.early_exits[0].message, "help message");
        assert_eq!(result.remaining, EMPTY_REMAINING);
    }

    #[test]
    fn parses_a_standalone_short_early_exit() {
        let early_exit = EarlyExit::new(
            Description::new("--", "help", "-", "h", None, None::<Void>),
            "help message",
        );
        let result = early_exit
            .parse_short_form([OsString::from("-h")].into_iter().map(ArgumentInput::Unchanged));

        assert_eq!(result.early_exits.len(), 1);
        assert_eq!(result.early_exits[0].name, "h");
        assert_eq!(result.remaining, EMPTY_REMAINING);
    }

    #[test]
    fn parses_a_long_early_exit() {
        let early_exit = EarlyExit::new(
            Description::new("--", "help", "-", "h", None, None::<Void>),
            "help message",
        );
        let result = early_exit
            .parse_long_form([OsString::from("--help")].into_iter().map(ArgumentInput::Unchanged));

        assert_eq!(result.early_exits.len(), 1);
        assert_eq!(result.early_exits[0].name, "help");
        assert_eq!(result.remaining, EMPTY_REMAINING);
    }
}
