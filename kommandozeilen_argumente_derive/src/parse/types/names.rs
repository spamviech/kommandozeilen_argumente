//! Generated argument-name and field-value configuration.

use litrs::StringLit;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use unicode_segmentation::UnicodeSegmentation;

use super::settings::Language;
use crate::utility::{Case, crate_ident, exactly_one};

/// Create a newtype with an identical [`ToTokens`] implementation.
macro_rules! create_newtype {
    ($($name: ident : $type: ty),* $(,)?) => {
        $(
            #[derive(Debug, Clone)]
            pub(in crate::parse) struct $name(pub(in crate::parse) $type);

            impl ToTokens for $name {
                fn to_tokens(&self, tokens: &mut TokenStream) {
                    self.0.to_tokens(tokens)
                }
            }
        )*
    };
}

create_newtype! {
    MetaVar: String,
    DefaultValue: TokenStream,
}

/// Create newtypes for string-like values wrapped in a [`Compare`] in the generated
/// [`TokenStream`].
macro_rules! compare_types {
    ($($name: ident ($sprache_ident: ident)),* $(,)?) => {
        $(
            #[derive(Debug, Clone)]
            pub(in crate::parse) struct $name { pub(in crate::parse) string: Option<String>, pub(in crate::parse) case: Option<Case> }

            impl Default for $name {
                fn default() -> Self {
                    $name { string: None, case: None }
                }
            }

            impl $name {
                pub(in crate::parse) fn token_stream(&self, language: &Language) -> TokenStream {
                    let crate_ident = crate_ident();
                    let string = if let Some(string) = &self.string {
                        quote!(#string)
                    } else {
                        let language_tokens = language.token_stream();
                        quote!(#language_tokens.$sprache_ident)
                    };
                    let case = self.case.unwrap_or_default();
                    quote!(::#crate_ident::unicode::Compare {
                        string: ::#crate_ident::unicode::Normalized::new(#string),
                        case: #case,
                    })
                }
            }
        )*
    };
}

compare_types! {
    LongPrefix(long_prefix),
    ShortPrefix(short_prefix),
    InvertPrefix(invert_prefix),
    InvertInfix(invert_infix),
    ValueInfix(value_infix),
}

/// Long names for an argument.
#[derive(Debug, Default)]
pub(in crate::parse) struct LongNames {
    /// The specified long names.
    pub(in crate::parse) namen: Option<(String, Vec<String>)>,
    /// Whether matching is case-sensitive.
    pub(in crate::parse) case: Option<Case>,
}

/// Specified short names without [`Case`].
#[derive(Debug)]
pub(in crate::parse) enum ShortNamesKind {
    /// No short names.
    None,
    /// Automatically derived short name.
    Auto,
    /// Explicitly specified short names.
    Names(Vec<String>),
}

/// Short names for an argument.
#[derive(Debug)]
pub(in crate::parse) struct ShortNames {
    /// Specified short names without [`Case`].
    pub(in crate::parse) namen: ShortNamesKind,
    /// Whether matching is case-sensitive.
    pub(in crate::parse) case: Option<Case>,
}

impl Default for ShortNames {
    fn default() -> Self {
        ShortNames { namen: ShortNamesKind::None, case: None }
    }
}

impl ShortNames {
    /// Convert to a potentially empty [`Vec`] of fixed [`String`]s.
    fn into_vec(
        self,
        lang_name: &str,
        long_names_case: Option<Case>,
    ) -> (Vec<String>, Option<Case>) {
        match self.namen {
            ShortNamesKind::None => (Vec::new(), self.case),
            ShortNamesKind::Auto => (
                vec![
                    lang_name.graphemes(true).next().expect("Langname ohne Graphemes!").to_owned(),
                ],
                self.case.or(long_names_case),
            ),
            ShortNamesKind::Names(namen) => (namen, self.case),
        }
    }

    /// Create a [`TokenStream`] with a [`vec!`] macro for all short names.
    pub(in crate::parse) fn into_vec_tokens(
        self,
        lang_name: &str,
        long_names_case: Option<Case>,
    ) -> TokenStream {
        let (vec, case) = self.into_vec(lang_name, long_names_case);
        if vec.is_empty() {
            quote!(None::<&str>)
        } else {
            let crate_ident = crate_ident();
            if let Some(case) = case {
                quote!(vec![#(#crate_ident::unicode::Compare {
                    string: ::#crate_ident::unicode::Normalized::new(#vec),
                    case: #case,
                }),*])
            } else {
                quote!(vec![#(#vec),*])
            }
        }
    }
}

/// Return a string literal directly, or otherwise convert the [`TokenStream`] with
/// [`ToString::to_string`].
pub(in crate::parse) fn literal_or_to_string(token_stream: &TokenStream) -> String {
    if let Some(string_lit) = exactly_one(token_stream.clone().into_iter())
        .ok()
        .and_then(|literal| StringLit::try_from(literal).ok())
    {
        string_lit.into_value()
    } else {
        token_stream.to_string()
    }
}
