//! Parsen von Kommandozeilen-Argumenten, inklusiver automatisch generierter (deutscher) Hilfe.
// TODO more glorious documentation, including examples; both german and english
// should be possible with `#![doc = include_str!(documentation.txt)]`
// maybe even `#![doc = include_str!(README.md)]`

// Enable all warnings except box_pointers, non_ascii_idents, unstable_features
#![warn(
    absolute_paths_not_starting_with_crate,
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    noop_method_call,
    pointer_structural_match,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]

#[doc(no_inline)]
pub use nonempty::NonEmpty;

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
pub mod ergebnis;
pub mod parse;
pub mod sprache;
pub mod unicode;

#[doc(inline)]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use self::{
    argumente::{wert::EnumArgument, Argumente, Arguments},
    beschreibung::{Beschreibung, Configuration, Description, Konfiguration},
    ergebnis::{Ergebnis, Error, Fehler, ParseError, ParseFehler, Result},
    parse::{Parse, ParseArgument},
    sprache::{Language, Sprache},
    unicode::{Normalisiert, Normalized},
};
