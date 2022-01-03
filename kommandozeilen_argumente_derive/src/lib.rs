//! Macros für das kommandozeilen_argumente crate.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum, ItemStruct};

/// Erstelle eine `kommandozeilen_argumente`-Methode, um ein Arg<Self> zu erzeugen.
#[proc_macro_attribute]
pub fn kommandozeilen_argumente(attr: TokenStream, mut item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        let fehlermeldung = format!("Kein Argument unterstützt, aber \"{}\" erhalten.", attr);
        let error: TokenStream = quote!(compile_error! {#fehlermeldung }).into();
        item.extend(error);
        return item;
    }
    let item_struct: ItemStruct = parse_macro_input!(item);
    todo!()
}

/// Derive-Macro für das ArgEnum-Trait.
#[proc_macro_derive(ArgEnum)]
pub fn derive_arg_enum(item: TokenStream) -> TokenStream {
    let item_enum: ItemEnum = parse_macro_input!(item);
    todo!()
}
