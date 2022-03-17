//! Unicode-berücksichtigende String-Funktionen.

use std::{borrow::Cow, convert::AsRef};

use cjk::is_cjkish_codepoint;
use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

/// Ein normalisierter Unicode String.
///
/// Der String ist in
/// [Unicode Normalization Form C](https://docs.rs/unicode-normalization/latest/unicode_normalization/trait.UnicodeNormalization.html#tymethod.nfc),
/// mit standardisierten Variantenselektoren für cjk-Zeichen.
///
/// ## English synonym
/// [Normalized]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(single_use_lifetimes)]
pub struct Normalisiert<'t>(Cow<'t, str>);

impl AsRef<str> for Normalisiert<'_> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'t, S: Into<Cow<'t, str>>> From<S> for Normalisiert<'t> {
    fn from(input: S) -> Self {
        Normalisiert::neu(input)
    }
}

/// A normalized unicode string.
///
/// The String is in
/// [Unicode Normalization Form C](https://docs.rs/unicode-normalization/latest/unicode_normalization/trait.UnicodeNormalization.html#tymethod.nfc),
/// with standardized variation sequences.
///
/// ## Deutsches Synonym
/// [Normalisiert]
pub type Normalized<'t> = Normalisiert<'t>;

impl<'t> Normalisiert<'t> {
    /// Normalisiere einen Unicode-String, sofern er nicht bereits normalisiert ist
    /// ([is_nfc_quick]) oder cjk-Zeichen enthalten sind ([is_cjkish_codepoint]).
    ///
    /// Zuerst werden cjk-Zeichen über [cjk_compat_variants](UnicodeNormalization::cjk_compat_variants)
    /// normalisiert, anschließend wird über [nfc](UnicodeNormalization::nfc) der String in
    /// [Unicode Normalization Form C](https://docs.rs/unicode-normalization/latest/unicode_normalization/trait.UnicodeNormalization.html#tymethod.nfc)
    /// transformiert.
    ///
    /// ## English synonym
    /// [new](Normalized::new)
    #[inline(always)]
    pub fn neu(s: impl Into<Cow<'t, str>>) -> Normalisiert<'t> {
        let cow = s.into();
        let normalisiert = match is_nfc_quick(cow.chars()) {
            IsNormalized::Yes if !cow.chars().any(is_cjkish_codepoint) => cow,
            _ => Cow::Owned(cow.cjk_compat_variants().nfc().collect()),
        };
        Normalisiert(normalisiert)
    }

    /// Normalize a unicode string, unless it is already normalized ([is_nfc_quick]),
    /// or contains any cjk character ([is_cjkish_codepoint]).
    ///
    /// First, cjk characters are normalized with
    /// [cjk_compat_variants](UnicodeNormalization::cjk_compat_variants).
    /// Afterwards, the string is transformed into
    /// [Unicode Normalization Form C](https://docs.rs/unicode-normalization/latest/unicode_normalization/trait.UnicodeNormalization.html#tymethod.nfc)
    /// using [nfc](UnicodeNormalization::nfc).
    ///
    /// ## Deutsches Synonym
    /// [neu](Normalisiert::neu)
    #[inline(always)]
    pub fn new(s: impl Into<Cow<'t, str>>) -> Normalized<'t> {
        Normalisiert::neu(s)
    }

    /// Überprüfe ob zwei Strings nach Unicode Normalisierung identisch sind,
    /// optional [ohne Groß-/Kleinschreibung zu beachten](unicase::eq).
    ///
    /// ## English
    /// Check whether two Strings are identical after unicode normalization,
    /// optionally in a [case-insensitive way](unicase::eq).
    pub fn eq(&self, s: &str, case_sensitive: Case) -> bool {
        let normalisiert = Normalisiert::neu(s);
        match case_sensitive {
            Case::Sensitive => *self == normalisiert,
            Case::Insensitive => unicase::eq(self, &normalisiert),
        }
    }
}

/// Wird Groß-/Kleinschreibung beachtet?
///
/// ## English
/// Are both Strings compared respecting or ignoring case differences?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Case {
    /// Beachte Groß-/Kleinschreibung: `"a" != "A"`
    ///
    /// ## English
    /// Compare respecting case differences: `"a" != "A"`
    Sensitive,

    /// Ignoriere Groß-/Kleinschreibung: `"a" == "A"`
    ///
    /// ## English
    /// Compare ignoring case differences: `"a" == "A"`
    Insensitive,
}

impl From<bool> for Case {
    fn from(input: bool) -> Self {
        if input {
            Case::Sensitive
        } else {
            Case::Insensitive
        }
    }
}

impl From<Case> for bool {
    fn from(input: Case) -> Self {
        input == Case::Sensitive
    }
}

/// Normalisierter Unicode-String, sowie ob dieser unter berücksichtigen von
/// Groß-/Kleinschreibung verglichen werden soll.
///
/// ## English synonym
/// [Compare]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[allow(single_use_lifetimes)]
pub struct Vergleich<'t> {
    pub string: Normalisiert<'t>,
    pub case: Case,
}

macro_rules! impl_vergleich_from {
    ($type: ty) => {
        #[allow(single_use_lifetimes)]
        impl<'t> From<$type> for Vergleich<'t> {
            fn from(input: $type) -> Self {
                Vergleich { string: Normalisiert::neu(input), case: Case::Sensitive }
            }
        }

        #[allow(single_use_lifetimes)]
        impl<'t> From<($type, Case)> for Vergleich<'t> {
            fn from((s, case): ($type, Case)) -> Self {
                Vergleich { string: Normalisiert::neu(s), case }
            }
        }
    };
}

impl_vergleich_from! {String}
impl_vergleich_from! {&'t str}

impl<'t> From<Normalisiert<'t>> for Vergleich<'t> {
    fn from(input: Normalisiert<'t>) -> Self {
        Vergleich { string: input, case: Case::Sensitive }
    }
}

impl<'t> From<(Normalisiert<'t>, Case)> for Vergleich<'t> {
    fn from((string, case): (Normalisiert<'t>, Case)) -> Self {
        Vergleich { string, case }
    }
}

impl AsRef<str> for Vergleich<'_> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.string.as_ref()
    }
}

/// Normalized unicode string, as well as if it should be compared in a case-(in)sensitive way.
///
/// ## Deutsches Synonym
/// [Vergleich]
pub type Compare<'t> = Vergleich<'t>;

impl Vergleich<'_> {
    /// Überprüfe ob zwei Strings nach Unicode Normalisierung identisch sind,
    /// optional [ohne Groß-/Kleinschreibung zu beachten](unicase::eq).
    ///
    /// ## English
    /// Check whether two Strings are identical after unicode normalization,
    /// optionally in a [case-insensitive way](unicase::eq).
    pub fn eq(&self, gesucht: &str) -> bool {
        let Vergleich { string, case } = self;
        string.eq(gesucht, *case)
    }

    /// Versuche einen String vom Anfang des anderen Strings zu entfernen.
    pub(crate) fn strip_als_präfix<'t>(
        &self,
        string: impl Into<Normalisiert<'t>>,
    ) -> Option<Graphemes<'t>> {
        let normalisiert = string.into();
        let lang_graphemes = self.string.as_ref().graphemes(true);
        let lang_länge = lang_graphemes.clone().count();
        let mut string_graphemes = normalisiert.as_ref().graphemes(true);
        let string_präfix: String = string_graphemes.clone().take(lang_länge).collect();
        if self.eq(&string_präfix) {
            // Bei leerem Präfix muss nichts übersprungen werden
            if lang_länge > 0 {
                // Verwende [Iterator::nth] anstelle von [Iterator::skip],
                // damit [Graphemes::as_str] weiterhin verwendet werden kann.
                string_graphemes.nth(lang_länge - 1)?;
            }
            Some(string_graphemes)
        } else {
            None
        }
    }
}
