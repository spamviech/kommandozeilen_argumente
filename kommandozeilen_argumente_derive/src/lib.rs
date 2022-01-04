//! Macros für das kommandozeilen_argumente crate.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, Field, Fields, Ident, ItemEnum, ItemStruct, Lit, Meta, MetaNameValue,
};
use unicode_segmentation::UnicodeSegmentation;

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

macro_rules! unwrap_option_or_compile_error {
    ($item: expr, $option: expr, $fehler: tt) => {
        match $option {
            Some(wert) => wert,
            None => compile_error_return!($item, "{:?}", $fehler),
        }
    };
}

/// Erstelle Methoden `kommandozeilen_argumente`, `parse[_aus_env][_frühes_beenden]`
/// zum parsen aus Kommandozeilen-Argumenten.
#[proc_macro_attribute]
pub fn kommandozeilen_argumente(args_ts: TokenStream, mut item: TokenStream) -> TokenStream {
    let args_str = args_ts.to_string();
    let args: Vec<_> = args_str.split(',').collect();
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    // let version = env!("CARGO_PKG_VERSION");
    // CARGO_PKG_NAME — The name of your package.
    // CARGO_PKG_VERSION — The full version of your package.
    // CARGO_PKG_AUTHORS — Colon separated list of authors from the manifest of your package.
    // CARGO_PKG_DESCRIPTION — The description from the manifest of your package.
    // CARGO_BIN_NAME — The name of the binary that is currently being compiled (if it is a binary). This name does not include any file extension, such as .exe
    let mut erstelle_version: Option<fn(TokenStream2) -> TokenStream2> = None;
    let mut erstelle_hilfe: Option<fn(TokenStream2, usize) -> TokenStream2> = None;
    let mut name_regex_breite: usize = 20;
    for arg in args {
        match arg {
            "version_deutsch" => {
                erstelle_version = Some(|item| {
                    quote!({
                        let name = env!("CARGO_PKG_NAME");
                        let version = env!("CARGO_PKG_VERSION");
                        #item.version_deutsch(name, version)
                    })
                })
            }
            "version_english" => {
                erstelle_version = Some(|item| {
                    quote!({
                        let name = env!("CARGO_PKG_NAME");
                        let version = env!("CARGO_PKG_VERSION");
                        #item.version_english(name, version)
                    })
                })
            }
            "hilfe" => erstelle_hilfe = Some(|item, breite| quote!(#item.hilfe(name, #breite))),
            "help" => erstelle_hilfe = Some(|item, width| quote!(#item.help(name, #width))),
            string => match string.split_once(':') {
                Some(("name_regex_breite" | "name_regex_width", wert_string)) => {
                    if let Ok(wert) = wert_string.parse() {
                        name_regex_breite = wert;
                    } else {
                        compile_error_return!(item, "Argument nicht unterstützt: {}", arg);
                    }
                }
                _ => compile_error_return!(item, "Argument nicht unterstützt: {}", arg),
            },
        }
    }
    let crate_name = unwrap_result_or_compile_error!(item, base_name());
    let item_clone = item.clone();
    let item_struct = parse_macro_input!(item_clone as ItemStruct);
    if !item_struct.generics.params.is_empty() {
        compile_error_return!(item, "Nur Structs ohne Generics unterstützt.");
    }
    let item_ty = &item_struct.ident;
    let mut lange = Vec::new();
    let mut kurze = Vec::new();
    let mut hilfen = Vec::new();
    let mut typen = Vec::new();
    for field in item_struct.fields {
        let Field { attrs, ident, ty, .. } = field;
        let mut hilfe = Vec::new();
        for attr in attrs {
            match attr.parse_meta() {
                Ok(Meta::NameValue(MetaNameValue {
                    path,
                    eq_token: _,
                    lit: Lit::Str(lit_str),
                })) if path.is_ident("doc") => {
                    hilfe.push(lit_str);
                }
                _ => {}
            }
        }
        let lang = unwrap_option_or_compile_error!(item, ident, "Nur benannte Felder unterstützt.")
            .to_string();
        if lang.is_empty() {
            compile_error_return!(item, "Benanntes Feld mit leerem Namen: {}", lang)
        }
        let kurz = lang.graphemes(true).next().map(str::to_owned);
        lange.push(lang);
        kurze.push(kurz);
        hilfen.push(hilfe);
        typen.push(ty);
    }
    let methoden: TokenStream = quote! {
        impl #item_ty {
            fn kommandozeilen_argumente() -> #crate_name::Arg<Self> {
                todo!()
            }
        }
    }
    .into();
    item.extend(methoden);
    item
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
        varianten.push(variant.ident);
    }
    let varianten_str: Vec<_> = varianten.iter().map(ToString::to_string).collect();
    let instance = quote!(
        impl #crate_name::ArgEnum for #ident {
            fn varianten() -> Vec<Self> {
                vec![#(Self::#varianten),*]
            }

            fn parse_enum(arg: &OsStr) -> Result<Self, OsString> {
                if let Some(string) = arg.to_str() {
                    #(
                        if #crate_name::unicase_eq(string, #varianten_str) {
                            Ok(Self::#varianten)
                        } else
                    )*
                    {
                        Err(arg.to_owned())
                    }
                } else {
                    Err(arg.to_owned())
                }
            }
        }
    );
    instance.into()
}
