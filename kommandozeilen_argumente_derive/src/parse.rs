//! Implementierung für das derive-Macro des Parse-Traits.

use std::{
    fmt::{self, Display, Formatter},
    iter,
};

use proc_macro2::{Delimiter, Punct, Spacing, TokenStream, TokenTree};
use quote::quote;
use syn::{parse2, Data, DataStruct, DeriveInput, Field, Ident};
use unicode_segmentation::UnicodeSegmentation;

use crate::base_name;

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

fn write_argument_wert(f: &mut Formatter<'_>, colon: bool, wert: &ArgumentWert) -> fmt::Result {
    use ArgumentWert::*;
    match wert {
        KeinWert => Ok(()),
        Unterargument(args) => write_liste(f, if colon { ": (" } else { "(" }, args, ")"),
        Liste(tts) => write_liste(f, if colon { ": [" } else { "[" }, tts.clone(), "]"),
        Stream(ts) => write!(f, "{ts}"),
    }
}

impl Display for ArgumentWert {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write_argument_wert(f, false, self)
    }
}

#[derive(Debug)]
struct Argument {
    name: String,
    wert: ArgumentWert,
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Argument { name, wert } = self;
        f.write_str(name)?;
        write_argument_wert(f, true, wert)
    }
}

#[derive(Debug)]
enum SplitArgumenteFehler {
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

impl Display for SplitArgumenteFehler {
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
        use SplitArgumenteFehler::*;
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
) -> Result<(), SplitArgumenteFehler> {
    use SplitArgumenteFehler::*;
    let mut iter = args_ts.into_iter().peekable();
    let iter_mut_ref = iter.by_ref();
    while iter_mut_ref.peek().is_some() {
        let mut arg_iter = iter_mut_ref.take_while(tt_is_not_comma).peekable();
        // Name extrahieren
        let name = match arg_iter.next() {
            Some(TokenTree::Ident(ident)) => ident.to_string(),
            Some(tt) => return Err(InvaliderArgumentName { parent, tt }),
            None => return Err(LeeresArgument { parent, ts: iter_mut_ref.collect() }),
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
                        if !current.is_empty() {
                            acc.push(current)
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
                return Err(InvaliderArgumentWert {
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
) -> Result<(), SplitArgumenteFehler> {
    use SplitArgumenteFehler::*;
    let group = match genau_eines(args_ts.into_iter()) {
        Ok(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => group,
        Ok(tt) => return Err(NichtInKlammer { parent, ts: tt.into() }),
        Err(fehler) => return Err(NichtInKlammer { parent, ts: fehler.collect() }),
    };
    split_argumente(parent, args, group.stream())
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

#[derive(Debug)]
enum ParseWertFehler<'t> {
    NichtUnterstützt { arg_name: &'t str, argument: Argument },
    KeinLangName { arg_name: &'t str, name: String },
}

impl Display for ParseWertFehler<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentWert::*;
        use ParseWertFehler::*;
        match self {
            NichtUnterstützt { arg_name, argument: Argument { name, wert: KeinWert } } => {
                write!(f, "Argument für {arg_name} nicht unterstützt: {name}")
            },
            NichtUnterstützt {
                arg_name,
                argument: Argument { name, wert: wert @ Unterargument(sub_args) },
            } => {
                write!(f, "Unterargument von {name} für {arg_name} nicht unterstützt: {wert}")
            },
            NichtUnterstützt { arg_name, argument: Argument { name, wert } } => {
                write!(f, "Benanntes Argument {name} für {arg_name} nicht unterstützt: {wert}")
            },
            KeinLangName { arg_name, name } => {
                write!(f, "Kein Langname für {arg_name} in expliziter Liste mit {name} angegeben!")
            },
        }
    }
}

fn parse_wert_arg<'t>(
    arg_name: &'t str,
    sub_args: Vec<Argument>,
    mut setze_sprache: impl FnMut(&str, String, Sprache, TokenStream) -> Result<(), ParseWertFehler<'_>>,
    setze_lang_namen: impl FnOnce(TokenStream),
    setze_kurz_namen: impl FnOnce(TokenStream),
    mut setze_meta_var: impl FnMut(&str, String, TokenStream) -> Result<(), ParseWertFehler<'_>>,
    mut setze_invertiere_präfix: impl FnMut(
        &str,
        String,
        TokenStream,
    ) -> Result<(), ParseWertFehler<'_>>,
    mut setze_standard: impl FnMut(&str, String, Option<TokenStream>) -> Result<(), ParseWertFehler<'_>>,
    mut feld_argument: Option<&mut FeldArgument>,
) -> Result<(), ParseWertFehler<'t>> {
    use ParseWertFehler::*;
    let crate_name = base_name();
    let mut lang_namen = None;
    let mut kurz_namen = KurzNamen::Keiner;
    macro_rules! setze_feld_argument {
        ($wert: expr, $sub_arg: expr) => {
            if let Some(var) = feld_argument.as_mut() {
                **var = $wert
            } else {
                return Err(NichtUnterstützt { arg_name, argument: $sub_arg });
            }
        };
    }
    for sub_arg in sub_args {
        let Argument { name, wert } = sub_arg;
        match wert {
            ArgumentWert::KeinWert => match name.as_str() {
                "kurz" | "short" => kurz_namen = KurzNamen::Auto,
                "glätten" | "flatten" => setze_feld_argument!(FeldArgument::Parse, sub_arg),
                "FromStr" => setze_feld_argument!(FeldArgument::FromStr, sub_arg),
                "benötigt" | "required" => setze_standard(arg_name, name, None)?,
                _ => return Err(NichtUnterstützt { arg_name, argument: sub_arg }),
            },
            ArgumentWert::Liste(liste) => match name.as_str() {
                "lang" | "long" => {
                    let mut namen_iter = liste.into_iter().map(|ts| ts.to_string());
                    let (head, tail) = if let Some(head) = namen_iter.next() {
                        (head, namen_iter.collect())
                    } else {
                        return Err(KeinLangName { arg_name, name });
                    };
                    lang_namen = Some((head, tail));
                },
                "kurz" | "short" => {
                    let namen_iter = liste.into_iter().map(|ts| ts.to_string());
                    kurz_namen = KurzNamen::Namen(namen_iter.collect());
                },
                _ => return Err(NichtUnterstützt { arg_name, argument: sub_arg }),
            },
            ArgumentWert::Stream(ts) => match name.as_str() {
                "sprache" | "language" => {
                    let sprache = parse_sprache(ts);
                    setze_sprache(arg_name, name, sprache, ts)?
                },
                "standard" | "default" => setze_standard(arg_name, name, Some(ts))?,
                "meta_var" => setze_meta_var(arg_name, name, ts)?,
                "invertiere_präfix" | "invert_prefix" => {
                    setze_invertiere_präfix(arg_name, name, ts)?
                },
                "lang" | "long" => lang_namen = Some((ts.to_string(), Vec::new())),
                "kurz" | "short" => kurz_namen = KurzNamen::Namen(vec![ts.to_string()]),
                _ => return Err(NichtUnterstützt { arg_name, argument: sub_arg }),
            },
            ArgumentWert::Unterargument(_) => {
                return Err(NichtUnterstützt { arg_name, argument: sub_arg })
            },
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

fn wert_argument_error_message(
    arg_name: &str,
    name: String,
    wert: TokenStream,
) -> Result<(), ParseWertFehler<'_>> {
    Err(ParseWertFehler::NichtUnterstützt {
        arg_name,
        argument: Argument { name, wert: ArgumentWert::Stream(wert) },
    })
}

fn sprache_error_message(
    arg_name: &str,
    name: String,
    _sprache: Sprache,
    input: TokenStream,
) -> Result<(), ParseWertFehler<'_>> {
    Err(ParseWertFehler::NichtUnterstützt {
        arg_name,
        argument: Argument { name, wert: ArgumentWert::Stream(input) },
    })
}

fn standard_error_message(
    arg_name: &str,
    ignored_name: String,
    standard: Option<TokenStream>,
) -> Result<(), ParseWertFehler<'_>> {
    let wert = match standard {
        Some(ts) => ArgumentWert::Stream(ts),
        None => ArgumentWert::KeinWert,
    };
    Err(ParseWertFehler::NichtUnterstützt {
        arg_name,
        argument: Argument { name: ignored_name, wert },
    })
}

#[derive(Debug)]
enum TypNichtUnterstützt {
    Enum,
    Union,
}

impl Display for TypNichtUnterstützt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use TypNichtUnterstützt::*;
        f.write_str(match self {
            Enum => "enum",
            Union => "union",
        })
    }
}

#[derive(Debug)]
enum Fehler<'t> {
    Syn(syn::Error),
    SplitArgumente(SplitArgumenteFehler),
    ParseWert(ParseWertFehler<'t>),
    KeinStruct { typ: TypNichtUnterstützt, input: TokenStream },
    Generics { anzahl: usize, where_clause: bool },
    NichtUnterstützt(Argument),
    FeldOhneName,
    LeererFeldName(Ident),
}

impl Display for Fehler<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentWert::*;
        use Fehler::*;
        match self {
            Syn(error) => write!(f, "{error}"),
            SplitArgumente(fehler) => write!(f, "{fehler}"),
            ParseWert(fehler) => write!(f, "{fehler}"),
            KeinStruct { typ, input } => {
                write!(f, "Nur structs unterstützt, aber {typ} bekommen: {input}")
            },
            Generics { anzahl, where_clause } => {
                write!(f, "Nur Structs ohne Generics unterstützt, aber {anzahl} Parameter ")?;
                if *where_clause {
                    write!(f, "und eine where-Klausel ")?;
                }
                write!(f, "bekommen.")
            },
            NichtUnterstützt(Argument { name, wert: KeinWert }) => {
                write!(f, "Argument nicht unterstützt: {name}")
            },
            NichtUnterstützt(Argument { name, wert: wert @ Unterargument(_sub_args) }) => {
                write!(f, "Unterargument von {name} nicht unterstützt: {wert}")
            },
            NichtUnterstützt(Argument { name, wert: wert @ Liste(_liste) }) => {
                write!(f, "Listen-Argument {name} nicht unterstützt: {wert}")
            },
            NichtUnterstützt(Argument { name, wert: wert @ Stream(_ts) }) => {
                write!(f, "Benanntes Argument {name} nicht unterstützt: {wert}")
            },
            FeldOhneName => f.write_str("Nur benannte Felder unterstützt."),
            LeererFeldName(ident) => write!(f, "Benanntes Feld mit leerem Namen: {ident}"),
        }
    }
}

impl<'t> From<syn::Error> for Fehler<'t> {
    fn from(input: syn::Error) -> Fehler<'t> {
        Fehler::Syn(input)
    }
}

impl<'t> From<SplitArgumenteFehler> for Fehler<'t> {
    fn from(input: SplitArgumenteFehler) -> Fehler<'t> {
        Fehler::SplitArgumente(input)
    }
}

impl<'t> From<ParseWertFehler<'t>> for Fehler<'t> {
    fn from(input: ParseWertFehler<'t>) -> Fehler<'t> {
        Fehler::ParseWert(input)
    }
}

pub(crate) fn derive_parse<'t>(input: TokenStream) -> Result<TokenStream, Fehler<'t>> {
    use Fehler::*;
    let input_clone = input.clone();
    let derive_input: DeriveInput = parse2(input)?;
    let DataStruct { fields, .. } = match derive_input.data {
        Data::Struct(data_struct) => data_struct,
        Data::Enum(_) => {
            return Err(KeinStruct { typ: TypNichtUnterstützt::Enum, input: input_clone })
        },
        Data::Union(_) => {
            return Err(KeinStruct { typ: TypNichtUnterstützt::Enum, input: input_clone })
        },
    };
    let DeriveInput { ident, generics, attrs, .. } = derive_input;
    let has_where_clause = generics.where_clause.is_some();
    if !generics.params.is_empty() || has_where_clause {
        return Err(Generics { anzahl: generics.params.len(), where_clause: has_where_clause });
    }
    let mut args: Vec<Argument> = Vec::new();
    for attr in attrs {
        if attr.path.is_ident("kommandozeilen_argumente") {
            split_klammer_argumente(Vec::new(), &mut args, attr.tokens)?;
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
                _ => return Err(NichtUnterstützt(arg)),
            },
            ArgumentWert::Unterargument(sub_args) => {
                let mut sub_sprache = None;
                let mut lang_namen = quote!();
                let mut kurz_namen = quote!();
                parse_wert_arg(
                    &name,
                    sub_args,
                    |_arg_name, _sub_arg_name, sprache, _ts| {
                        sub_sprache = Some(sprache);
                        Ok(())
                    },
                    |namen| lang_namen = namen,
                    |namen| kurz_namen = namen,
                    wert_argument_error_message,
                    wert_argument_error_message,
                    standard_error_message,
                    None,
                )?;
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
                    _ => return Err(NichtUnterstützt(arg)),
                }
            },
            ArgumentWert::Liste(_liste) => return Err(NichtUnterstützt(arg)),
            ArgumentWert::Stream(ts) => match name.as_str() {
                "sprache" | "language" => sprache = parse_sprache(ts),
                "invertiere_präfix" | "invert_prefix" => invertiere_präfix = Some(ts.to_string()),
                "meta_var" => meta_var = Some(ts.to_string()),
                _ => return Err(NichtUnterstützt(arg)),
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
        let ident = ident.ok_or(FeldOhneName)?;
        let ident_str = ident.to_string();
        if ident_str.to_string().is_empty() {
            return Err(LeererFeldName(ident));
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
                split_klammer_argumente(Vec::new(), &mut feld_args, attr.tokens)?;
                parse_wert_arg(
                    &ident_str,
                    feld_args,
                    sprache_error_message,
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
                    |_arg_name, _sub_arg_name, opt_wert_ts| {
                        standard = match opt_wert_ts {
                            None => quote!(None),
                            Some(wert_ts) => quote!(Some(#wert_ts)),
                        };
                        Ok(())
                    },
                    Some(&mut feld_argument),
                )?;
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
    let ts = quote! {
        impl #crate_name::Parse for #ident {
            type Fehler = String;

            fn kommandozeilen_argumente() -> #crate_name::Argumente<Self, Self::Fehler> {
                #nach_hilfe
            }
        }
    };
    Ok(ts)
}
