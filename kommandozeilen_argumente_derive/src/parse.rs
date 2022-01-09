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
    English,
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

fn find_or_len(string: &str, c: char) -> usize {
    string.find(c).unwrap_or_else(|| string.len())
}

#[test]
fn test_split_argumente() {
    let mut args: Vec<&str> = Vec::new();
    let args_str = "(hello(hi), world: [it's, a, big, world!])";
    split_argumente(&mut args, args_str).expect("Argumente sind wohlgeformt");
    assert_eq!(args, vec!["hello(hi)", "world: [it's, a, big, world!]"])
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
    // Argumente getrennt durch Kommas, Unterargumente mit () angegeben, potentiell mit Kommas.
    // Argumente können Listen (angegeben durch [], potentiell mit Kommas) sein.
    // In einem Argument müssen alle Klammern geschlossen sein.
    // Zusätzliche schließende Klammern werden ignoriert.
    // Argumente werden nicht weiter behandelt.
    // Implementiert als Kellerautomat für simple, Kontext-freie Grammatik
    let mut rest = stripped;
    'arg_suche: while !rest.is_empty() {
        let mut suffix_start = 0;
        let mut suffix = rest;
        macro_rules! verkürze_suffix {
            ($sub_start: expr) => {
                suffix_start += $sub_start;
                suffix = string_suffix(suffix, $sub_start);
            };
        }
        macro_rules! arg_speichern {
            ($ix: expr) => {
                let ix = suffix_start + $ix;
                let trimmed = string_präfix(&rest, ix).trim();
                if trimmed.is_empty() {
                    if rest.trim() == "," {
                        // Ignoriere extra Komma am Ende
                        rest = "";
                    } else {
                        return Err(format!("Leeres Argument: {}", args_str));
                    }
                } else {
                    args.push(S::from(trimmed));
                    rest = string_suffix(&rest, ix + 1);
                }
                continue 'arg_suche
            };
        }
        loop {
            let nächstes_komma = find_or_len(suffix, ',');
            let nächste_runde_klammer = find_or_len(suffix, '(');
            let nächste_eckige_klammer = find_or_len(suffix, '[');
            let nächste_klammer = nächste_runde_klammer.min(nächste_eckige_klammer);
            if nächste_klammer < nächstes_komma {
                let schließende_klammer =
                    if nächste_runde_klammer < nächste_eckige_klammer { ')' } else { ']' };
                let mut zu_schließende_klammern = vec![schließende_klammer];
                verkürze_suffix!(nächste_klammer + 1);
                while !zu_schließende_klammern.is_empty() {
                    if suffix.is_empty() {
                        return Err(format!("Nicht geschlossene Klammer: {}", rest));
                    }
                    let öffnende_runde_klammer = find_or_len(suffix, '(');
                    let öffnende_eckige_klammer = find_or_len(suffix, '[');
                    let öffnende_klammer = öffnende_runde_klammer.min(öffnende_eckige_klammer);
                    let erwartete_klammer =
                        zu_schließende_klammern.pop().expect("!zu_schließende_klammern.is_empty()");
                    let schließende_klammer = find_or_len(suffix, erwartete_klammer);
                    if schließende_klammer < öffnende_klammer {
                        verkürze_suffix!(schließende_klammer + 1);
                    } else {
                        zu_schließende_klammern.push(erwartete_klammer);
                        let schließende_klammer =
                            if öffnende_runde_klammer < öffnende_eckige_klammer {
                                ')'
                            } else {
                                ']'
                            };
                        zu_schließende_klammern.push(schließende_klammer);
                        verkürze_suffix!(öffnende_klammer + 1);
                    }
                }
            } else {
                arg_speichern!(nächstes_komma);
            }
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

fn erstelle_version_methode(
    feste_sprache: Option<Sprache>,
    namen: Option<(TokenStream, TokenStream)>,
) -> impl Fn(TokenStream, Sprache) -> TokenStream {
    let version_methode = feste_sprache.map(|sprache| match sprache {
        Deutsch => "version_mit_namen",
        English => "version_with_names",
    });
    let crate_name = base_name();
    let (lang_namen, kurz_namen) = namen.unwrap_or_else(|| (quote!("version"), quote!("v")));
    move |item, sprache| {
        let version_methode = version_methode.unwrap_or_else(|| match sprache {
            Deutsch => "version_mit_namen",
            English => "version_with_names",
        });
        let version_ident = format_ident!("{}", version_methode);
        quote!(
            #item.#version_ident(
                #lang_namen,
                #kurz_namen,
                #crate_name::crate_name!(),
                #crate_name::crate_version!(),
            )
        )
    }
}

fn erstelle_hilfe_methode(
    sprache: Sprache,
    namen: Option<(TokenStream, TokenStream)>,
) -> impl Fn(TokenStream) -> TokenStream {
    let crate_name = base_name();
    let (hilfe_methode, hilfe_ident) = match sprache {
        Deutsch => ("hilfe", "hilfe_mit_namen"),
        English => ("help", "help_with_names"),
    };
    let hilfe_ident = format_ident!("{}", hilfe_ident);
    let kurz_standard = if let Some(kurz) = hilfe_methode.graphemes(true).next() {
        quote!(#kurz)
    } else {
        quote!(None)
    };
    let (lang_namen, kurz_namen) = namen.unwrap_or_else(|| (quote!(#hilfe_methode), kurz_standard));
    move |item| {
        quote!(
            #item.#hilfe_ident(
                #lang_namen,
                #kurz_namen,
                #crate_name::crate_name!(),
                Some(#crate_name::crate_version!()),
            )
        )
    }
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
    let mut erstelle_hilfe: Option<Box<dyn FnOnce(TokenStream) -> TokenStream>> = None;
    let mut sprache = Deutsch;
    let mut invertiere_präfix = None;
    let mut meta_var = None;
    let crate_name = base_name();
    for arg in args {
        match arg.as_str() {
            "deutsch" => sprache = Deutsch,
            "german" => sprache = Deutsch,
            "english" => sprache = English,
            "englisch" => sprache = English,
            "version" => erstelle_version = Some(Box::new(erstelle_version_methode(None, None))),
            "version_deutsch" => {
                erstelle_version = Some(Box::new(erstelle_version_methode(Some(Deutsch), None)))
            }
            "version_english" => {
                erstelle_version = Some(Box::new(erstelle_version_methode(Some(English), None)))
            }
            "hilfe" => erstelle_hilfe = Some(Box::new(erstelle_hilfe_methode(Deutsch, None))),
            "help" => erstelle_hilfe = Some(Box::new(erstelle_hilfe_methode(English, None))),
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
                            erstelle_hilfe = Some(Box::new(erstelle_hilfe_methode(
                                Deutsch,
                                Some((lang_namen, kurz_namen)),
                            )))
                        }
                        "help" => {
                            erstelle_hilfe = Some(Box::new(erstelle_hilfe_methode(
                                English,
                                Some((lang_namen, kurz_namen)),
                            )))
                        }
                        "version" => {
                            erstelle_version = Some(Box::new(erstelle_version_methode(
                                None,
                                Some((lang_namen, kurz_namen)),
                            )))
                        }
                        "version_deutsch" => {
                            erstelle_version = Some(Box::new(erstelle_version_methode(
                                Some(Deutsch),
                                Some((lang_namen, kurz_namen)),
                            )))
                        }
                        "version_english" => {
                            erstelle_version = Some(Box::new(erstelle_version_methode(
                                Some(English),
                                Some((lang_namen, kurz_namen)),
                            )))
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
        English => "no".to_owned(),
    });
    let meta_var = meta_var.unwrap_or(match sprache {
        Deutsch => "WERT".to_owned(),
        English => "VALUE".to_owned(),
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
