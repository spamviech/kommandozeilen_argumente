//! derive-Macros für das kommandozeilen_argumente crate.

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

use std::fmt::Display;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod enum_argument;
mod parse;
mod split_argumente;

fn unwrap_or_compile_error<Fehler: Display>(result: Result<TokenStream2, Fehler>) -> TokenStream {
    let ts = match result {
        Ok(ts) => ts,
        Err(fehler) => {
            let fehlermeldung = fehler.to_string();
            quote!(compile_error! {#fehlermeldung })
        },
    };
    ts.into()
}

/// Derive-Macro für das [Parse](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.Parse.html)-Traits.
///
/// ## English
/// Derive macro for the [Parse](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.Parse.html) trait.
#[proc_macro_derive(Parse, attributes(kommandozeilen_argumente))]
pub fn derive_parse(item: TokenStream) -> TokenStream {
    unwrap_or_compile_error(parse::derive_parse(item.into()))
}

/// Derive-Macro für das [EnumArgument](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.EnumArgument.html)-Trait.
///
/// ## English
/// Derive macro for the [EnumArgument](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.EnumArgument.html) trait.
#[proc_macro_derive(EnumArgument, attributes(kommandozeilen_argumente))]
pub fn derive_arg_enum(item: TokenStream) -> TokenStream {
    unwrap_or_compile_error(enum_argument::derive_enum_argument(item.into()))
}
