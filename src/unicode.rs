//! Unicode-berücksichtigende String-Funktionen.

use std::borrow::Cow;

use cjk::is_cjkish_codepoint;
use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};

fn nfc_normalize(s: &str) -> Cow<'_, str> {
    match is_nfc_quick(s.chars()) {
        IsNormalized::Yes if !s.chars().any(is_cjkish_codepoint) => Cow::Borrowed(s),
        _ => Cow::Owned(s.nfc().cjk_compat_variants().collect()),
    }
}

/// Überprüfe ob zwei Strings nach Unicode Normalisierung identisch sind,
/// optional ohne Groß-/Kleinschreibung zu beachten.
///
/// Diese Funktion verwendet
/// [Unicode Normalization Form C](https://docs.rs/unicode-normalization/latest/unicode_normalization/trait.UnicodeNormalization.html#tymethod.nfc)
/// um beide Strings zu vergleichen.
///
/// ## English
/// Check whether two Strings are identical after unicode normalization,
/// optionally in a [case-insensitive way](unicase::eq).
///
/// This funktion uses
/// [Unicode Normalization Form C](https://docs.rs/unicode-normalization/latest/unicode_normalization/trait.UnicodeNormalization.html#tymethod.nfc)
/// to compare both strings.
pub fn unicode_eq(a: &str, b: &str, case_sensitive: bool) -> bool {
    let a_nfc = nfc_normalize(a);
    let b_nfc = nfc_normalize(b);
    if case_sensitive {
        a_nfc == b_nfc
    } else {
        unicase::eq(&a_nfc, &b_nfc)
    }
}
