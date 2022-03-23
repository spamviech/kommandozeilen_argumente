//! Unicode-berücksichtigende String-Funktionen.

use std::{borrow::Cow, convert::AsRef};

use unicode_normalization::{is_nfc_quick, IsNormalized, UnicodeNormalization};
use unicode_segmentation::UnicodeSegmentation;

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
    /// ([is_nfc_quick]) oder bestimmte cjk-Zeichen enthalten sind.
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
            IsNormalized::Yes if !cow.chars().eq(cow.cjk_compat_variants()) => cow,
            _ => Cow::Owned(cow.cjk_compat_variants().nfc().collect()),
        };
        Normalisiert(normalisiert)
    }

    /// Normalize a unicode string, unless it is already normalized ([is_nfc_quick]),
    /// or contains certain cjk characters.
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

    pub(crate) fn neu_borrowed_unchecked(s: &'t str) -> Normalisiert<'t> {
        Normalisiert(Cow::Borrowed(s))
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
    /// Der zu vergleichende String.
    ///
    /// ## English
    /// The string to compare to.
    pub string: Normalisiert<'t>,

    /// Soll der String unter Berücksichtigung von Groß-/Kleinschreibung verglichen werden.
    ///
    /// ## English
    /// Is the comparison case-(in)sensitive?
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
    pub(crate) fn strip_als_präfix<'t>(&self, string: &'t Normalisiert<'t>) -> Option<&'t str> {
        let string_str = string.as_ref();
        let string_länge = string_str.len();
        let mut graphemes_indices = string_str.grapheme_indices(true);
        let mut präfixe = vec![(string_str, string_länge)];
        while let Some((ix, _str)) = graphemes_indices.next_back() {
            präfixe.push((graphemes_indices.as_str(), ix))
        }
        präfixe
            .iter()
            .rev()
            .find(|(präfix, _ix)| self.eq(präfix))
            .map(|(_präfix, ix)| &string_str[*ix..string_länge])
    }
}
