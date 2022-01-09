//! Implementierung für das derive-Macro des Parse-Traits.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Field, ItemStruct};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    base_name, compile_error_return, unwrap_option_or_compile_error, unwrap_result_or_compile_error,
};

enum Sprache {
    Deutsch,
    Englisch,
}
use Sprache::*;

fn string_präfix(string: &str, bis: usize) -> &str {
    if bis < string.len() {
        &string[0..bis]
    } else {
        string
    }
}

fn string_suffix(string: &str, von: usize) -> &str {
    if von < string.len() {
        &string[von..]
    } else {
        ""
    }
}

fn split_argumente<'t, S: From<&'t str>>(
    args: &mut Vec<S>,
    args_str: &'t str,
) -> Result<(), String> {
    let stripped = if let Some(("", stripped)) = parse_paren_arg(args_str) {
        stripped
    } else {
        return Err(format!("Args nicht in Klammern eingeschlossen: {}", args_str));
    };
    // Argumente getrennt durch Kommas, Unterargumente mit () angegeben,
    // kann ebenfalls Kommas enthalten.
    // Bisher erste eine Ebene, sonst müssten Klammen gezählt werden.
    // (Kellerautomat für Kontext-freite Grammatik)
    let mut rest = stripped;
    while !rest.is_empty() {
        let rest_länge = rest.len();
        macro_rules! arg_speichern {
            ($ix: expr) => {
                let trimmed = string_präfix(&rest, $ix).trim();
                if trimmed.is_empty() {
                    if $ix + 1 < rest_länge {
                        return Err(format!("Leeres Argument: {}", args_str));
                    } else {
                        break;
                    }
                } else {
                    args.push(S::from(trimmed));
                    rest = string_suffix(&rest, $ix + 1);
                }
            };
        }
        let nächstes_komma = rest.find(',').unwrap_or(rest_länge);
        let nächste_klammer = rest.find('(').unwrap_or(rest_länge);
        if nächste_klammer < nächstes_komma {
            let nach_klammer = string_suffix(&rest, nächste_klammer + 1);
            if let Some(schließende_klammer) = nach_klammer.find(')') {
                let ix = nächste_klammer + schließende_klammer + 1;
                let nächstes_komma = string_suffix(&rest, ix).find(',').unwrap_or(rest_länge);
                arg_speichern!(ix + nächstes_komma);
            } else {
                return Err(format!("Nicht geschlossene Klammer: {}", rest));
            }
        } else {
            arg_speichern!(nächstes_komma);
        }
    }
    Ok(())
}

macro_rules! split_argumente {
    ($args: expr, $stripped: expr) => {
        if let Err(fehler) = split_argumente(&mut $args, $stripped) {
            let compile_error = quote!(compile_error! {#fehler });
            return compile_error;
        }
    };
}

fn parse_paren_arg(string: &str) -> Option<(&str, &str)> {
    string.split_once('(').and_then(|(name, wert)| wert.strip_suffix(')').map(|wert| (name, wert)))
}

enum FeldArgument {
    ArgEnum,
    FromStr,
    Parse,
}

pub(crate) fn derive_parse(item_struct: ItemStruct) -> TokenStream {
    if !item_struct.generics.params.is_empty() {
        compile_error_return!("Nur Structs ohne Generics unterstützt.");
    }
    let item_ty = &item_struct.ident;
    let mut args: Vec<String> = Vec::new();
    for attr in &item_struct.attrs {
        if attr.path.is_ident("kommandozeilen_argumente") {
            let args_str = attr.tokens.to_string();
            split_argumente!(args, &args_str);
        }
    }
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    // let version = env!("CARGO_PKG_VERSION");
    // CARGO_PKG_NAME — The name of your package.
    // CARGO_PKG_VERSION — The full version of your package.
    // CARGO_PKG_AUTHORS — Colon separated list of authors from the manifest of your package.
    // CARGO_PKG_DESCRIPTION — The description from the manifest of your package.
    // CARGO_BIN_NAME — The name of the binary that is currently being compiled (if it is a binary). This name does not include any file extension, such as .exe
    let mut erstelle_version: Option<Box<dyn Fn(TokenStream, Sprache) -> TokenStream>> = None;
    let mut erstelle_hilfe: Option<Box<dyn Fn(TokenStream) -> TokenStream>> = None;
    let mut sprache = Deutsch;
    let mut invertiere_präfix = None;
    let mut meta_var = None;
    let crate_name = base_name();
    for arg in args {
        match arg.as_str() {
            "deutsch" => sprache = Deutsch,
            "german" => sprache = Deutsch,
            "english" => sprache = Englisch,
            "englisch" => sprache = Englisch,
            "version" => {
                erstelle_version = Some(Box::new(|item, sprache| {
                    let version_methode = match sprache {
                        Deutsch => "version_deutsch",
                        Englisch => "version_english",
                    };
                    let version_ident = format_ident!("{}", version_methode);
                    quote!(
                        #item.#version_ident(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                    )
                }))
            }
            "version_deutsch" => {
                erstelle_version = Some(Box::new(|item, _sprache| {
                    quote!(
                        #item.version_deutsch(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                    )
                }))
            }
            "version_english" => {
                erstelle_version = Some(Box::new(|item, _sprache| {
                    quote!(
                        #item.version_english(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                    )
                }))
            }
            "hilfe" => {
                erstelle_hilfe = Some(Box::new(|item| {
                    quote!(
                        #item.hilfe(env!("CARGO_PKG_NAME"), Some(env!("CARGO_PKG_VERSION")))
                    )
                }))
            }
            "help" => {
                erstelle_hilfe = Some(Box::new(|item| {
                    quote!(
                        #item.help(env!("CARGO_PKG_NAME"), Some(env!("CARGO_PKG_VERSION")))
                    )
                }))
            }
            string => {
                if let Some((arg_name, wert_string)) = string.split_once(':') {
                    match arg_name.trim() {
                        "invertiere_präfix" | "invert_prefix" => {
                            invertiere_präfix = Some(wert_string.to_owned())
                        }
                        "meta_var" => meta_var = Some(wert_string.to_owned()),
                        trimmed => {
                            compile_error_return!(
                                "Benanntes Argument nicht unterstützt: {}",
                                trimmed
                            )
                        }
                    }
                } else if let Some((arg_name, namen_str)) = parse_paren_arg(string) {
                    let (lang_namen_str, kurz_namen_str) =
                        namen_str.split_once(';').unwrap_or((namen_str, ""));
                    let mut werte =
                        lang_namen_str.split(',').map(str::trim).filter(|s| !s.is_empty());
                    let (head, tail) = if let Some(head) = werte.next() {
                        (head, werte)
                    } else {
                        compile_error_return!("Kein LangName für {}!", arg_name)
                    };
                    let lang_namen = quote!(
                        #crate_name::NonEmpty {
                            head: #head.to_owned(),
                            tail: vec![#(#tail.to_owned()),*]
                        }
                    );
                    let kurz_namen_iter =
                        kurz_namen_str.split(',').map(str::trim).filter(|s| !s.is_empty());
                    let kurz_namen = quote!(vec![#(#kurz_namen_iter.to_owned()),*]);
                    match arg_name.trim() {
                        "hilfe" => {
                            erstelle_hilfe = Some(Box::new(move |item| {
                                quote!(
                                    #item.hilfe_mit_namen(
                                        #lang_namen,
                                        #kurz_namen,
                                        env!("CARGO_PKG_NAME"),
                                        Some(env!("CARGO_PKG_VERSION"))
                                    )
                                )
                            }));
                        }
                        "help" => {
                            erstelle_hilfe = Some(Box::new(move |item| {
                                quote!(
                                    #item.help_with_names(
                                        #lang_namen,
                                        #kurz_namen,
                                        env!("CARGO_PKG_NAME"),
                                        Some(env!("CARGO_PKG_VERSION"))
                                    )
                                )
                            }));
                        }
                        "version" => {
                            erstelle_version = Some(Box::new(move |item, sprache| {
                                let version_methode = match sprache {
                                    Deutsch => "version_mit_namen",
                                    Englisch => "version_with_names",
                                };
                                let version_ident = format_ident!("{}", version_methode);
                                quote!(
                                    #item.#version_ident(
                                        #lang_namen,
                                        #kurz_namen,
                                        env!("CARGO_PKG_NAME"),
                                        env!("CARGO_PKG_VERSION")
                                    )
                                )
                            }))
                        }
                        "version_deutsch" => {
                            erstelle_version = Some(Box::new(move |item, _sprache| {
                                quote!(
                                    #item.version_mit_namen(
                                        #lang_namen,
                                        #kurz_namen,
                                        env!("CARGO_PKG_NAME"),
                                        env!("CARGO_PKG_VERSION")
                                    )
                                )
                            }))
                        }
                        "version_english" => {
                            erstelle_version = Some(Box::new(move |item, _sprache| {
                                quote!(
                                    #item.version_with_names(
                                        #lang_namen,
                                        #kurz_namen,
                                        env!("CARGO_PKG_NAME"),
                                        env!("CARGO_PKG_VERSION")
                                    )
                                )
                            }))
                        }
                        trimmed => {
                            compile_error_return!(
                                "Benanntes Argument nicht unterstützt: {}",
                                trimmed
                            )
                        }
                    }
                } else {
                    compile_error_return!("Argument nicht unterstützt: {}", arg)
                }
            }
        }
    }
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
        let ident = unwrap_option_or_compile_error!(ident, "Nur benannte Felder unterstützt.");
        let ident_str = ident.to_string();
        if ident_str.is_empty() {
            compile_error_return!("Benanntes Feld mit leerem Namen: {}", ident_str)
        }
        let mut lang = quote!(#ident_str.to_owned());
        let mut kurz = quote!(None);
        let mut standard = quote!(#crate_name::parse::ParseArgument::standard());
        let mut feld_argument = FeldArgument::ArgEnum;
        let mut feld_invertiere_präfix = quote!(#invertiere_präfix);
        let mut feld_meta_var = quote!(#meta_var);
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
                let mut args = Vec::new();
                split_argumente!(args, &args_str);
                for arg in args {
                    match arg {
                        "glätten" | "flatten" => feld_argument = FeldArgument::Parse,
                        "FromStr" => feld_argument = FeldArgument::FromStr,
                        "benötigt" | "required" => standard = quote!(None),
                        "kurz" | "short" => {
                            if let Some(kurz_str) = ident_str.graphemes(true).next() {
                                kurz = quote!(Some(#kurz_str.to_owned()));
                            }
                        }
                        string => {
                            if let Some((arg_name, wert_string)) = string.split_once(':') {
                                match arg_name.trim() {
                                    "standard" | "default" => {
                                        let ts: TokenStream =
                                            unwrap_result_or_compile_error!(wert_string.parse());
                                        standard = quote!(Some(#ts));
                                    }
                                    "kurz" | "short" => {
                                        let trimmed = wert_string.trim();
                                        kurz = quote!(Some(#trimmed.to_owned()))
                                    }
                                    "invertiere_präfix" | "invert_prefix" => {
                                        let trimmed = wert_string.trim();
                                        feld_invertiere_präfix = quote!(#trimmed)
                                    }
                                    "meta_var" => {
                                        let trimmed = wert_string.trim();
                                        feld_meta_var = quote!(#trimmed)
                                    }
                                    trimmed => compile_error_return!(
                                        "Benanntes Argument nicht unterstützt: {}",
                                        trimmed
                                    ),
                                }
                            } else if let Some((arg_name, namen_str)) = parse_paren_arg(string) {
                                let mut werte =
                                    namen_str.split(',').map(str::trim).filter(|s| !s.is_empty());
                                match arg_name.trim() {
                                    "lang" | "long" => {
                                        let (head, tail) = if let Some(head) = werte.next() {
                                            (head, werte)
                                        } else {
                                            compile_error_return!("Kein LangName für {}!", arg_name)
                                        };
                                        lang = quote!(
                                            #crate_name::NonEmpty {
                                                head: #head.to_owned(),
                                                tail: vec![#(#tail.to_owned()),*]
                                            }
                                        );
                                    }
                                    "kurz" | "short" => {
                                        let kurz_namen_iter = namen_str
                                            .split(',')
                                            .map(str::trim)
                                            .filter(|s| !s.is_empty());
                                        kurz = quote!(vec![#(#kurz_namen_iter.to_owned()),*]);
                                    }
                                    trimmed => compile_error_return!(
                                        "Benanntes Argument nicht unterstützt: {}",
                                        trimmed
                                    ),
                                }
                            } else {
                                compile_error_return!("Argument nicht unterstützt: {}", arg,)
                            }
                        }
                    }
                }
            }
        }
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
        let erstelle_args = match feld_argument {
            FeldArgument::ArgEnum => {
                quote!({
                    let beschreibung = #crate_name::Beschreibung::neu(
                        #lang.to_owned(),
                        #kurz,
                        #hilfe,
                        #standard,
                    );
                    #crate_name::ParseArgument::argumente(
                        beschreibung,
                        #feld_invertiere_präfix,
                        #feld_meta_var
                    )
                })
            }
            FeldArgument::FromStr => {
                quote!({
                    let beschreibung = #crate_name::Beschreibung::neu(
                        #lang.to_owned(),
                        #kurz,
                        #hilfe,
                        #standard,
                    );
                    #crate_name::Argumente::wert_from_str_display(
                        beschreibung,
                        #feld_meta_var.to_owned(),
                        None,
                    )
                })
            }
            FeldArgument::Parse => {
                quote!(#crate_name::Parse::kommandozeilen_argumente())
            }
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
    quote! {
        impl #crate_name::Parse for #item_ty {
            type Fehler = String;

            fn kommandozeilen_argumente() -> #crate_name::Argumente<Self, Self::Fehler> {
                #nach_hilfe
            }
        }
    }
}
