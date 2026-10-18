#![doc = include_str!("../README.md")]
// Use #[doc(auto_cfg)] (enabled by default) for better documentation of feature-gated Items.
// https://doc.rust-lang.org/rustdoc/unstable-features.html#docauto_cfgdocauto_cfg--truedocauto_cfg--false
#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]

#[doc(no_inline)]
pub use ::nonempty::{NonEmpty, nonempty};

#[macro_export]
/// Crate Name spezifiziert in Cargo.toml.
///
/// ## English
/// Crate name specified in Cargo.toml.
macro_rules! crate_name {
    () => {
        env!("CARGO_PKG_NAME")
    };
}

#[macro_export]
/// Crate Version spezifiziert in Cargo.toml.
///
/// ## English
/// Crate version specified in Cargo.toml.
macro_rules! crate_version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

pub mod arguments;
pub mod description;
pub mod dyn_to_owned;
pub mod language;
pub mod outcome;
pub mod parse;
pub mod unicode;

#[doc(inline)]
// doc(inline) for doc_cfg to work on derive macros (ambiguous name with trait).
pub use crate::{arguments::value::EnumArgument, parse::Parse};
pub use crate::{
    arguments::{Arguments, early_exit::EarlyExit, flag::Flag, value::Value},
    description::{ArgumentInput, Description},
    language::Language,
    outcome::{Error, ParseError, Result},
    parse::ParseArgument,
    unicode::{Case, Compare, Normalized},
};
#[cfg(feature = "german")]
pub use crate::{
    arguments::{argumente::Argumente, early_exit::FrühesBeenden, value::Wert},
    description::Beschreibung,
    language::Sprache,
    outcome::{Ergebnis, Fehler, ParseFehler},
    unicode::{Normalisiert, Vergleich},
};
