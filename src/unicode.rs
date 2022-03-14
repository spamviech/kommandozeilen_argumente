//! Unicode-berücksichtigende String-Funktionen.

use std::borrow::Cow;

use cjk::is_cjkish_codepoint;
use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};

/// Ein normalisierter Unicode String.
///
/// Der String ist in
/// [Unicode Normalization Form C](https://docs.rs/unicode-normalization/latest/unicode_normalization/trait.UnicodeNormalization.html#tymethod.nfc),
/// mit standardisierten Variantenselektoren für cjk-Zeichen.
///
/// ## English synonym
/// [Normalized]
#[derive(Debug, PartialEq, Eq)]
#[allow(single_use_lifetimes)]
pub struct Normalisiert<'t>(Cow<'t, str>);

impl AsRef<str> for Normalisiert<'_> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
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
    pub fn neu(s: &'t str) -> Normalisiert<'t> {
        let cow = match is_nfc_quick(s.chars()) {
            IsNormalized::Yes if !s.chars().any(is_cjkish_codepoint) => Cow::Borrowed(s),
            _ => Cow::Owned(s.cjk_compat_variants().nfc().collect()),
        };
        Normalisiert(cow)
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
    pub fn new(s: &'t str) -> Normalized<'t> {
        Normalisiert::neu(s)
    }

    /// Überprüfe ob zwei Strings nach Unicode Normalisierung identisch sind,
    /// optional ohne Groß-/Kleinschreibung zu beachten.
    ///
    /// ## English
    /// Check whether two Strings are identical after unicode normalization,
    /// optionally in a [case-insensitive way](unicase::eq).
    pub fn eq(&self, s: &str, case_sensitive: bool) -> bool {
        let normalisiert = Normalisiert::neu(s);
        if case_sensitive {
            *self == normalisiert
        } else {
            unicase::eq(self, &normalisiert)
        }
    }
}
