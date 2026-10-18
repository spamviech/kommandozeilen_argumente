//! Argument names and low-level form recognition.

use std::{convert::AsRef, ffi::OsStr, iter};

use itertools::Itertools as _;
use nonempty::NonEmpty;
use unicode_segmentation::UnicodeSegmentation;

use super::{
    AdjustedMergedShortNames, ArgumentInput, MergedShortNameSuffix, contains_str,
    input::ParsedMergedShortName,
};
use crate::unicode::{Compare, Normalized};

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
                #[expect(
                    clippy::redundant_else,
                    reason = "The branches are deliberately visually separated while parsing \
                              argument forms."
                )]
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
        let Name { long_prefix, long: _, short_prefix, short } = self;
        if short.is_empty() {
            return None;
        }
        match arg {
            ArgumentInput::Unchanged(arg) => {
                if let Some(string) = arg.to_str() {
                    let normalized = Normalized::new(string);
                    // A long-form prefix must not be interpreted as a short-prefix followed by
                    // a merged block (for example, `--help` is not `-h` plus `elp`).
                    if long_prefix.strip_as_prefix_n(&normalized).is_some() {
                        return None;
                    }
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
    pub(crate) fn parse_early_exit_merge_short_forms(
        &self,
        arg: &ArgumentInput,
    ) -> Option<ParsedMergedShortName<()>> {
        self.parse_flag_merge_short_forms_aux(|| (), arg)
    }

    /// Parses a standalone name as a value.
    fn parse_with_value<'t, 'i>(
        prefix: &Compare<'_>,
        names: impl Iterator<Item = &'i Compare<'i>>,
        value_infix: &Compare<'_>,
        infix_required: bool,
        arg: &'t OsStr,
    ) -> Option<(Box<str>, Option<Normalized<'t>>)> {
        let string = arg.to_str()?;
        let normalized = Normalized::new(string);
        let (_prefix, name_infix_value) = prefix.strip_as_prefix_n(&normalized)?;
        for name in names {
            // Note: When it is part of the same argument, the parse-function for the inline_value
            // receives a normalized string representation as the argument.
            let Some((name, infix_value)) = name.strip_as_prefix_n(&name_infix_value) else {
                continue;
            };
            if infix_value.as_str().is_empty() {
                // Full argument consumed
                return Some((name.as_str().into(), None));
            }
            // Infix is optional for short names.
            let value = if let Some((_infix, value)) = value_infix.strip_as_prefix_n(&infix_value) {
                value
            } else if infix_required {
                continue;
            } else {
                infix_value
            };

            return Some((name.as_str().into(), Some(value.into_owned())));
        }
        None
    }

    /// Parses a long name as a value.
    pub(crate) fn parse_long_with_value<'t>(
        &self,
        value_infix: &Compare<'_>,
        arg: &'t OsStr,
    ) -> Option<(Box<str>, Option<Normalized<'t>>)> {
        Self::parse_with_value(&self.long_prefix, self.long.iter(), value_infix, true, arg)
    }

    /// Parses a standalone short name as a value.
    pub(crate) fn parse_short_with_value<'t>(
        &self,
        value_infix: &Compare<'_>,
        arg: &'t OsStr,
    ) -> Option<(Box<str>, Option<Normalized<'t>>)> {
        Self::parse_with_value(&self.short_prefix, self.short.iter(), value_infix, false, arg)
    }

    /// Parses the name as a value.
    pub(crate) fn parse_with_value_merge_short_forms(
        &self,
        value_infix: &Compare<'_>,
        arg: &ArgumentInput,
    ) -> Option<ParsedMergedShortName<()>> {
        struct InfixChecked;

        let Name { long_prefix, long: _, short_prefix, short } = self;
        if short.is_empty() {
            return None;
        }
        let (prefix, graphemes): (Normalized<'static>, Vec<Box<str>>) = match arg {
            ArgumentInput::Unchanged(arg) => {
                let string = arg.to_str()?;
                let normalized = Normalized::new(string);
                if long_prefix.strip_as_prefix_n(&normalized).is_some() {
                    return None;
                }
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

        let (remaining_graphemes, name, _infix, inline_value) = graphemes.into_iter().fold(
            (Vec::new(), None, None, Vec::new()),
            |(mut remaining, name, infix, mut inline_value), grapheme| {
                // Note: When it is part of the same argument, the parse-function for the
                // inline_value receives a normalized string representation as the
                // argument.
                if infix.is_some() {
                    inline_value.push(grapheme);
                    (remaining, name, infix, inline_value)
                } else if name.is_some() {
                    if value_infix == grapheme.as_ref() {
                        // Drop the infix grapheme, we only support single-grapheme value_infix with
                        // short names.
                    } else {
                        inline_value.push(grapheme);
                    }
                    (remaining, name, Some(InfixChecked), inline_value)
                } else if contains_str(short, &grapheme) {
                    (remaining, Some(grapheme), infix, inline_value)
                } else {
                    remaining.push(grapheme);
                    (remaining, name, infix, inline_value)
                }
            },
        );

        let remaining = NonEmpty::from_vec(remaining_graphemes).map(|remaining_graphemes| {
            ArgumentInput::AdjustedMergedShortNames(AdjustedMergedShortNames {
                prefix: prefix.into_owned(),
                graphemes: remaining_graphemes,
                suffix: MergedShortNameSuffix::Removed,
            })
        });
        let inline_value =
            (!inline_value.is_empty()).then_some(inline_value.concat().into_boxed_str());

        name.map(|name| ParsedMergedShortName {
            name: name.clone(),
            value: (),
            inline_value,
            remaining,
        })
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
