//! Description of a command-line argument.
//!
//! ## Deutsch
//! Beschreibung eines Arguments.

mod input;
mod name;
mod names;

pub use input::{AdjustedMergedShortNames, ArgumentInput, ArgumentInputRef, MergedShortNameSuffix};
pub use name::Name;
pub use names::{LongNames, ShortNames};

#[cfg(feature = "german")]
use crate::language::Sprache;
use crate::{language::Language, unicode::Compare};

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
#[cfg(feature = "german")]
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

#[cfg(feature = "german")]
impl<'t, T> From<Description<'t, T>> for Beschreibung<'t, T> {
    #[inline]
    fn from(Description { name, help, default }: Description<'t, T>) -> Self {
        Self { name, hilfe: help, standard: default }
    }
}

#[cfg(feature = "german")]
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

#[cfg(feature = "german")]
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

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use nonempty::nonempty;

    use super::{AdjustedMergedShortNames, ArgumentInput, Description, MergedShortNameSuffix};
    use crate::unicode::Normalized;

    fn description(short: impl super::ShortNames<'static>) -> Description<'static, ()> {
        Description::new("--", "café", "-", short, None, None)
    }

    #[test]
    fn matches_canonically_equivalent_long_and_short_names() {
        let description = description("é");
        assert!(description.name.parse_early_exit(&OsString::from("--cafe\u{301}")));
        assert!(description.name.parse_early_exit(&OsString::from("-e\u{301}")));
    }

    #[test]
    fn merged_short_names_preserve_multiscalar_graphemes() {
        let description = description(vec!["e\u{301}", "x"]);
        let parsed = description
            .name
            .parse_flag_merge_short_forms(&ArgumentInput::Unchanged(OsString::from("-e\u{301}x")))
            .expect("recognized multi-scalar grapheme");
        assert_eq!(parsed.name.as_ref(), "é");
        assert_eq!(
            parsed.remaining,
            Some(ArgumentInput::AdjustedMergedShortNames(AdjustedMergedShortNames {
                prefix: Normalized::new("-").into_owned(),
                graphemes: nonempty![Box::from("x")],
                suffix: MergedShortNameSuffix::Unchanged,
            }))
        );
    }

    #[test]
    fn multigrapheme_short_name_is_standalone_only() {
        let description = description("ab");
        assert!(description.name.parse_early_exit(&OsString::from("-ab")));
        assert!(
            description
                .name
                .parse_flag_merge_short_forms(&ArgumentInput::Unchanged(OsString::from("-ab")))
                .is_none()
        );
    }

    #[test]
    fn overlapping_long_and_short_prefixes_do_not_reinterpret_long_forms_as_merged_blocks() {
        let description: Description<'_, ()> = Description::new("--", "help", "-", "h", None, None);
        assert!(
            description
                .name
                .parse_flag_merge_short_forms(&ArgumentInput::Unchanged(OsString::from("--help")))
                .is_none()
        );
    }

    #[test]
    fn identical_windows_style_prefixes_do_not_reinterpret_long_forms_as_merged_blocks() {
        let description: Description<'_, ()> = Description::new("/", "help", "/", "h", None, None);
        assert!(description.name.parse_early_exit(&OsString::from("/help")));
        assert!(
            description
                .name
                .parse_flag_merge_short_forms(&ArgumentInput::Unchanged(OsString::from("/help")))
                .is_none()
        );
    }
}
