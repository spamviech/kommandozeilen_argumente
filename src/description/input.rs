//! Input state retained while recognizing merged short forms.

use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
};

use nonempty::NonEmpty;

use crate::unicode::Normalized;

/// The remainder after successfully parsing merged short name arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjustedMergedShortNames {
    /// The prefix of the adjusted merged short names argument.
    pub prefix: Normalized<'static>,
    /// The graphemes of the remaining short names.
    pub graphemes: NonEmpty<Box<str>>,
    /// Is the suffix still intact.
    pub suffix: MergedShortNameSuffix,
}

/// Helper type to track parsing of merged short names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgumentInput {
    /// The input was not used yet.
    Unchanged(OsString),
    /// The remainder after successfully parsing merged short name arguments.
    AdjustedMergedShortNames(AdjustedMergedShortNames),
}

/// Helper type to track parsing of merged short names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgumentInputRef<'a> {
    /// The input was not used yet.
    Unchanged(Cow<'a, OsStr>),
    /// The remainder after successfully parsing merged short name arguments.
    AdjustedMergedShortNames(AdjustedMergedShortNames),
}

/// Is the suffix still intact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergedShortNameSuffix {
    /// The suffix has been removed.
    Removed,
    /// The suffix is still part of the remaining graphemes.
    Unchanged,
}

/// A short name recognized in a merged-short-name block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedMergedShortName<T> {
    /// The recognized single-grapheme short name.
    pub name: Box<str>,
    /// The value produced for the recognized name.
    pub value: T,
    /// An inline value following this short name, with its infix removed.
    pub inline_value: Option<Box<str>>,
    /// The unconsumed portion of the merged-short-name block.
    pub remaining: Option<ArgumentInput>,
}
