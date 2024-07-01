#![doc = include_str!("../LIESMICH.md")]
// Verwende doc_cfg für bessere Dokumentation von feature-gated derive Macros.
#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]

#[doc(no_inline)]
pub use ::nonempty::{nonempty, NonEmpty};

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

pub mod argumente;
pub mod beschreibung;
pub mod dyn_to_owned;
pub mod ergebnis;
pub mod parse;
pub mod sprache;
pub mod unicode;

#[doc(inline)]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use self::{
    argumente::{
        flag::Flag,
        frühes_beenden::{EarlyExit, FrühesBeenden},
        wert::{EnumArgument, Value, Wert},
        Argumente, Arguments,
    },
    beschreibung::{ArgumentInput, Beschreibung, Description},
    ergebnis::{Ergebnis, Error, Fehler, ParseError, ParseFehler, Result},
    parse::{Parse, ParseArgument},
    sprache::{Language, Sprache},
    unicode::{Case, Compare, Normalisiert, Normalized, Vergleich},
};
