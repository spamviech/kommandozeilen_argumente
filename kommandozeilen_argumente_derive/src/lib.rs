//! Macros für das kommandozeilen_argumente crate.

use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Fields, Ident, ItemEnum, ItemStruct};

fn base_name() -> Result<Ident, proc_macro_crate::Error> {
    Ok(match crate_name("kommandozeilen_argumente")? {
        FoundCrate::Itself => format_ident!("{}", "crate"),
        FoundCrate::Name(name) => format_ident!("{}", name),
    })
}

macro_rules! compile_error_return {
    ($item: expr, $format_string: tt$(, $($format_args: expr)+)?) => {{
        let fehlermeldung = format!($format_string$(, $($format_args),+)?);
        let compile_error: TokenStream = quote!(compile_error! {#fehlermeldung }).into();
        $item.extend(compile_error);
        return $item;
    }};
}

macro_rules! unwrap_result_or_compile_error {
    ($item: expr, $result: expr) => {
        match $result {
            Ok(wert) => wert,
            Err(fehler) => compile_error_return!($item, "{:?}", fehler),
        }
    };
}

/// Erstelle Methoden `kommandozeilen_argumente`, `parse[_aus_env][_frühes_beenden]`
/// zum parsen aus Kommandozeilen-Argumenten.
#[proc_macro_attribute]
pub fn kommandozeilen_argumente(attr: TokenStream, mut item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        compile_error_return!(item, "Kein Argument unterstützt, aber \"{}\" erhalten.", attr);
    }
    let crate_name = unwrap_result_or_compile_error!(item, base_name());
    let item_struct = parse_macro_input!(item as ItemStruct);
    todo!()
}

/// Derive-Macro für das ArgEnum-Trait.
#[proc_macro_derive(ArgEnum)]
pub fn derive_arg_enum(item: TokenStream) -> TokenStream {
    let mut dummy = TokenStream::new();
    let crate_name = unwrap_result_or_compile_error!(dummy, base_name());
    let item_enum = parse_macro_input!(item as ItemEnum);
    if !item_enum.generics.params.is_empty() {
        compile_error_return!(dummy, "Nur Enums ohne Generics unterstützt.");
    }
    let ident = &item_enum.ident;
    let mut varianten = Vec::new();
    for variant in item_enum.variants {
        if variant.fields != Fields::Unit {
            compile_error_return!(
                dummy,
                "Nur Enums mit Unit-Varianten unterstützt, aber {} hält Daten.",
                variant.ident
            );
        }
        varianten.push(&item_enum.ident);
    }
    let instance = quote!(
        impl #crate_name::ArgEnum for #ident {
            fn varianten() -> Vec<Self> {
                vec![#(#varianten),*]
            }
        }
    );
    instance.into()
}
