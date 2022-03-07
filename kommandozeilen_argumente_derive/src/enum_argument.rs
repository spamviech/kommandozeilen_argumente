//! Implementierung für das derive-Macro des EnumArgument-Traits.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, ItemEnum};

use crate::base_name;

macro_rules! compile_error_return {
    ($format_string: tt$(, $($format_args: expr),+)?$(,)?) => {{
        let fehlermeldung = format!($format_string$(, $($format_args),+)?);
        let compile_error = quote!(compile_error! {#fehlermeldung });
        return compile_error.into();
    }};
}

pub(crate) fn derive_enum_argument(item_enum: ItemEnum) -> TokenStream {
    let crate_name = base_name();
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
        impl #crate_name::EnumArgument for #ident {
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
