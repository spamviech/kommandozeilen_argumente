//! Description of a command-line argument.
//!
//! ## Deutsch
//! Beschreibung eines Arguments.

use std::{
    borrow::Cow,
    convert::AsRef,
    ffi::{OsStr, OsString},
    iter,
};

use itertools::Itertools as _;
use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    language::{Language, Sprache},
    unicode::{Case, Compare, Normalized},
};

/// All names of an argument.
///
/// ## Deutsch
/// Alle Namen eines Arguments.
#[derive(Debug, Clone)]
pub struct Name<'t> {
    /// Prefix before the long name.
    ///
    /// ## Deutsch
    /// Präfix vor dem Lang-Namen.
    pub long_prefix: Compare<'t>,

    /// Full name, given after `long_prefix`.
    ///
    /// ## Deutsch
    /// Voller Name, wird nach `long_prefix` angegeben.
    pub long: NonEmpty<Compare<'t>>,

    /// Prefix before the short name.
    ///
    /// ## Deutsch
    /// Präfix vor dem Kurz-Namen.
    pub short_prefix: Compare<'t>,

    /// Short name, given after `short_prefix`.
    /// Flag arguments with identical `short_prefix` may be given at once, e.g. "-fgh".
    /// Short names longer than a
    /// [`Grapheme`](unicode_segmentation::UnicodeSegmentation::graphemes) are not supported.
    ///
    /// ## Deutsch
    /// Kurzer Name, wird nach `short_prefix` angegeben.
    /// Bei Flag-Argumenten können Kurz-Namen mit identischen `short_prefix` zusammen angegeben
    /// werden, zum Beispiel "-fgh".
    /// Kurznamen länger als ein [`Grapheme`](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// werden nicht unterstützt.
    pub short: Vec<Compare<'t>>,
}

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

impl Name<'_> {
    /// Helper for [`parse_flag_aux`](Name::parse_flag_aux).
    fn parse_merged_short_name<E>(
        prefix: Normalized<'_>,
        short: &[Compare<'_>],
        value_found: impl FnOnce() -> E,
        graphemes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Option<ParsedMergedShortName<E>> {
        let (first_match, others, suffix) = graphemes.into_iter().fold(
            (None, Vec::new(), MergedShortNameSuffix::Unchanged),
            |(mut first_match, mut others, _suffix), grapheme| {
                let suffix;
                if first_match.is_some() || !contains_str(short, grapheme.as_ref()) {
                    suffix = MergedShortNameSuffix::Unchanged;
                    others.push(Box::from(grapheme.as_ref()));
                } else {
                    suffix = MergedShortNameSuffix::Removed;
                    first_match = Some(Box::from(grapheme.as_ref()));
                }
                (first_match, others, suffix)
            },
        );

        first_match.map(|name| {
            let value = value_found();
            let adjusted_argument = NonEmpty::from_vec(others).map(|remaining| {
                ArgumentInput::AdjustedMergedShortNames(AdjustedMergedShortNames {
                    prefix: prefix.into_owned(),
                    graphemes: remaining,
                    suffix,
                })
            });
            ParsedMergedShortName { name, value, inline_value: None, remaining: adjusted_argument }
        })
    }

    /// Helper for [`parse_flag`](Name::parse_flag) and its variants.
    ///
    /// Returns [`Some`] when a name was found and [`None`] otherwise.
    fn parse_flag_aux<E>(
        &self,
        name_gefunden: impl FnOnce() -> E,
        parse_invertiert: impl FnOnce(&NonEmpty<Compare<'_>>, &Normalized<'_>) -> Option<E>,
        arg: &OsStr,
    ) -> Option<E> {
        let Name { long_prefix, long, short_prefix, short } = self;
        let name_kurz_existiert = !short.is_empty();
        if let Some(string) = arg.to_str() {
            let normalized = Normalized::new(string);
            if let Some((_prefix, lang_str)) = &long_prefix.strip_as_prefix_n(&normalized) {
                #[allow(clippy::redundant_else)]
                if contains_str(long, lang_str.as_str()) {
                    return Some(name_gefunden());
                } else if let Some(wert) = parse_invertiert(long, lang_str) {
                    return Some(wert);
                } else {
                    // kein match für {long_prefix}[invertiert_infix]{lang_name}
                }
            } else if name_kurz_existiert {
                if let Some((_prefix, kurz_suffix)) = short_prefix.strip_as_prefix_n(&normalized) {
                    if contains_str(short, kurz_suffix.as_str()) {
                        return Some(name_gefunden());
                    }
                }
            } else {
                // kein match für "{long_prefix}.*" und es gibt keine Kurz-Namen.
            }
        }
        None
    }

    /// Helper for [`parse_flag_merge_short_forms`](Name::parse_flag_merge_short_forms) and its
    /// variants.
    ///
    /// Returns [`Some`] when a name was found and [`None`] otherwise.
    fn parse_flag_merge_short_forms_aux<E>(
        &self,
        value_found: impl FnOnce() -> E,
        arg: &ArgumentInput,
    ) -> Option<ParsedMergedShortName<E>> {
        let Name { long_prefix: _, long: _, short_prefix, short } = self;
        if short.is_empty() {
            return None;
        }
        match arg {
            ArgumentInput::Unchanged(arg) => {
                if let Some(string) = arg.to_str() {
                    let normalized = Normalized::new(string);
                    if let Some((prefix, kurz_suffix)) = short_prefix.strip_as_prefix_n(&normalized)
                    {
                        let graphemes: Box<dyn Iterator<Item = &str>> =
                            match kurz_suffix.as_str().graphemes(true).exactly_one() {
                                Ok(einzelnes) => Box::new(iter::once(einzelnes)),
                                Err(mehrere) => Box::new(mehrere),
                            };

                        return Name::parse_merged_short_name(
                            prefix,
                            short,
                            value_found,
                            graphemes,
                        );
                    }
                }
            },
            ArgumentInput::AdjustedMergedShortNames(AdjustedMergedShortNames {
                prefix,
                graphemes,
                suffix: _,
            }) => {
                return Name::parse_merged_short_name(
                    prefix.clone(),
                    short,
                    value_found,
                    graphemes,
                );
            },
        }
        None
    }

    /// Parses the name as a flag.
    ///
    /// Returns [`Some`] when a name was found and [`None`] otherwise.
    #[inline]
    pub(crate) fn parse_flag(
        &self,
        invertiere_präfix: &Compare<'_>,
        invertiere_infix: &Compare<'_>,
        arg: &OsStr,
    ) -> Option<bool> {
        let parse_invertiert = |lang: &NonEmpty<Compare<'_>>,
                                lang_str: &Normalized<'_>|
         -> Option<bool> {
            if let Some((_prefix, infix_name)) = invertiere_präfix.strip_as_prefix_n(lang_str) {
                let infix_name_normalized = infix_name;
                if let Some((_infix, negiert)) =
                    invertiere_infix.strip_as_prefix_n(&infix_name_normalized)
                {
                    if contains_str(lang, negiert.as_str()) {
                        return Some(false);
                    }
                }
            }
            None
        };
        self.parse_flag_aux(|| true, parse_invertiert, arg)
    }

    /// Parses a long name as a flag.
    ///
    /// Returns the matched long name and its value, including inversion when configured.
    #[inline]
    pub(crate) fn parse_long_flag(
        &self,
        invert_prefix: &Compare<'_>,
        invert_infix: &Compare<'_>,
        arg: &OsStr,
    ) -> Option<(Box<str>, bool)> {
        let string = arg.to_str()?;
        let normalized = Normalized::new(string);
        let (_, suffix) = self.long_prefix.strip_as_prefix_n(&normalized)?;
        if let Some(name) = self.long.iter().find(|name| (**name).eq(suffix.as_str())) {
            return Some((name.as_str().into(), true));
        }
        let (_, suffix) = invert_prefix.strip_as_prefix_n(&suffix)?;
        let (_, suffix) = invert_infix.strip_as_prefix_n(&suffix)?;
        self.long
            .iter()
            .find(|name| (**name).eq(suffix.as_str()))
            .map(|name| (name.as_str().into(), false))
    }

    /// Parses a long name as a flag that causes an early exit.
    #[inline]
    pub(crate) fn parse_long_early_exit(&self, arg: &OsStr) -> Option<Box<str>> {
        let string = arg.to_str()?;
        let normalized = Normalized::new(string);
        let (_, suffix) = self.long_prefix.strip_as_prefix_n(&normalized)?;
        self.long.iter().find(|name| (**name).eq(suffix.as_str())).map(|name| name.as_str().into())
    }

    /// Parses a standalone short name as a flag.
    ///
    /// Returns [`Some`] when a short name was found and [`None`] otherwise.
    #[inline]
    pub(crate) fn parse_short_flag(
        &self,
        invert_prefix: &Compare<'_>,
        invert_infix: &Compare<'_>,
        arg: &OsStr,
    ) -> Option<(Box<str>, bool)> {
        let string = arg.to_str()?;
        let normalized = Normalized::new(string);
        let (_, suffix) = self.short_prefix.strip_as_prefix_n(&normalized)?;
        let name = self.short.iter().find(|name| (**name).eq(suffix.as_str()))?.as_str().into();
        self.parse_flag(invert_prefix, invert_infix, arg).map(|value| (name, value))
    }

    /// Parses the name as a flag.
    ///
    /// Returns [`Some`] when a name was found and [`None`] otherwise.
    #[inline]
    pub(crate) fn parse_flag_merge_short_forms(
        &self,
        arg: &ArgumentInput,
    ) -> Option<ParsedMergedShortName<bool>> {
        self.parse_flag_merge_short_forms_aux(|| true, arg)
    }

    /// Parses the name as a flag that causes an early exit.
    ///
    /// Returns [`Some`] when a name was found and [`None`] otherwise.
    #[inline]
    #[allow(clippy::option_option)]
    pub(crate) fn parse_early_exit(&self, arg: &OsStr) -> bool {
        self.parse_flag_aux(|| (), |_, _| None, arg).is_some()
    }

    /// Parses a standalone short name as a flag that causes an early exit.
    #[inline]
    pub(crate) fn parse_short_early_exit(&self, arg: &OsStr) -> Option<Box<str>> {
        let string = arg.to_str()?;
        let normalized = Normalized::new(string);
        let (_, suffix) = self.short_prefix.strip_as_prefix_n(&normalized)?;
        let name = self.short.iter().find(|name| (**name).eq(suffix.as_str()))?.as_str().into();
        self.parse_early_exit(arg).then_some(name)
    }

    /// Parses the name as a flag that causes an early exit.
    ///
    /// Returns [`Some`] when a name was found and [`None`] otherwise.
    #[inline]
    #[allow(clippy::option_option)]
    pub(crate) fn parse_early_exit_merge_short_forms(
        &self,
        arg: &ArgumentInput,
    ) -> Option<ParsedMergedShortName<()>> {
        self.parse_flag_merge_short_forms_aux(|| (), arg)
    }

    /// Parses a long name as a value.
    #[allow(clippy::option_option)]
    pub(crate) fn parse_long_with_value<'t>(
        &self,
        value_infix: &Compare<'_>,
        arg: &'t OsStr,
    ) -> Option<(Box<str>, Option<Cow<'t, OsStr>>)> {
        let string = arg.to_str()?;
        for name in &self.long {
            if long_name_matches(&self.long_prefix, name, string) {
                return Some((name.as_str().into(), None));
            }
            if let Some(value) =
                raw_value_after_long_name(&self.long_prefix, name, value_infix, string)
            {
                return Some((name.as_str().into(), Some(value)));
            }
        }
        None
    }

    /// Parses a standalone short name as a value.
    #[allow(clippy::option_option)]
    pub(crate) fn parse_short_with_value<'t>(
        &self,
        value_infix: &Compare<'_>,
        arg: &'t OsStr,
    ) -> Option<(Box<str>, Option<Cow<'t, OsStr>>)> {
        let string = arg.to_str()?;
        let normalized = Normalized::new(string);
        let (_, suffix) = self.short_prefix.strip_as_prefix_n(&normalized)?;
        let short_name = suffix.as_str().graphemes(true).next()?;
        let name = self.short.iter().find(|name| (**name).eq(short_name))?;
        let value = raw_value_after_short_name(&self.short_prefix, name, value_infix, string)?
            .map(|value| Cow::Borrowed(OsStr::new(value)));
        Some((name.as_str().into(), value))
    }

    /// Parses the name as a value.
    #[allow(clippy::option_option)]
    pub(crate) fn parse_with_value_merge_short_forms(
        &self,
        value_infix: &Compare<'_>,
        arg: &ArgumentInput,
    ) -> Option<ParsedMergedShortName<()>> {
        let Name { long_prefix: _, long: _, short_prefix, short } = self;
        if short.is_empty() {
            return None;
        }
        let (prefix, graphemes): (Normalized<'static>, Vec<Box<str>>) = match arg {
            ArgumentInput::Unchanged(arg) => {
                let string = arg.to_str()?;
                let normalized = Normalized::new(string);
                let (prefix, suffix) = short_prefix.strip_as_prefix_n(&normalized)?;
                (prefix.into_owned(), suffix.as_str().graphemes(true).map(Box::from).collect())
            },
            ArgumentInput::AdjustedMergedShortNames(AdjustedMergedShortNames {
                prefix,
                graphemes,
                suffix: MergedShortNameSuffix::Unchanged,
            }) if short_prefix.eq(prefix.as_str()) => (prefix.clone(), graphemes.clone().into()),
            ArgumentInput::AdjustedMergedShortNames { .. } => return None,
        };

        let (index, name) = graphemes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, grapheme)| contains_str(short, grapheme))?;
        let inline_value = graphemes[index + 1..].concat();
        let inline_value = match arg {
            ArgumentInput::Unchanged(argument) => argument.to_str().and_then(|input| {
                raw_value_after_merged_short_name(short_prefix, short, value_infix, input)
            }),
            ArgumentInput::AdjustedMergedShortNames { .. } => {
                (!inline_value.is_empty()).then(|| {
                    match raw_suffix_after_prefix(value_infix, &inline_value) {
                        Some(value) => Box::from(value),
                        None => Box::from(inline_value),
                    }
                })
            },
        };
        let remaining = NonEmpty::collect(graphemes[..index].iter().cloned()).map(|graphemes| {
            ArgumentInput::AdjustedMergedShortNames(AdjustedMergedShortNames {
                prefix: prefix.into_owned(),
                graphemes,
                suffix: MergedShortNameSuffix::Removed,
            })
        });
        Some(ParsedMergedShortName { name: name.clone(), value: (), inline_value, remaining })
    }

    /// Appends a regular-expression representation of the long names to `string`.
    pub(crate) fn alternatives_as_regex(
        head: &Compare<'_>,
        tail: &[Compare<'_>],
        string: &mut String,
    ) {
        if !tail.is_empty() {
            string.push('(');
        }
        string.push_str(head.as_str());
        for elem in tail {
            string.push('|');
            string.push_str(elem.as_str());
        }
        if !tail.is_empty() {
            string.push(')');
        }
    }
}

/// Description of a command line argument.
#[must_use]
#[derive(Debug, Clone)]
pub struct Description<'t, T> {
    /// Names used to invoke the argument.
    pub name: Name<'t>,
    /// Description shown in automatically generated help text.
    pub help: Option<&'t str>,
    /// Value used when the argument is absent.
    pub default: Option<T>,
}

/// Description of a command-line argument.
///
/// ## Deutsch
/// Beschreibung eines Kommandozeilen-Arguments.
#[must_use]
#[derive(Debug, Clone)]
pub struct Beschreibung<'t, T> {
    /// Names used to invoke the argument.
    ///
    /// ## Deutsch
    /// Namen zum Verwenden des Arguments.
    pub name: Name<'t>,
    /// Description shown in automatically generated help text.
    ///
    /// ## Deutsch
    /// Im automatisch erzeugten Hilfetext angezeigte Beschreibung.
    pub hilfe: Option<&'t str>,
    /// Value used when the argument is absent.
    ///
    /// ## Deutsch
    /// Standardwert, wenn das Argument nicht verwendet wurde.
    pub standard: Option<T>,
}

impl<'t, T> From<Description<'t, T>> for Beschreibung<'t, T> {
    #[inline]
    fn from(Description { name, help, default }: Description<'t, T>) -> Self {
        Self { name, hilfe: help, standard: default }
    }
}

impl<'t, T> From<Beschreibung<'t, T>> for Description<'t, T> {
    #[inline]
    fn from(Beschreibung { name, hilfe, standard }: Beschreibung<'t, T>) -> Self {
        Self { name, help: hilfe, default: standard }
    }
}

impl<'t, T> Description<'t, T> {
    /// Converts this description to a different value type.
    #[inline]
    pub fn convert<S>(self, convert: impl FnOnce(T) -> S) -> Description<'t, S> {
        let Self { name, help, default } = self;
        Description { name, help, default: default.map(convert) }
    }

    /// Borrows the default value.
    #[inline]
    pub fn as_ref(&self) -> Description<'t, &T> {
        let Self { name, help, default } = self;
        Description { name: name.clone(), help: *help, default: default.as_ref() }
    }

    /// Creates an argument description.
    #[inline]
    pub fn new(
        long_prefix: impl Into<Compare<'t>>,
        long: impl LongNames<'t>,
        short_prefix: impl Into<Compare<'t>>,
        short: impl ShortNames<'t>,
        help: Option<&'t str>,
        default: Option<T>,
    ) -> Self {
        Self {
            name: Name {
                long_prefix: long_prefix.into(),
                long: long.long_names(),
                short_prefix: short_prefix.into(),
                short: short.short_names(),
            },
            help,
            default,
        }
    }

    /// Creates an argument description with localized prefixes.
    #[inline]
    pub fn new_with_language(
        long: impl LongNames<'t>,
        short: impl ShortNames<'t>,
        help: Option<&'t str>,
        default: Option<T>,
        language: Language,
    ) -> Self {
        Self::new(language.long_prefix, long, language.short_prefix, short, help, default)
    }
}

impl<'t, T> Beschreibung<'t, T> {
    /// Converts this description to another value type.
    ///
    /// ## Deutsch
    /// Konvertiert diese Beschreibung in einen anderen Werttyp.
    #[inline]
    pub fn konvertiere<S>(self, konvertiere: impl FnOnce(T) -> S) -> Beschreibung<'t, S> {
        Description::from(self).convert(konvertiere).into()
    }

    /// Borrows the default value.
    ///
    /// ## Deutsch
    /// Leiht den Standardwert aus.
    #[inline]
    pub fn as_ref(&self) -> Beschreibung<'t, &T> {
        Description { name: self.name.clone(), help: self.hilfe, default: self.standard.as_ref() }
            .into()
    }

    /// Creates an argument description.
    ///
    /// ## Deutsch
    /// Erzeugt eine Argumentbeschreibung.
    #[inline]
    pub fn neu(
        lang_präfix: impl Into<Compare<'t>>,
        lang: impl LongNames<'t>,
        kurz_präfix: impl Into<Compare<'t>>,
        kurz: impl ShortNames<'t>,
        hilfe: Option<&'t str>,
        standard: Option<T>,
    ) -> Self {
        Description::new(lang_präfix.into(), lang, kurz_präfix.into(), kurz, hilfe, standard).into()
    }

    /// Creates an argument description with localized prefixes.
    ///
    /// ## Deutsch
    /// Erzeugt eine Argumentbeschreibung mit lokalisierten Präfixen.
    #[inline]
    pub fn neu_mit_sprache(
        lang: impl LongNames<'t>,
        kurz: impl ShortNames<'t>,
        hilfe: Option<&'t str>,
        standard: Option<T>,
        sprache: Sprache,
    ) -> Self {
        Description::new_with_language(lang, kurz, hilfe, standard, sprache.into()).into()
    }
}

/// Returns the raw suffix after a normalized match of `prefix`.
///
/// The match boundary is found by testing grapheme-boundary prefixes of the original input, so
/// the returned slice always refers to the original, unnormalized string.
fn raw_suffix_after_prefix<'t>(prefix: &Compare<'_>, input: &'t str) -> Option<&'t str> {
    input
        .grapheme_indices(true)
        .map(|(index, _)| index)
        .chain(iter::once(input.len()))
        .find(|&index| prefix.eq(&input[..index]))
        .map(|index| &input[index..])
}

/// Returns whether the raw input is a normalized match for a complete long name.
fn long_name_matches(long_prefix: &Compare<'_>, name: &Compare<'_>, input: &str) -> bool {
    let normalized = Normalized::new(input);
    long_prefix.strip_as_prefix_n(&normalized).is_some_and(|(_, suffix)| name.eq(suffix.as_str()))
}

/// Returns the raw value suffix after a long name and value infix.
///
/// Candidate prefixes end at grapheme boundaries in the original input. Each candidate is then
/// normalized for matching, so the returned `OsStr` preserves the exact unnormalized value.
fn raw_value_after_long_name<'t>(
    long_prefix: &Compare<'_>,
    name: &Compare<'_>,
    value_infix: &Compare<'_>,
    input: &'t str,
) -> Option<Cow<'t, OsStr>> {
    raw_suffix_after_prefix(
        value_infix,
        raw_suffix_after_prefix(name, raw_suffix_after_prefix(long_prefix, input)?)?,
    )
    .map(|value| Cow::Borrowed(OsStr::new(value)))
}

/// Returns the raw value suffix after a short name and value infix.
fn raw_value_after_short_name<'t>(
    short_prefix: &Compare<'_>,
    name: &Compare<'_>,
    value_infix: &Compare<'_>,
    input: &'t str,
) -> Option<Option<&'t str>> {
    let suffix = raw_suffix_after_prefix(name, raw_suffix_after_prefix(short_prefix, input)?)?;
    if suffix.is_empty() {
        Some(None)
    } else {
        Some(Some(raw_suffix_after_prefix(value_infix, suffix).unwrap_or(suffix)))
    }
}

/// Returns the raw inline value after the last recognized short name in a merged block.
fn raw_value_after_merged_short_name(
    short_prefix: &Compare<'_>,
    short_names: &[Compare<'_>],
    value_infix: &Compare<'_>,
    input: &str,
) -> Option<Box<str>> {
    let suffix = raw_suffix_after_prefix(short_prefix, input)?;
    let (_, name) = suffix
        .grapheme_indices(true)
        .filter_map(|(index, grapheme)| {
            short_names
                .iter()
                .any(|short_name| short_name.eq(grapheme))
                .then_some((index, grapheme))
        })
        .last()?;
    let value_suffix = &suffix[name.len()..];
    (!value_suffix.is_empty()).then(|| {
        Box::from(raw_suffix_after_prefix(value_infix, value_suffix).unwrap_or(value_suffix))
    })
}

/// Returns whether `searched` is in `collection`.
pub(crate) fn contains_str<'t>(
    collection: impl IntoIterator<Item = &'t Compare<'t>>,
    searched: &str,
) -> bool {
    collection.into_iter().any(|target| target.eq(searched))
}

/// One or more long names.
pub trait LongNames<'t> {
    /// Converts to a non-empty collection of comparisons.
    fn long_names(self) -> NonEmpty<Compare<'t>>;
}

macro_rules! impl_long_names {
    ($type:ty) => {
        impl<'t> LongNames<'t> for $type {
            #[inline]
            fn long_names(self) -> NonEmpty<Compare<'t>> {
                NonEmpty::singleton(self.into())
            }
        }
        impl<'t> LongNames<'t> for ($type, Case) {
            #[inline]
            fn long_names(self) -> NonEmpty<Compare<'t>> {
                NonEmpty::singleton(self.into())
            }
        }
        impl<'t> LongNames<'t> for NonEmpty<$type> {
            #[inline]
            fn long_names(self) -> NonEmpty<Compare<'t>> {
                let NonEmpty { head, tail } = self;
                NonEmpty { head: head.into(), tail: tail.into_iter().map(Into::into).collect() }
            }
        }
        impl<'t> LongNames<'t> for NonEmpty<($type, Case)> {
            #[inline]
            fn long_names(self) -> NonEmpty<Compare<'t>> {
                let NonEmpty { head, tail } = self;
                NonEmpty { head: head.into(), tail: tail.into_iter().map(Into::into).collect() }
            }
        }
    };
}
impl_long_names! {String}
impl_long_names! {&'t str}
impl_long_names! {Normalized<'t>}
impl<'t> LongNames<'t> for Compare<'t> {
    #[inline]
    fn long_names(self) -> NonEmpty<Compare<'t>> {
        NonEmpty::singleton(self)
    }
}
impl<'t> LongNames<'t> for NonEmpty<Compare<'t>> {
    #[inline]
    fn long_names(self) -> NonEmpty<Compare<'t>> {
        self
    }
}
impl<'t, S: AsRef<str>> LongNames<'t> for &'t NonEmpty<S> {
    #[inline]
    fn long_names(self) -> NonEmpty<Compare<'t>> {
        let NonEmpty { head, tail } = self;
        NonEmpty {
            head: head.as_ref().into(),
            tail: tail.iter().map(|value| value.as_ref().into()).collect(),
        }
    }
}

/// Zero or more short names.
pub trait ShortNames<'t> {
    /// Converts to comparisons.
    fn short_names(self) -> Vec<Compare<'t>>;
}
macro_rules! impl_short_names {
    ($type:ty) => {
        impl<'t> ShortNames<'t> for $type {
            #[inline]
            fn short_names(self) -> Vec<Compare<'t>> {
                vec![self.into()]
            }
        }
        impl<'t> ShortNames<'t> for ($type, Case) {
            #[inline]
            fn short_names(self) -> Vec<Compare<'t>> {
                vec![self.into()]
            }
        }
        macro_rules! impl_into_iter {
            ($collection: ident) => {
                impl<'t> ShortNames<'t> for $collection<$type> {
                    #[inline]
                    fn short_names(self) -> Vec<Compare<'t>> {
                        self.into_iter().map(Into::into).collect()
                    }
                }
                impl<'t> ShortNames<'t> for $collection<($type, Case)> {
                    #[inline]
                    fn short_names(self) -> Vec<Compare<'t>> {
                        self.into_iter().map(Into::into).collect()
                    }
                }
            };
        }
        impl_into_iter! {Option}
        impl_into_iter! {Vec}
        impl_into_iter! {NonEmpty}
    };
}
impl_short_names! {String}
impl_short_names! {&'t str}
impl_short_names! {Normalized<'t>}
impl<'t> ShortNames<'t> for Vec<Compare<'t>> {
    #[inline]
    fn short_names(self) -> Vec<Compare<'t>> {
        self
    }
}
impl<'t, S: AsRef<str>> ShortNames<'t> for &'t Vec<S> {
    #[inline]
    fn short_names(self) -> Vec<Compare<'t>> {
        self.iter().map(|value| value.as_ref().into()).collect()
    }
}
