#![doc = include_str!("../README.md")]
// Verwende doc_cfg für bessere Dokumentation von feature-gated derive Macros.
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
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use self::{
    arguments::{
        Arguments,
        early_exit::EarlyExit,
        flag::Flag,
        value::{EnumArgument, Value},
    },
    description::{ArgumentInput, Description},
    language::Language,
    outcome::{Error, ParseError, Result},
    parse::{Parse, ParseArgument},
    unicode::{Case, Compare, Normalized},
};
#[cfg(feature = "german")]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "german")))]
pub use self::{
    arguments::{argumente::Argumente, early_exit::FrühesBeenden, value::Wert},
    description::Beschreibung,
    language::Sprache,
    outcome::{Ergebnis, Fehler, ParseFehler},
    unicode::{Normalisiert, Vergleich},
};
