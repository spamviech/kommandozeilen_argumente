//! Implementierung für das derive-Macro des Parse-Traits.

use std::{
    fmt::{self, Display, Formatter},
    iter,
};

use proc_macro2::{Delimiter, Punct, Spacing, TokenStream, TokenTree};
use quote::quote;
use syn::{Field, ItemStruct};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    base_name, compile_error_return, unwrap_option_or_compile_error, unwrap_result_or_compile_error,
};

#[derive(Debug, Clone)]
enum Sprache {
    Deutsch,
    English,
    TokenStream(TokenStream),
}
use Sprache::{Deutsch, English};

impl Sprache {
    fn token_stream(self) -> TokenStream {
        use Sprache::*;
        let crate_name = base_name();
        match self {
            Deutsch => quote!(#crate_name::Sprache::DEUTSCH),
            English => quote!(#crate_name::Sprache::ENGLISH),
            TokenStream(ts) => ts,
        }
    }
}

enum GenauEinesFehler<T, I> {
    Leer,
    MehrAlsEins { erstes: Option<T>, zweites: Option<T>, rest: I },
}

impl<T, I: Iterator<Item = T>> Iterator for GenauEinesFehler<T, I> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        use GenauEinesFehler::*;
        match self {
            Leer => None,
            MehrAlsEins { erstes, zweites, rest } => {
                erstes.take().or_else(|| zweites.take()).or_else(|| rest.next())
            },
        }
    }
}

fn genau_eines<T, I: Iterator<Item = T>>(mut iter: I) -> Result<T, GenauEinesFehler<T, I>> {
    let erstes = iter.next().ok_or(GenauEinesFehler::Leer)?;
    if let Some(zweites) = iter.next() {
        Err(GenauEinesFehler::MehrAlsEins {
            erstes: Some(erstes),
            zweites: Some(zweites),
            rest: iter,
        })
    } else {
        Ok(erstes)
    }
}

#[inline(always)]
fn punct_is_char(punct: &Punct, c: char) -> bool {
    punct.as_char() == c && punct.spacing() == Spacing::Alone
}

#[derive(Debug)]
enum ArgumentWert {
    KeinWert,
    Unterargument(Vec<Argument>),
    Liste(Vec<TokenStream>),
    Stream(TokenStream),
}

impl Display for ArgumentWert {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentWert::*;
        match self {
            KeinWert => Ok(()),
            Unterargument(args) => write_liste(f, "(", args, ")"),
            Liste(tts) => write_liste(f, "[", tts.clone(), "]"),
            Stream(ts) => write!(f, "{ts}"),
        }
    }
}

#[derive(Debug)]
struct Argument {
    name: String,
    wert: ArgumentWert,
}

fn write_liste<T: Display>(
    f: &mut Formatter<'_>,
    open: &str,
    list: impl IntoIterator<Item = T>,
    close: &str,
) -> fmt::Result {
    f.write_str(open)?;
    let mut first = true;
    for elem in list {
        if first {
            first = false
        } else {
            write!(f, ", ")?;
        }
        write!(f, "{elem}")?;
    }
    f.write_str(close)?;
    Ok(())
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Argument { name, wert } = self;
        f.write_str(name)?;
        use ArgumentWert::*;
        match wert {
            KeinWert => Ok(()),
            Unterargument(args) => write_liste(f, "(", args, ")"),
            Liste(tts) => write_liste(f, ": [", tts.clone(), "]"),
            Stream(ts) => write!(f, ": {ts}"),
        }
    }
}

#[derive(Debug)]
enum Fehler {
    NichtInKlammer {
        parent: Vec<String>,
        ts: TokenStream,
    },
    LeeresArgument {
        parent: Vec<String>,
        ts: TokenStream,
    },
    InvaliderArgumentName {
        parent: Vec<String>,
        tt: TokenTree,
    },
    InvaliderArgumentWert {
        parent: Vec<String>,
        name: String,
        doppelpunkt: bool,
        wert: TokenStream,
    },
}

impl Display for Fehler {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fn write_parent(
            f: &mut Formatter<'_>,
            parent: &[String],
            präposition: &str,
        ) -> fmt::Result {
            if !parent.is_empty() {
                write!(f, " {präposition} ")?;
                let mut first = true;
                for name in parent {
                    if first {
                        first = false
                    } else {
                        write!(f, "::")?;
                    }
                    write!(f, "{name}")?;
                }
            }
            Ok(())
        }
        use Fehler::*;
        match self {
            NichtInKlammer { parent, ts } => {
                write!(f, "Argumente")?;
                write_parent(f, parent, "für")?;
                write!(f, " nicht in Klammern eingeschlossen: {ts}")
            },
            LeeresArgument { parent, ts } => {
                write!(f, "Leeres Argument")?;
                write_parent(f, parent, "für")?;
                write!(f, ": {ts}")
            },
            InvaliderArgumentName { parent, tt } => {
                write!(f, "Invalider Name für ein Argument")?;
                write_parent(f, parent, "von")?;
                write!(f, ": {tt}")
            },
            InvaliderArgumentWert { parent, name, doppelpunkt, wert } => {
                write!(f, "Invalider Wert für Argument {name}")?;
                write_parent(f, parent, "von")?;
                write!(f, "!\n{name} ")?;
                if *doppelpunkt {
                    write!(f, ": ")?;
                }
                write!(f, "{wert}")
            },
        }
    }
}

#[test]
fn test_split_argumente() {
    let mut args: Vec<Argument> = Vec::new();
    let args_ts =
        "(hello(hi), world: [it's, a, big, world!])".parse().expect("Valider TokenStream");
    let world_wert =
        "[it's, a, big, world!]".parse::<TokenStream>().expect("world_wert").to_string();
    let world_string = format!("world: {world_wert}");
    split_klammer_argumente(Vec::new(), &mut args, args_ts).expect("Argumente sind wohlgeformt");
    let args_str: Vec<_> = args.iter().map(ToString::to_string).collect();
    assert_eq!(args_str, vec!["hello(hi)", &world_string])
}

fn tt_is_not_comma(tt: &TokenTree) -> bool {
    if let TokenTree::Punct(punct) = tt {
        !punct_is_char(punct, ',')
    } else {
        true
    }
}

/// Argumente getrennt durch Kommas, Unterargumente mit () angegeben, potentiell mit Kommas, z.B. help.
/// Argumente können Werte haben, getrennt durch `:`.
/// Wert-Argumente können Listen (angegeben durch [], potentiell mit Kommas) sein.
/// Argumente werden nicht weiter behandelt.
fn split_argumente(
    parent: Vec<String>,
    args: &mut Vec<Argument>,
    args_ts: TokenStream,
) -> Result<(), Fehler> {
    let mut iter = args_ts.into_iter().peekable();
    let iter_mut_ref = iter.by_ref();
    while iter_mut_ref.peek().is_some() {
        let mut arg_iter = iter_mut_ref.take_while(tt_is_not_comma).peekable();
        // Name extrahieren
        let name = match arg_iter.next() {
            Some(TokenTree::Ident(ident)) => ident.to_string(),
            Some(tt) => return Err(Fehler::InvaliderArgumentName { parent, tt }),
            None => return Err(Fehler::LeeresArgument { parent, ts: iter_mut_ref.collect() }),
        };
        // Wert bestimmen
        let wert = match arg_iter.next() {
            None => ArgumentWert::KeinWert,
            Some(TokenTree::Punct(punct)) if punct_is_char(&punct, ':') => {
                match genau_eines(arg_iter) {
                    Ok(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                        let mut iter = group.stream().into_iter();
                        let mut acc = Vec::new();
                        let mut current = TokenStream::new();
                        while let Some(tt) = iter.next() {
                            match tt {
                                TokenTree::Punct(punct) if punct_is_char(&punct, ',') => {
                                    acc.push(current);
                                    current = TokenStream::new();
                                },
                                _ => current.extend(iter::once(tt)),
                            }
                        }
                        ArgumentWert::Liste(acc)
                    },
                    Ok(tt) => ArgumentWert::Stream(tt.into()),
                    Err(fehler) => ArgumentWert::Stream(fehler.collect()),
                }
            },
            Some(TokenTree::Group(group))
                if group.delimiter() == Delimiter::Parenthesis && arg_iter.peek().is_none() =>
            {
                let mut sub_parent = parent.clone();
                sub_parent.push(name.clone());
                let mut sub_args = Vec::new();
                split_argumente(sub_parent, &mut sub_args, group.stream())?;
                ArgumentWert::Unterargument(sub_args)
            },
            Some(erstes) => {
                return Err(Fehler::InvaliderArgumentWert {
                    parent,
                    name,
                    doppelpunkt: false,
                    wert: iter::once(erstes).chain(arg_iter).collect(),
                })
            },
        };
        // Argument hinzufügen
        args.push(Argument { name, wert });
    }
    Ok(())
}

fn split_klammer_argumente(
    parent: Vec<String>,
    args: &mut Vec<Argument>,
    args_ts: TokenStream,
) -> Result<(), Fehler> {
    let group = match genau_eines(args_ts.into_iter()) {
        Ok(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => group,
        Ok(tt) => return Err(Fehler::NichtInKlammer { parent, ts: tt.into() }),
        Err(fehler) => return Err(Fehler::NichtInKlammer { parent, ts: fehler.collect() }),
    };
    split_argumente(parent, args, group.stream())
}

macro_rules! split_klammer_argumente {
    ($args: expr, $ts: expr) => {
        if let Err(fehler) = split_klammer_argumente(Vec::new(), &mut $args, $ts) {
            let fehler_str = fehler.to_string();
            let compile_error = quote!(compile_error! {#fehler_str});
            return compile_error;
        }
    };
}

enum FeldArgument {
    EnumArgument,
    FromStr,
    Parse,
}

#[derive(Debug)]
enum KurzNamen {
    Keiner,
    Auto,
    Namen(Vec<String>),
}

impl KurzNamen {
    fn to_vec(self, lang_name: &str) -> Vec<String> {
        match self {
            KurzNamen::Keiner => Vec::new(),
            KurzNamen::Auto => {
                vec![lang_name.graphemes(true).next().expect("Langname ohne Graphemes!").to_owned()]
            },
            KurzNamen::Namen(namen) => namen,
        }
    }
}

fn erstelle_version_methode(
    feste_sprache: Option<Sprache>,
    namen: Option<(TokenStream, TokenStream)>,
) -> impl FnOnce(TokenStream, Sprache) -> TokenStream {
    let crate_name = base_name();
    move |item, sprache| {
        let sprache_ts = feste_sprache.unwrap_or(sprache).token_stream();
        let (lang_namen, kurz_namen) = namen.unwrap_or_else(|| {
            (quote!(#sprache_ts.version_lang), quote!(#sprache_ts.version_kurz))
        });
        quote!(
            #item.version_mit_namen_und_sprache(
                #lang_namen,
                #kurz_namen,
                #crate_name::crate_name!(),
                #crate_name::crate_version!(),
                #sprache_ts,
            )
        )
    }
}

fn erstelle_hilfe_methode(
    sprache: Sprache,
    namen: Option<(TokenStream, TokenStream)>,
) -> impl Fn(TokenStream) -> TokenStream {
    let crate_name = base_name();
    let sprache_ts = sprache.token_stream();
    let lang_standard = quote!(#sprache_ts.hilfe_lang);
    let kurz_standard = quote!(#sprache_ts.hilfe_kurz);
    let (lang_namen, kurz_namen) = namen.unwrap_or_else(|| (lang_standard, kurz_standard));
    move |item| {
        quote!(
            #item.hilfe_mit_namen_und_sprache(
                #lang_namen,
                #kurz_namen,
                #crate_name::crate_name!(),
                Some(#crate_name::crate_version!()),
                #sprache_ts
            )
        )
    }
}

fn parse_sprache(ts: TokenStream) -> Sprache {
    match genau_eines(ts.into_iter()) {
        Ok(TokenTree::Ident(ident)) => match ident.to_string().as_str() {
            "deutsch" | "german" => Deutsch,
            "englisch" | "english" => English,
            _ => Sprache::TokenStream(TokenTree::Ident(ident).into()),
        },
        Ok(tt) => Sprache::TokenStream(tt.into()),
        Err(fehler) => Sprache::TokenStream(fehler.collect()),
    }
}

fn parse_wert_arg(
    arg_name: &str,
    sub_args: Vec<Argument>,
    mut setze_sprache: impl FnMut(Sprache, &str) -> Result<(), String>,
    setze_lang_namen: impl FnOnce(TokenStream),
    setze_kurz_namen: impl FnOnce(TokenStream),
    mut setze_meta_var: impl FnMut(&str, &str, &str) -> Result<(), String>,
    mut setze_invertiere_präfix: impl FnMut(&str, &str, &str) -> Result<(), String>,
    mut setze_standard: impl FnMut(Option<TokenStream>, &str, &str) -> Result<(), String>,
    mut feld_argument: Option<&mut FeldArgument>,
) -> Result<(), String> {
    let crate_name = base_name();
    let mut lang_namen = None;
    let mut kurz_namen = KurzNamen::Keiner;
    macro_rules! setze_feld_argument {
        ($wert: expr, $sub_arg: expr) => {
            if let Some(var) = feld_argument.as_mut() {
                **var = $wert
            } else {
                return Err(format!("Argument nicht unterstützt: {}", $sub_arg));
            }
        };
    }
    for sub_arg in sub_args {
        let sub_arg_str = sub_arg.to_string();
        let Argument { name, wert } = sub_arg;
        let wert_str = wert.to_string();
        match wert {
            ArgumentWert::KeinWert => match name.as_str() {
                "kurz" | "short" => kurz_namen = KurzNamen::Auto,
                "glätten" | "flatten" => setze_feld_argument!(FeldArgument::Parse, sub_arg_str),
                "FromStr" => setze_feld_argument!(FeldArgument::FromStr, sub_arg_str),
                "benötigt" | "required" => setze_standard(None, &name, &name)?,
                _ => todo!(),
            },
            ArgumentWert::Liste(liste) => match name.as_str() {
                "lang" | "long" => {
                    let mut namen_iter = liste.into_iter().map(|ts| ts.to_string());
                    let (head, tail) = if let Some(head) = namen_iter.next() {
                        (head, namen_iter.collect())
                    } else {
                        return Err(format!("Kein LangName für {arg_name}!"));
                    };
                    lang_namen = Some((head, tail));
                },
                "kurz" | "short" => {
                    let namen_iter = liste.into_iter().map(|ts| ts.to_string());
                    kurz_namen = KurzNamen::Namen(namen_iter.collect());
                },
                _ => {
                    return Err(format!(
                        "Benanntes Argument {name} für {arg_name} nicht unterstützt: {wert_str}"
                    ))
                },
            },
            ArgumentWert::Stream(ts) => match name.as_str() {
                "sprache" | "language" => {
                    let sprache = parse_sprache(ts);
                    setze_sprache(sprache, &sub_arg_str)?
                },
                "standard" | "default" => setze_standard(Some(ts), &name, &wert_str)?,
                "meta_var" => setze_meta_var(&wert_str, &name, &wert_str)?,
                "invertiere_präfix" | "invert_prefix" => {
                    setze_invertiere_präfix(&wert_str, &name, &wert_str)?
                },
                "lang" | "long" => lang_namen = Some((wert_str, Vec::new())),
                "kurz" | "short" => kurz_namen = KurzNamen::Namen(vec![wert_str]),
                _ => {
                    return Err(format!(
                        "Benanntes Argument {name} für {arg_name} nicht unterstützt: {wert_str}"
                    ))
                },
            },
            _ => return Err(format!("Argument für {arg_name} nicht unterstützt: {sub_arg_str}")),
        }
    }
    let (head, tail) = lang_namen.unwrap_or((arg_name.to_owned(), Vec::new()));
    let lang_namen = quote!(
        #crate_name::NonEmpty {
            head: #head.to_owned(),
            tail: vec![#(#tail.to_owned()),*]
        }
    );
    setze_lang_namen(lang_namen);
    let kurz_namen_iter = kurz_namen.to_vec(&head).into_iter();
    let kurz_namen = quote!(vec![#(#kurz_namen_iter.to_owned()),*]);
    setze_kurz_namen(kurz_namen);
    Ok(())
}

fn wert_argument_error_message<'t>(
    arg_name: &'t str,
) -> impl 't + Fn(&str, &str, &str) -> Result<(), String> {
    move |_wert, ignored, string| {
        Err(format!("Benanntes Argument {ignored} für {arg_name} nicht unterstützt: {string}"))
    }
}

fn sprache_error_message<'t>() -> impl 't + Fn(Sprache, &str) -> Result<(), String> {
    move |_sprache, string| Err(format!("Argument nicht unterstützt: {string}"))
}

fn standard_error_message<'t>(
    arg_name: &'t str,
) -> impl 't + Fn(Option<TokenStream>, &str, &str) -> Result<(), String> {
    move |_wert, ignored, string| {
        Err(format!("Benanntes Argument {ignored} für {arg_name} nicht unterstützt: {string}"))
    }
}

pub(crate) fn derive_parse(item_struct: ItemStruct) -> TokenStream {
    let ItemStruct { ident, generics, attrs, fields, .. } = item_struct;
    if !generics.params.is_empty() {
        compile_error_return!("Nur Structs ohne Generics unterstützt.");
    }
    let mut args: Vec<Argument> = Vec::new();
    for attr in attrs {
        if attr.path.is_ident("kommandozeilen_argumente") {
            split_klammer_argumente!(args, attr.tokens);
        }
    }
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    // let version = env!("CARGO_PKG_VERSION");
    // CARGO_PKG_NAME — The name of your package.
    // CARGO_PKG_VERSION — The full version of your package.
    // CARGO_PKG_AUTHORS — Colon separated list of authors from the manifest of your package.
    // CARGO_PKG_DESCRIPTION — The description from the manifest of your package.
    // CARGO_BIN_NAME — The name of the binary that is currently being compiled (if it is a binary). This name does not include any file extension, such as .exe
    let mut erstelle_version: Option<Box<dyn FnOnce(TokenStream, Sprache) -> TokenStream>> = None;
    let mut erstelle_hilfe: Option<Box<dyn FnOnce(TokenStream) -> TokenStream>> = None;
    let mut sprache = English;
    let mut invertiere_präfix = None;
    let mut meta_var = None;
    let crate_name = base_name();
    for arg in args {
        let arg_str = arg.to_string();
        let Argument { name, wert } = arg;
        match wert {
            ArgumentWert::KeinWert => match name.as_str() {
                "version" => {
                    erstelle_version = Some(Box::new(erstelle_version_methode(None, None)))
                },
                "hilfe" => erstelle_hilfe = Some(Box::new(erstelle_hilfe_methode(Deutsch, None))),
                "help" => erstelle_hilfe = Some(Box::new(erstelle_hilfe_methode(English, None))),
                _ => compile_error_return!("Argument nicht unterstützt: {name}"),
            },
            ArgumentWert::Unterargument(sub_args) => {
                let mut sub_sprache = None;
                let mut lang_namen = quote!();
                let mut kurz_namen = quote!();
                unwrap_result_or_compile_error!(parse_wert_arg(
                    &name,
                    sub_args,
                    |sprache, _string| {
                        sub_sprache = Some(sprache);
                        Ok(())
                    },
                    |namen| { lang_namen = namen },
                    |namen| { kurz_namen = namen },
                    wert_argument_error_message(&name),
                    wert_argument_error_message(&name),
                    standard_error_message(&name),
                    None,
                ));
                match name.as_str() {
                    "hilfe" | "help" => {
                        let standard_sprache = if name == "hilfe" { Deutsch } else { English };
                        erstelle_hilfe = Some(Box::new(erstelle_hilfe_methode(
                            sub_sprache.unwrap_or(standard_sprache),
                            Some((lang_namen, kurz_namen)),
                        )))
                    },
                    "version" => {
                        erstelle_version = Some(Box::new(erstelle_version_methode(
                            sub_sprache,
                            Some((lang_namen, kurz_namen)),
                        )))
                    },
                    _ => {
                        compile_error_return!(
                            "Unter-Argument für {name} nicht unterstützt: {arg_str}"
                        )
                    },
                }
            },
            ArgumentWert::Liste(_liste) => {
                compile_error_return!("Listen-Argument für {name} nicht unterstützt: {arg_str:?}")
            },
            ArgumentWert::Stream(ts) => match name.as_str() {
                "sprache" | "language" => sprache = parse_sprache(ts),
                "invertiere_präfix" | "invert_prefix" => invertiere_präfix = Some(ts.to_string()),
                "meta_var" => meta_var = Some(ts.to_string()),
                _ => {
                    compile_error_return!(
                        "Benanntes Argument {name} nicht unterstützt: {arg_str:?}"
                    )
                },
            },
        }
    }
    let sprache_ts = sprache.clone().token_stream();
    let invertiere_präfix = if let Some(präfix) = invertiere_präfix {
        quote!(#präfix)
    } else {
        quote!(#sprache_ts.invertiere_präfix)
    };
    let meta_var = if let Some(meta_var) = meta_var {
        quote!(#meta_var)
    } else {
        quote!(#sprache_ts.meta_var)
    };
    let mut tuples = Vec::new();
    for field in fields {
        let Field { attrs, ident, .. } = field;
        let mut hilfe_lits = Vec::new();
        let ident = unwrap_option_or_compile_error!(ident, "Nur benannte Felder unterstützt.");
        let ident_str = ident.to_string();
        if ident_str.is_empty() {
            compile_error_return!("Benanntes Feld mit leerem Namen: {ident_str}")
        }
        let mut lang = quote!(#ident_str.to_owned());
        let mut kurz = quote!(None);
        let mut standard = quote!(#crate_name::parse::ParseArgument::standard());
        let mut feld_argument = FeldArgument::EnumArgument;
        let mut feld_invertiere_präfix = invertiere_präfix.clone();
        let mut feld_meta_var = meta_var.clone();
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
                let mut feld_args = Vec::new();
                split_klammer_argumente!(&mut feld_args, attr.tokens);
                unwrap_result_or_compile_error!(parse_wert_arg(
                    &ident_str,
                    feld_args,
                    sprache_error_message(),
                    |namen| lang = namen,
                    |namen| kurz = namen,
                    |wert_string, _, _| {
                        feld_meta_var = quote!(#wert_string);
                        Ok(())
                    },
                    |wert_string, _, _| {
                        feld_invertiere_präfix = quote!(#wert_string);
                        Ok(())
                    },
                    |opt_wert_ts, _sub_arg_name, _string| {
                        standard = match opt_wert_ts {
                            None => quote!(None),
                            Some(wert_ts) => quote!(Some(#wert_ts)),
                        };
                        Ok(())
                    },
                    Some(&mut feld_argument),
                ))
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
        let erstelle_beschreibung = quote!(
            let beschreibung = #crate_name::Beschreibung::neu(
                #lang,
                #kurz,
                #hilfe,
                #standard,
            );
        );
        let erstelle_args = match feld_argument {
            FeldArgument::EnumArgument => {
                quote!({
                    #erstelle_beschreibung
                    #crate_name::ParseArgument::argumente(
                        beschreibung,
                        #feld_invertiere_präfix,
                        #feld_meta_var
                    )
                })
            },
            FeldArgument::FromStr => {
                quote!({
                    #erstelle_beschreibung
                    #crate_name::Argumente::wert_from_str_display(
                        beschreibung,
                        #feld_meta_var.to_owned(),
                        None,
                    )
                })
            },
            FeldArgument::Parse => {
                quote!(#crate_name::Parse::kommandozeilen_argumente())
            },
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
        impl #crate_name::Parse for #ident {
            type Fehler = String;

            fn kommandozeilen_argumente() -> #crate_name::Argumente<Self, Self::Fehler> {
                #nach_hilfe
            }
        }
    }
}
