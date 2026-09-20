//! Flag arguments.

use std::{
    borrow::Cow,
    convert::identity,
    ffi::OsString,
    fmt::{self, Debug},
};

use nonempty::NonEmpty;

use crate::{
    argumente::{help::Help, ParseMergedShortFormsResult},
    description::{Description, Name},
    dyn_to_owned::{Bool, Show},
    language::Language,
    unicode::Compare,
};

/// A flag argument.
#[must_use]
pub struct Flag<'t, T> {
    /// General description of the argument.
    pub description: Description<'t, T>,

    /// Prefix used to invert the flag argument.
    pub invert_prefix: Compare<'t>,

    /// Infix following the inversion prefix.
    pub invert_infix: Compare<'t>,

    /// Converts the parsed boolean to the argument value.
    pub convert: Cow<'t, dyn Bool<'t, T>>,

    /// Displays an argument value.
    pub display: Cow<'t, dyn Show<'t, T>>,
}

impl<T: Debug> Debug for Flag<'_, T> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Flag")
            .field("description", &self.description)
            .field("invert_prefix", &self.invert_prefix)
            .field("invert_infix", &self.invert_infix)
            .field("convert", &"<closure>")
            .field("display", &"<closure>")
            .finish()
    }
}

impl<'t> Flag<'t, bool> {
    /// Creates a flag argument that can be disabled with a `no` prefix.
    #[inline]
    pub fn new(description: Description<'t, bool>) -> Self {
        Self::new_with_language(description, Language::ENGLISH)
    }

    /// Creates a flag argument that can be disabled with a German `kein` prefix.
    ///
    /// ## Deutsch
    /// Erzeugt ein Flag-Argument, das mit einem `kein`-Präfix deaktiviert werden kann.
    #[inline]
    pub fn neu(beschreibung: crate::Beschreibung<'t, bool>) -> Self {
        Self::neu_mit_sprache(beschreibung, crate::Sprache::DEUTSCH)
    }

    /// Creates a flag argument that can be disabled with the language's configured prefix.
    #[inline]
    pub fn new_with_language(description: Description<'t, bool>, language: Language) -> Self {
        Self {
            description,
            invert_prefix: language.invert_prefix.into(),
            invert_infix: language.invert_infix.into(),
            convert: Cow::Borrowed(&identity),
            display: Cow::Borrowed(&<bool as ToString>::to_string),
        }
    }

    /// Creates a flag argument that can be disabled with the configured German language prefix.
    ///
    /// ## Deutsch
    /// Erzeugt ein Flag-Argument, das mit dem konfigurierten Präfix deaktiviert werden kann.
    #[inline]
    pub fn neu_mit_sprache(
        beschreibung: crate::Beschreibung<'t, bool>,
        sprache: crate::Sprache,
    ) -> Self {
        Self::new_with_language(beschreibung.into(), sprache.into())
    }

}

impl<T> Flag<'_, T> {
    /// Creates the syntax and corresponding help text for this argument.
    #[inline]
    pub fn create_help_text(&self, meta_default: &str) -> Help {
        let Self { description, invert_prefix, invert_infix, convert: _, display } = self;
        let Description { name, help, default } = description;
        let Name { long_prefix, long, short_prefix, short } = name;
        let mut syntax = String::new();
        syntax.push_str(long_prefix.as_str());
        syntax.push('[');
        syntax.push_str(invert_prefix.as_str());
        syntax.push_str(invert_infix.as_str());
        syntax.push(']');
        let NonEmpty { head, tail } = long;
        Name::alternatives_as_regex(head, tail.as_slice(), &mut syntax);
        if let Some((short_head, short_tail)) = short.split_first() {
            syntax.push_str(" | ");
            syntax.push_str(short_prefix.as_str());
            Name::alternatives_as_regex(short_head, short_tail, &mut syntax);
        }
        let help = match (help, default) {
            (None, None) => None,
            (None, Some(default)) => Some(format!("{meta_default}: {}", display(default))),
            (Some(help), None) => Some(String::from(*help)),
            (Some(help), Some(default)) => {
                let mut help_with_default = (*help).to_owned();
                help_with_default.push(' ');
                help_with_default.push_str(meta_default);
                help_with_default.push_str(": ");
                help_with_default.push_str(&display(default));
                Some(help_with_default)
            },
        };
        Help { syntax, help }
    }

    /// Converts this flag's value to a string using its display function.
    #[inline]
    pub fn as_string_flag(&self) -> Flag<'_, String> {
        let Self { description, invert_prefix, invert_infix, convert, display } = self;
        let convert_boxed: Box<dyn '_ + Bool<'_, String>> =
            Box::new(|value: bool| display(&convert(value)));
        Flag {
            description: description.as_ref().convert(&**display),
            invert_prefix: invert_prefix.clone(),
            invert_infix: invert_infix.clone(),
            convert: Cow::Owned(convert_boxed),
            display: Cow::Borrowed(&Clone::clone),
        }
    }
}

impl<T> Flag<'_, T> {
    /// Parses merged short-form arguments.
    ///
    /// Rules to allow merging of short names:
    ///
    /// - All short names in the same string share the same (short) prefix.
    /// - Only short names consisting of a single grapheme participate.
    /// - At most one value argument per block; it must be last.
    /// - Merging of short names must be enabled for this argument.
    #[inline]
    pub fn parse_merged_short_forms<F>(
        &self,
        args: impl Iterator<Item = OsString>,
    ) -> ParseMergedShortFormsResult<'_, T, F> {
        let Self { description: _, invert_prefix: _, invert_infix: _, convert: _, display: _ } =
            self;
        let _ = args;
        todo!();
    }
}
