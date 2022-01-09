//! Macros für das kommandozeilen_argumente crate.

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

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Fields, Ident, ItemEnum, ItemStruct};

fn base_name() -> Ident {
    format_ident!("{}", "kommandozeilen_argumente")
}

macro_rules! compile_error_return {
    ($format_string: tt$(, $($format_args: expr),+$(,)?)?) => {{
        let fehlermeldung = format!($format_string$(, $($format_args),+)?);
        let compile_error = quote!(compile_error! {#fehlermeldung });
        return compile_error.into();
    }};
}
pub(crate) use compile_error_return;

macro_rules! unwrap_result_or_compile_error {
    ($result: expr) => {
        match $result {
            Ok(wert) => wert,
            Err(fehler) => compile_error_return!("{:?}", fehler),
        }
    };
}
pub(crate) use unwrap_result_or_compile_error;

macro_rules! unwrap_option_or_compile_error {
    ($option: expr, $fehler: tt) => {
        match $option {
            Some(wert) => wert,
            None => compile_error_return!("{:?}", $fehler),
        }
    };
}
pub(crate) use unwrap_option_or_compile_error;

mod parse;

/// Implementierung für das derive-Macro des Parse-Traits.
#[proc_macro_derive(Parse, attributes(kommandozeilen_argumente))]
pub fn derive_parse(item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);

    parse::derive_parse(item_struct).into()
}

/// Derive-Macro für das ArgEnum-Trait.
#[proc_macro_derive(ArgEnum)]
pub fn derive_arg_enum(item: TokenStream) -> TokenStream {
    let crate_name = base_name();
    let item_enum = parse_macro_input!(item as ItemEnum);
    if !item_enum.generics.params.is_empty() {
        compile_error_return!("Nur Enums ohne Generics unterstützt.");
    }
    let ident = &item_enum.ident;
    let mut varianten = Vec::new();
    for variant in item_enum.variants {
        if let Fields::Unit = variant.fields {
        } else {
            compile_error_return!(
                "Nur Enums mit Unit-Varianten unterstützt, aber {} hält Daten.",
                variant.ident
            );
        }
        varianten.push(variant.ident);
    }
    let varianten_str: Vec<_> = varianten.iter().map(ToString::to_string).collect();
    let instance = quote!(
        impl #crate_name::ArgEnum for #ident {
            fn varianten() -> Vec<Self> {
                vec![#(Self::#varianten),*]
            }

            fn parse_enum(arg: &std::ffi::OsStr) -> Result<Self, #crate_name::ParseFehler<String>> {
                if let Some(string) = arg.to_str() {
                    #(
                        if #crate_name::unicase_eq(string, #varianten_str) {
                            Ok(Self::#varianten)
                        } else
                    )*
                    {
                        Err(#crate_name::ParseFehler::ParseFehler(
                            format!("Unbekannte Variante: {}", string))
                        )
                    }
                } else {
                    Err(#crate_name::ParseFehler::InvaliderString(arg.to_owned()))
                }
            }
        }
    );
    instance.into()
}
