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
    ($format_string: tt$(, $($format_args: expr),+$(,)?)?) => {{
        let fehlermeldung = format!($format_string$(, $($format_args),+)?);
        let compile_error: TokenStream = quote!(compile_error! {#fehlermeldung }).into();
        return compile_error;
    }};
}

macro_rules! unwrap_result_or_compile_error {
    ($result: expr) => {
        match $result {
            Ok(wert) => wert,
            Err(fehler) => compile_error_return!("{:?}", fehler),
        }
    };
}

macro_rules! unwrap_option_or_compile_error {
    ($option: expr, $fehler: tt) => {
        match $option {
            Some(wert) => wert,
            None => compile_error_return!("{:?}", $fehler),
        }
    };
}

/// Erstelle Methoden `kommandozeilen_argumente`, `parse[_aus_env][_frühes_beenden]`
/// zum parsen aus Kommandozeilen-Argumenten.
#[proc_macro_derive(Parse, attributes(kommandozeilen_argumente))]
pub fn kommandozeilen_argumente(item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    if !item_struct.generics.params.is_empty() {
        compile_error_return!("Nur Structs ohne Generics unterstützt.");
    }
    let item_ty = &item_struct.ident;
    let mut args = Vec::new();
    for attr in &item_struct.attrs {
        if attr.path.is_ident("kommandozeilen_argumente") {
            let args_str = attr.tokens.to_string();
            if let Some(stripped) = args_str.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
                args.extend(stripped.split(',').flat_map(|s| {
                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_owned())
                    }
                }));
            } else {
                compile_error_return!("Args nicht in Klammern eingeschlossen: {}", args_str);
            }
        }
    }
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    // let version = env!("CARGO_PKG_VERSION");
    // CARGO_PKG_NAME — The name of your package.
    // CARGO_PKG_VERSION — The full version of your package.
    // CARGO_PKG_AUTHORS — Colon separated list of authors from the manifest of your package.
    // CARGO_PKG_DESCRIPTION — The description from the manifest of your package.
    // CARGO_BIN_NAME — The name of the binary that is currently being compiled (if it is a binary). This name does not include any file extension, such as .exe
    let mut erstelle_version: Option<fn(TokenStream2) -> TokenStream2> = None;
    let mut erstelle_hilfe: Option<fn(TokenStream2, usize) -> TokenStream2> = None;
    let mut name_regex_breite: usize = 40;
    let mut invertiere_prefix = "kein".to_owned();
    let mut meta_var = "Wert".to_owned();
    for arg in args {
        match arg.as_str() {
            "version_deutsch" => {
                erstelle_version = Some(|item| {
                    quote!(
                        #item.version_deutsch(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                    )
                })
            }
            "version_english" => {
                erstelle_version = Some(|item| {
                    quote!(
                        #item.version_english(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                    )
                })
            }
            "hilfe" => {
                erstelle_hilfe = Some(|item, breite| {
                    quote!(
                        #item.hilfe(env!("CARGO_PKG_NAME"), Some(env!("CARGO_PKG_VERSION")), #breite)
                    )
                })
            }
            "help" => {
                erstelle_hilfe = Some(|item, width| {
                    quote!(
                        #item.help(env!("CARGO_PKG_NAME"), Some(env!("CARGO_PKG_VERSION")), #width)
                    )
                })
            }
            string => match string.split_once(':') {
                Some(("name_regex_breite" | "name_regex_width", wert_string)) => {
                    if let Ok(wert) = wert_string.parse() {
                        name_regex_breite = wert;
                    } else {
                        compile_error_return!("Argument nicht unterstützt: {}", arg);
                    }
                }
                Some(("invertiere_prefix" | "invert_prefix", wert_string)) => {
                    invertiere_prefix = wert_string.to_owned();
                }
                Some(("meta_var", wert_string)) => meta_var = wert_string.to_owned(),
                _ => compile_error_return!("Argument nicht unterstützt: {}", arg),
            },
        }
    }
    let crate_name = unwrap_result_or_compile_error!(base_name());
    let mut idents = Vec::new();
    let mut lange = Vec::new();
    let mut kurze = Vec::new();
    let mut hilfen = Vec::new();
    let mut typen = Vec::new();
    for field in item_struct.fields {
        let Field { attrs, ident, ty, .. } = field;
        let mut hilfe_lits = Vec::new();
        for attr in attrs {
            match attr.parse_meta() {
                Ok(Meta::NameValue(MetaNameValue {
                    path,
                    eq_token: _,
                    lit: Lit::Str(lit_str),
                })) if path.is_ident("doc") => {
                    let trimmed = lit_str.value().trim().to_owned();
                    if !trimmed.is_empty() {
                        hilfe_lits.push(trimmed);
                    }
                }
                _ => {}
            }
        }
        let lang =
            unwrap_option_or_compile_error!(ident, "Nur benannte Felder unterstützt.").to_string();
        if lang.is_empty() {
            compile_error_return!("Benanntes Feld mit leerem Namen: {}", lang)
        }
        let kurz = if let Some(kurz) = lang.graphemes(true).next() {
            quote!(Some(#kurz.to_owned()))
        } else {
            quote!(None)
        };
        idents.push(format_ident!("{}", lang));
        lange.push(lang);
        kurze.push(kurz);
        let mut hilfe_string = String::new();
        for teil_string in hilfe_lits {
            if !hilfe_string.is_empty() {
                hilfe_string.push(' ');
            }
            hilfe_string.push_str(&teil_string);
        }
        let hilfe = if hilfe_string.is_empty() {
            quote!(None)
        } else {
            quote!(Some(#hilfe_string.to_owned()))
        };
        hilfen.push(hilfe);
        typen.push(ty);
    }
    // TODO Flag für booleans erzeugen
    // TODO Attribute, z.B. standard, meta_var, Flatten, FromStr-Werte, ...
    let kombiniere = quote!(
        #(
            let beschreibung = #crate_name::Beschreibung {
                lang: #lange.to_owned(),
                kurz: #kurze,
                hilfe: #hilfen,
                standard: None,
            };
            let #idents = #crate_name::parse::ArgumentArt::erstelle_arg(
                beschreibung,
                #invertiere_prefix,
                #meta_var
            );
        )*
        #crate_name::kombiniere!(|#(#idents),*| Self {#(#idents),*} => #(#idents),*)
    );
    let nach_version = if let Some(version_hinzufügen) = erstelle_version {
        version_hinzufügen(kombiniere)
    } else {
        kombiniere
    };
    let nach_hilfe = if let Some(hilfe_hinzufügen) = erstelle_hilfe {
        hilfe_hinzufügen(nach_version, name_regex_breite)
    } else {
        nach_version
    };
    let impl_parse: TokenStream = quote! {
        impl #crate_name::Parse for #item_ty {
            type Fehler = OsString;

            fn kommandozeilen_argumente() -> #crate_name::Arg<Self, Self::Fehler> {
                #nach_hilfe
            }
        }
    }
    .into();
    impl_parse
}

/// Derive-Macro für das ArgEnum-Trait.
#[proc_macro_derive(ArgEnum)]
pub fn derive_arg_enum(item: TokenStream) -> TokenStream {
    let crate_name = unwrap_result_or_compile_error!(base_name());
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
