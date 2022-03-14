//! Unicode-berücksichtigende String-Funktionen.

use std::borrow::Cow;

use cjk::is_cjkish_codepoint;
use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};

/// Konvertiere einen String in
/// [Unicode Normalization Form C](https://docs.rs/unicode-normalization/latest/unicode_normalization/trait.UnicodeNormalization.html#tymethod.nfc)
/// und ersetze cjk-chars mit einer Normalform.
///
/// [cjk_compat_variants]:
/// A transformation which replaces CJK Compatibility Ideograph codepoints with normal forms using Standardized Variation Sequences. This is not part of the canonical or compatibility decomposition algorithms, but performing it before those algorithms produces normalized output which better preserves the intent of the original text.
///
/// Note that many systems today ignore variation selectors, so these may not immediately help text display as intended, but they at least preserve the information in a standardized form, giving implementations the option to recognize them.
fn nfc_normalize(s: &str) -> Cow<'_, str> {
    match is_nfc_quick(s.chars()) {
        IsNormalized::Yes if !s.chars().any(is_cjkish_codepoint) => Cow::Borrowed(s),
        _ => Cow::Owned(s.cjk_compat_variants().nfc().collect()),
    }
}

/// Wie [nfc_normalize], aber verändere den Inhalt einer [Cow],
/// sofern diese nicht bereits in Normalform ohne cjk-chars ist.
fn nfc_normalize_cow(cow: &mut Cow<'_, str>) {
    // TODO eigener abstrakter Typ, der sich merkt ob er bereits normalisiert wurde
    match is_nfc_quick(cow.chars()) {
        IsNormalized::Yes if !cow.chars().any(is_cjkish_codepoint) => {},
        _ => *cow = Cow::Owned(cow.cjk_compat_variants().nfc().collect()),
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

/// Wie [unicode_eq], aber verändere den Inhalt einer [Cow],
/// sofern diese nicht bereits in Normalform ohne cjk-chars ist.
pub fn unicode_eq_cow(a_cow: &mut Cow<'_, str>, b: &str, case_sensitive: bool) -> bool {
    nfc_normalize_cow(a_cow);
    let b_nfc = nfc_normalize(b);
    if case_sensitive {
        *a_cow == b_nfc
    } else {
        unicase::eq(a_cow, &b_nfc)
    }
}
