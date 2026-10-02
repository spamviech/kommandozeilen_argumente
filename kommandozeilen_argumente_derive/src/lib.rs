//! Derive macros for the `kommandozeilen_argumente` crate.
//!
//! ## Deutsch
//! Derive-Makros für das `kommandozeilen_argumente`-Crate.

use std::fmt::Display;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod enum_argument;
mod parse;
mod utility;

// TODO verwende Span
/// Return the value or create a [`compile_error!`] with the error message.
fn unwrap_or_compile_error<Error: Display>(result: Result<TokenStream2, Error>) -> TokenStream {
    let ts = match result {
        Ok(ts) => ts,
        Err(fehler) => {
            let fehlermeldung = fehler.to_string();
            quote!(compile_error! {#fehlermeldung })
        },
    };
    ts.into()
}

/// Derive macro for the [`Parse`](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.Parse.html) trait.
///
/// ## Deutsch
/// Derive-Makro für das [`Parse`](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.Parse.html)-Trait.
#[proc_macro_derive(Parse, attributes(kommandozeilen_argumente))]
pub fn derive_parse(item: TokenStream) -> TokenStream {
    unwrap_or_compile_error(parse::derive_parse(item.into()))
}

/// Derive macro for the [`EnumArgument`](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.EnumArgument.html) trait.
///
/// ## Deutsch
/// Derive-Makro für das [`EnumArgument`](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.EnumArgument.html)-Trait.
#[proc_macro_derive(EnumArgument, attributes(kommandozeilen_argumente))]
pub fn derive_arg_enum(item: TokenStream) -> TokenStream {
    unwrap_or_compile_error(enum_argument::derive_enum_argument(item.into()))
}
