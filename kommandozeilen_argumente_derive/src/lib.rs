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
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{parse_macro_input, Field, Fields, Ident, ItemEnum, ItemStruct};
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

enum Sprache {
    Deutsch,
    Englisch,
}
use Sprache::*;

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
    let mut erstelle_version: Option<fn(TokenStream2, Sprache) -> TokenStream2> = None;
    let mut erstelle_hilfe: Option<fn(TokenStream2) -> TokenStream2> = None;
    let mut sprache = Deutsch;
    let mut invertiere_präfix = None;
    let mut meta_var = None;
    for arg in args {
        match arg.as_str() {
            "deutsch" => sprache = Deutsch,
            "german" => sprache = Deutsch,
            "english" => sprache = Englisch,
            "englisch" => sprache = Englisch,
            "version" => {
                erstelle_version = Some(|item, sprache| {
                    let version_methode = match sprache {
                        Deutsch => "version_deutsch",
                        Englisch => "version_english",
                    };
                    let version_ident = format_ident!("{}", version_methode);
                    quote!(
                        #item.#version_ident(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                    )
                })
            }
            "version_deutsch" => {
                erstelle_version = Some(|item, _sprache| {
                    quote!(
                        #item.version_deutsch(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                    )
                })
            }
            "version_english" => {
                erstelle_version = Some(|item, _sprache| {
                    quote!(
                        #item.version_english(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                    )
                })
            }
            "hilfe" => {
                erstelle_hilfe = Some(|item| {
                    quote!(
                        #item.hilfe(env!("CARGO_PKG_NAME"), Some(env!("CARGO_PKG_VERSION")))
                    )
                })
            }
            "help" => {
                erstelle_hilfe = Some(|item| {
                    quote!(
                        #item.help(env!("CARGO_PKG_NAME"), Some(env!("CARGO_PKG_VERSION")))
                    )
                })
            }
            string => match string.split_once(':') {
                Some((arg_name, wert_string)) => match arg_name.trim() {
                    "invertiere_präfix" | "invert_prefix" => {
                        invertiere_präfix = Some(wert_string.to_owned())
                    }
                    "meta_var" => meta_var = Some(wert_string.to_owned()),
                    trimmed => {
                        compile_error_return!("Benanntes Argument nicht unterstützt: {}", trimmed)
                    }
                },
                _ => compile_error_return!("Argument nicht unterstützt: {}", arg),
            },
        }
    }
    let crate_name = unwrap_result_or_compile_error!(base_name());
    let invertiere_präfix = invertiere_präfix.unwrap_or(match sprache {
        Deutsch => "kein".to_owned(),
        Englisch => "no".to_owned(),
    });
    let meta_var = meta_var.unwrap_or(match sprache {
        Deutsch => "WERT".to_owned(),
        Englisch => "VALUE".to_owned(),
    });
    let mut tuples = Vec::new();
    for field in item_struct.fields {
        let Field { attrs, ident, .. } = field;
        let mut hilfe_lits = Vec::new();
        let lang =
            unwrap_option_or_compile_error!(ident, "Nur benannte Felder unterstützt.").to_string();
        if lang.is_empty() {
            compile_error_return!("Benanntes Feld mit leerem Namen: {}", lang)
        }
        let mut kurz = quote!(None);
        let mut standard = quote!(#crate_name::parse::ParseArgument::standard());
        let mut glätten = false;
        let mut field_invertiere_präfix = quote!(#invertiere_präfix);
        let mut field_meta_var = quote!(#meta_var);
        for attr in attrs {
            if attr.path.is_ident("doc") {
                let args_str = attr.tokens.to_string();
                if let Some(stripped) =
                    args_str.strip_prefix("= \"").and_then(|s| s.strip_suffix('"'))
                {
                    let trimmed = stripped.trim().to_owned();
                    if !trimmed.is_empty() {
                        hilfe_lits.push(trimmed);
                    }
                }
            } else if attr.path.is_ident("kommandozeilen_argumente") {
                let args_str = attr.tokens.to_string();
                if let Some(stripped) = args_str.strip_prefix('(').and_then(|s| s.strip_suffix(')'))
                {
                    let args = stripped.split(',').flat_map(|s| {
                        let trimmed = s.trim();
                        if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed)
                        }
                    });
                    for arg in args {
                        match arg {
                            "glätten" | "flatten" => glätten = true,
                            "benötigt" | "required" => standard = quote!(None),
                            "kurz" | "short" => {
                                if let Some(kurz_str) = lang.graphemes(true).next() {
                                    kurz = quote!(Some(#kurz_str.to_owned()));
                                }
                            }
                            string => match string.split_once(':') {
                                Some((arg_name, wert_string)) => match arg_name.trim() {
                                    "standard" | "default" => {
                                        let ts: TokenStream2 =
                                            unwrap_result_or_compile_error!(wert_string.parse());
                                        standard = quote!(Some(#ts));
                                    }
                                    "kurz" | "short" => {
                                        let trimmed = wert_string.trim();
                                        kurz = quote!(Some(#trimmed.to_owned()))
                                    }
                                    "invertiere_präfix" | "invert_prefix" => {
                                        let trimmed = wert_string.trim();
                                        field_invertiere_präfix = quote!(#trimmed)
                                    }
                                    "meta_var" => {
                                        let trimmed = wert_string.trim();
                                        field_meta_var = quote!(#trimmed)
                                    }
                                    trimmed => compile_error_return!(
                                        "Benanntes Argument nicht unterstützt: {}",
                                        trimmed
                                    ),
                                },
                                _ => compile_error_return!("Argument nicht unterstützt: {}", arg,),
                            },
                        }
                    }
                } else {
                    compile_error_return!("Args nicht in Klammern eingeschlossen: {}", args_str);
                }
            }
        }
        let ident = format_ident!("{}", lang);
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
        let erstelle_args = if glätten {
            quote!(#crate_name::Parse::kommandozeilen_argumente())
        } else {
            quote!({
                let beschreibung = #crate_name::Beschreibung::neu(
                    #lang.to_owned(),
                    #kurz,
                    #hilfe,
                    #standard,
                );
                #crate_name::ParseArgument::argumente(
                    beschreibung,
                    #field_invertiere_präfix,
                    #field_meta_var
                )
            })
        };
        tuples.push((ident, erstelle_args));
    }
    let (idents, erstelle_args): (Vec<_>, Vec<_>) = tuples.into_iter().unzip();
    let kombiniere = quote!(
        #(
            let #idents = #erstelle_args;
        )*
        #crate_name::kombiniere!(|#(#idents),*| Self {#(#idents),*} => #(#idents),*)
    );
    let nach_version = if let Some(version_hinzufügen) = erstelle_version {
        version_hinzufügen(kombiniere, sprache)
    } else {
        kombiniere
    };
    let nach_hilfe = if let Some(hilfe_hinzufügen) = erstelle_hilfe {
        hilfe_hinzufügen(nach_version)
    } else {
        nach_version
    };
    let impl_parse: TokenStream = quote! {
        impl #crate_name::Parse for #item_ty {
            type Fehler = String;

            fn kommandozeilen_argumente() -> #crate_name::Argumente<Self, Self::Fehler> {
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
