//! Value conversion for argument configurations.

use std::{ffi::OsString, fmt};

use nonempty::NonEmpty;

use crate::{
    arguments::{
        Arguments,
        combine::Combine,
        help::{Alternatives, CreateHelpText},
        parsing::{ParsedLongForms, ParsedMergedShortForms, ParsedShortForms, RecognizedArguments},
    },
    description::ArgumentInput,
    outcome::Result as OutcomeResult,
};

/// A type-erased combination whose parsed result is converted after accumulation.
///
/// The wrapper retains the original parser tree, including its recognition and help-text
/// behavior, while changing only its accumulated value.
pub(crate) struct MappedArguments<'t, T, S, Error> {
    /// The parser definition producing the original value type.
    arguments: Arguments<'t, T, Error>,
    /// Converts the accumulated value.
    mapper: Box<dyn 't + Fn(T) -> S>,
}

impl<'t, T, S, Error> MappedArguments<'t, T, S, Error> {
    /// Creates a value-converting wrapper around `arguments`.
    #[inline]
    pub(crate) fn new(mapper: impl 't + Fn(T) -> S, arguments: Arguments<'t, T, Error>) -> Self {
        Self { arguments, mapper: Box::new(mapper) }
    }
}

impl<'t, T, S, Error> Combine<'t, S, Error> for MappedArguments<'t, T, S, Error> {
    #[inline]
    fn create_help_text(
        &self,
        variant: &dyn CreateHelpText,
        meta_default: &str,
        meta_possible_values: &str,
    ) -> NonEmpty<Alternatives> {
        self.arguments.create_help_text(variant, meta_default, meta_possible_values)
    }

    #[inline]
    fn debug_fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("MappedArguments").field(&"<parser>").finish()
    }

    #[inline]
    fn parse_merged_short_forms(
        &self,
        args: Box<dyn Iterator<Item = OsString> + '_>,
    ) -> NonEmpty<ParsedMergedShortForms<'t>> {
        self.arguments.parse_merged_short_forms(args)
    }

    #[inline]
    fn parse_merged_short_inputs(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedMergedShortForms<'t>> {
        self.arguments.parse_merged_short_inputs(args)
    }

    #[inline]
    fn parse_short_forms(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedShortForms<'t>> {
        self.arguments.parse_short_forms(args)
    }

    #[inline]
    fn parse_long_forms(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedLongForms<'t>> {
        self.arguments.parse_long_forms(args)
    }

    #[inline]
    fn validate(&self, recognized: RecognizedArguments<'t>) -> OutcomeResult<'t, (), Error> {
        self.arguments.validate(recognized)
    }

    #[inline]
    fn accumulate(
        self: Box<Self>,
        recognized: RecognizedArguments<'t>,
    ) -> OutcomeResult<'t, S, Error> {
        let Self { arguments, mapper } = *self;
        arguments.accumulate(recognized).convert(&*mapper)
    }
}
