//! Parsen von Kommandozeilen-Argumenten, inklusiver automatisch generierter (deutscher) Hilfe.

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
    // missing_docs,
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

pub use kommandozeilen_argumente_derive::{kommandozeilen_argumente, ArgEnum};
pub use nonempty::NonEmpty;
#[cfg(feature = "derive")]
pub use unicase::eq as unicase_eq;
pub use void::Void;

pub mod arg;
pub mod beschreibung;
pub mod ergebnis;
#[cfg(test)]
mod test;

pub use self::{
    arg::{Arg, ArgEnum},
    beschreibung::ArgBeschreibung,
    ergebnis::{ParseErgebnis, ParseFehler},
};
