//! derive-Macros für das kommandozeilen_argumente crate.

use std::fmt::Display;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod enum_argument;
mod parse;
mod utility;

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
