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
    /// Short names longer than a [`Grapheme`](unicode_segmentation::UnicodeSegmentation::graphemes)
    /// are not supported.
    ///
    /// ## Deutsch
    /// Kurzer Name, wird nach `short_prefix` angegeben.
    /// Bei Flag-Argumenten können Kurz-Namen mit identischen `short_prefix` zusammen angegeben werden,
    /// zum Beispiel "-fgh".
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

impl Name<'_> {
    /// Helper for [`parse_flag_aux`](Name::parse_flag_aux).
    fn parse_merged_short_name<E>(
        prefix: Normalized<'_>,
        short: &[Compare<'_>],
        name_gefunden: impl FnOnce() -> E,
        graphemes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Option<(E, Option<ArgumentInput>)> {
        let (erster_match, andere, suffix) = graphemes.into_iter().fold(
            (None, Vec::new(), MergedShortNameSuffix::Unchanged),
            |(mut erster_match, mut andere, _suffix), grapheme| {
                let suffix;
                if erster_match.is_some() || !contains_str(short, grapheme.as_ref()) {
                    suffix = MergedShortNameSuffix::Unchanged;
                    andere.push(Box::from(grapheme.as_ref()));
                } else {
                    suffix = MergedShortNameSuffix::Removed;
                    erster_match = Some(grapheme);
                }
                (erster_match, andere, suffix)
            },
        );

        if erster_match.is_some() {
            let wert = name_gefunden();
            let angepasstes_argument = NonEmpty::from_vec(andere).map(|remaining| {
                ArgumentInput::AdjustedMergedShortNames(AdjustedMergedShortNames {
                    prefix: prefix.into_owned(),
                    graphemes: remaining,
                    suffix,
                })
            });
            return Some((wert, angepasstes_argument));
        }
        None
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

    /// Helper for [`parse_flag_merge_short_forms`](Name::parse_flag_merge_short_forms) and its variants.
    ///
    /// Returns [`Some`] when a name was found and [`None`] otherwise.
    fn parse_flag_merge_short_forms_aux<E>(
        &self,
        name_gefunden: impl FnOnce() -> E,
        arg: &ArgumentInput,
    ) -> Option<(E, Option<ArgumentInput>)> {
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
                            name_gefunden,
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
                    name_gefunden,
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
    /// Parses the name as a flag.
    ///
    /// Returns [`Some`] when a name was found and [`None`] otherwise.
    #[inline]
    pub(crate) fn parse_flag_merge_short_forms(
        &self,
        arg: &ArgumentInput,
    ) -> Option<(bool, Option<ArgumentInput>)> {
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

    /// Parses the name as a flag that causes an early exit.
    ///
    /// Returns [`Some`] when a name was found and [`None`] otherwise.
    #[inline]
    #[allow(clippy::option_option)]
    pub(crate) fn parse_early_exit_merge_short_forms(
        &self,
        arg: &ArgumentInput,
    ) -> Option<Option<ArgumentInput>> {
        self.parse_flag_merge_short_forms_aux(|| (), arg)
            .map(|((), angepasstes_arg)| angepasstes_arg)
    }

    /// Parses the name as a value.
    #[allow(clippy::option_option)]
    pub(crate) fn parse_with_value<'t>(
        &self,
        value_infix: &Compare<'_>,
        arg: &'t OsStr,
    ) -> Option<Option<Cow<'t, OsStr>>> {
        let Name { long_prefix, long, short_prefix, short } = self;
        let kurz_existiert = !short.is_empty();
        if let Some(string) = arg.to_str() {
            let normalized = Normalized::new(string);
            if let Some((_prefix, lang_str)) = long_prefix.strip_as_prefix_n(&normalized) {
                let suffixe = filter_prefix(long, &lang_str);
                for suffix in suffixe {
                    let suffix_normalized = Normalized::new(suffix);
                    #[allow(clippy::redundant_else)]
                    if suffix.is_empty() {
                        return Some(None);
                    } else if let Some((_infix, wert_graphemes)) =
                        value_infix.strip_as_prefix_n(&suffix_normalized)
                    {
                        let wert_str = wert_graphemes.as_str();
                        let wert_länge = wert_str.len();
                        let wert_cow = match normalized.cow_ref() {
                            Cow::Borrowed(_) => {
                                let string_länge = string.len();
                                // Berechne Index aus suffix-Länge
                                #[allow(clippy::arithmetic_side_effects)]
                                let start_index = string_länge - wert_länge - 1;
                                #[allow(clippy::string_slice, clippy::indexing_slicing)]
                                Cow::Borrowed(string[start_index..string_länge].as_ref())
                            },
                            Cow::Owned(_) => {
                                Cow::Owned(OsString::from(wert_graphemes.as_str().to_owned()))
                            },
                        };
                        return Some(Some(wert_cow));
                    } else {
                        // Suffix ist nicht leer, beginnt aber nicht mit value_infix
                    }
                }
            } else if kurz_existiert {
                if let Some((_prefix, kurz_str)) = short_prefix.strip_as_prefix_n(&normalized) {
                    let mut kurz_graphemes = kurz_str.as_str().graphemes(true);
                    if kurz_graphemes.next().is_some_and(|name| contains_str(short, name)) {
                        let rest = Normalized::new(kurz_graphemes.as_str());
                        let wert_str = if rest.as_str().is_empty() {
                            None
                        } else {
                            let wert_str = value_infix
                                .strip_as_prefix_n(&rest)
                                .map_or_else(|| rest.clone(), |(_infix, suffix)| suffix);
                            let wert_länge = wert_str.as_str().len();
                            Some(match normalized.cow_ref() {
                                Cow::Borrowed(_) => {
                                    let string_länge = string.len();
                                    // Berechne Index aus suffix-Länge
                                    #[allow(clippy::arithmetic_side_effects)]
                                    let start_index = string_länge - wert_länge - 1;
                                    #[allow(clippy::string_slice, clippy::indexing_slicing)]
                                    Cow::Borrowed(string[start_index..string_länge].as_ref())
                                },
                                Cow::Owned(_) => {
                                    Cow::Owned(OsString::from(wert_str.cow().into_owned()))
                                },
                            })
                        };
                        return Some(wert_str);
                    }
                }
            } else {
                // kein match für "{lang_name_präfix}.*" und Argument hat keinen short_names.
            }
        }
        None
    }

    /// Parses the name as a value.
    #[allow(clippy::option_option)]
    pub(crate) fn parse_with_value_merge_short_forms(
        &self,
        arg: &ArgumentInput,
    ) -> Option<Option<ArgumentInput>> {
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
                        let graphemes: Vec<&str> = kurz_suffix.as_str().graphemes(true).collect();

                        if let Some((last, graphemes)) = graphemes.split_last() {
                            if contains_str(short, last) {
                                let boxed_graphemes = graphemes
                                    .iter()
                                    .map(|&grapheme: &&str| -> Box<str> { Box::from(grapheme) });
                                let remainder =
                                    NonEmpty::collect(boxed_graphemes).map(|remaining| {
                                        ArgumentInput::AdjustedMergedShortNames(
                                            AdjustedMergedShortNames {
                                                prefix: prefix.into_owned(),
                                                graphemes: remaining,
                                                suffix: MergedShortNameSuffix::Removed,
                                            },
                                        )
                                    });
                                return Some(remainder);
                            }
                        }
                    }
                }
            },
            ArgumentInput::AdjustedMergedShortNames(AdjustedMergedShortNames {
                prefix,
                graphemes,
                suffix: MergedShortNameSuffix::Unchanged,
            }) if short_prefix.eq(prefix.as_str()) => {
                if contains_str(short, graphemes.last()) {
                    let remainder = graphemes.tail.split_last().map(|(_last, tail)| {
                        let graphemes =
                            NonEmpty { head: graphemes.head.clone(), tail: Vec::from(tail) };
                        ArgumentInput::AdjustedMergedShortNames(AdjustedMergedShortNames {
                            prefix: prefix.clone(),
                            graphemes,
                            suffix: MergedShortNameSuffix::Removed,
                        })
                    });
                    return Some(remainder);
                }
            },
            ArgumentInput::AdjustedMergedShortNames(AdjustedMergedShortNames {
                prefix: _,
                graphemes: _,
                suffix: _,
            }) => {
                // only allow the last grapheme
            },
        }
        None
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

/// Returns whether `searched` is in `collection`.
pub(crate) fn contains_str<'t>(
    collection: impl IntoIterator<Item = &'t Compare<'t>>,
    searched: &str,
) -> bool {
    collection.into_iter().any(|target| target.eq(searched))
}

/// Returns all strings in `collection` which start with `input`.
pub(crate) fn filter_prefix<'t>(
    collection: impl 't + IntoIterator<Item = &'t Compare<'t>>,
    input: &'t Normalized<'t>,
) -> impl 't + Iterator<Item = &'t str> {
    collection
        .into_iter()
        .filter_map(|target| target.strip_as_prefix(input).map(|(_, suffix)| suffix))
}

/// One or more long names.
pub trait LongNames<'t> {
    /// Converts to a non-empty collection of comparisons.
    fn long_names(self) -> NonEmpty<Compare<'t>>;
}

macro_rules! impl_long_names {
    ($type: ty) => {
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
    ($type: ty) => {
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
