//! Implementierung für das derive-Macro des Parse-Traits.

use std::fmt::{self, Display, Formatter};

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{parse2, Data, DataStruct, DeriveInput, Field, Ident};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    base_name,
    split_argumente::{
        genau_eines, split_klammer_argumente, Argument, ArgumentWert, SplitArgumenteFehler,
    },
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

    fn to_vec_ts(self, lang_name: &str) -> TokenStream {
        let vec = self.to_vec(&lang_name);
        if vec.is_empty() {
            quote!(None::<&str>)
        } else {
            quote!(vec![#(#vec),*])
        }
    }
}

fn erstelle_version_methode(
    feste_sprache: Option<Sprache>,
    namen: Option<(Option<String>, TokenStream, Option<String>, TokenStream)>,
) -> impl FnOnce(TokenStream, Sprache) -> TokenStream {
    let crate_name = base_name();
    move |item, sprache| {
        let sprache_ts = feste_sprache.unwrap_or(sprache).token_stream();
        let lang_standard = quote!(#sprache_ts.version_lang);
        let kurz_standard = quote!(#sprache_ts.version_kurz);
        let (lang_präfix, lang_namen, kurz_präfix, kurz_namen) =
            namen.unwrap_or_else(|| (None, lang_standard, None, kurz_standard));
        let lang_präfix = if let Some(präfix_str) = lang_präfix {
            quote!(#präfix_str)
        } else {
            quote!(#sprache_ts.lang_präfix)
        };
        let kurz_präfix = if let Some(präfix_str) = kurz_präfix {
            quote!(#präfix_str)
        } else {
            quote!(#sprache_ts.kurz_präfix)
        };
        let beschreibung = quote!(
            #crate_name::Beschreibung::neu(
                #lang_präfix,
                #lang_namen,
                #kurz_präfix,
                #kurz_namen,
                Some(#sprache_ts.version_beschreibung),
                None,
            )
        );
        quote!(
            #item.zeige_version(
                #beschreibung,
                #crate_name::crate_name!(),
                #crate_name::crate_version!(),
            )
        )
    }
}

fn erstelle_hilfe_methode(
    sprache: Sprache,
    namen: Option<(Option<String>, TokenStream, Option<String>, TokenStream)>,
) -> impl Fn(TokenStream) -> TokenStream {
    let crate_name = base_name();
    let sprache_ts = sprache.token_stream();
    let lang_standard = quote!(#sprache_ts.hilfe_lang);
    let kurz_standard = quote!(#sprache_ts.hilfe_kurz);
    let (lang_präfix, lang_namen, kurz_präfix, kurz_namen) =
        namen.unwrap_or_else(|| (None, lang_standard, None, kurz_standard));
    let lang_präfix = if let Some(präfix_str) = lang_präfix {
        quote!(#präfix_str)
    } else {
        quote!(#sprache_ts.lang_präfix)
    };
    let kurz_präfix = if let Some(präfix_str) = kurz_präfix {
        quote!(#präfix_str)
    } else {
        quote!(#sprache_ts.kurz_präfix)
    };
    let beschreibung = quote!(
        #crate_name::Beschreibung::neu(
            #lang_präfix,
            #lang_namen,
            #kurz_präfix,
            #kurz_namen,
            Some(#sprache_ts.hilfe_beschreibung),
            None,
        )
    );
    move |item| {
        quote!(
            #item.erstelle_hilfe_mit_sprache(
                #beschreibung,
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
pub(crate) enum ParseWertFehler {
    NichtUnterstützt { arg_name: String, argument: Argument },
    KeinLangName { arg_name: String, name: String },
}

impl Display for ParseWertFehler {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentWert::*;
        use ParseWertFehler::*;
        match self {
            NichtUnterstützt { arg_name, argument: Argument { name, wert: KeinWert } } => {
                write!(f, "Argument für {arg_name} nicht unterstützt: {name}")
            },
            NichtUnterstützt {
                arg_name,
                argument: Argument { name, wert: wert @ Unterargument(_) },
            } => {
                write!(f, "Unterargument von {name} für {arg_name} nicht unterstützt: {wert}")
            },
            NichtUnterstützt { arg_name, argument: Argument { name, wert: wert @ Liste(_) } } => {
                write!(f, "Listen-Argument {name} für {arg_name} nicht unterstützt: {wert}")
            },
            NichtUnterstützt { arg_name, argument: Argument { name, wert: wert @ Stream(_) } } => {
                write!(f, "Benanntes Argument {name} für {arg_name} nicht unterstützt: {wert}")
            },
            KeinLangName { arg_name, name } => {
                write!(f, "Kein Langname für {arg_name} in expliziter Liste mit {name} angegeben!")
            },
        }
    }
}

type LangNamen = (String, Vec<String>);
type ErstelleFehler<'t> = Box<dyn 't + FnOnce(String) -> ParseWertFehler>;

fn parse_wert_arg<'t, ES, EL, EK, EW, EM, EP, EI, ED>(
    sub_args: Vec<Argument>,
    mut setze_sprache: impl FnMut(String, TokenStream) -> Result<(), ES>,
    mut setze_lang_präfix: impl FnMut(String, TokenStream) -> Result<(), EL>,
    mut setze_kurz_präfix: impl FnMut(String, TokenStream) -> Result<(), EK>,
    mut setze_invertiere_präfix: impl FnMut(String, TokenStream) -> Result<(), EP>,
    mut setze_invertiere_infix: impl FnMut(String, TokenStream) -> Result<(), EI>,
    mut setze_wert_infix: impl FnMut(String, TokenStream) -> Result<(), EW>,
    mut setze_meta_var: impl FnMut(String, TokenStream) -> Result<(), EM>,
    mut setze_standard: impl FnMut(String, Option<TokenStream>) -> Result<(), ED>,
    mut feld_argument: Option<&mut FeldArgument>,
) -> Result<(Option<LangNamen>, KurzNamen), ErstelleFehler<'t>>
where
    ES: 't + FnOnce(String) -> ParseWertFehler,
    EL: 't + FnOnce(String) -> ParseWertFehler,
    EK: 't + FnOnce(String) -> ParseWertFehler,
    EW: 't + FnOnce(String) -> ParseWertFehler,
    EM: 't + FnOnce(String) -> ParseWertFehler,
    EP: 't + FnOnce(String) -> ParseWertFehler,
    EI: 't + FnOnce(String) -> ParseWertFehler,
    ED: 't + FnOnce(String) -> ParseWertFehler,
{
    let map_es = |f: ES| -> Box<dyn FnOnce(String) -> ParseWertFehler> { Box::new(f) };
    let map_el = |f: EL| -> Box<dyn FnOnce(String) -> ParseWertFehler> { Box::new(f) };
    let map_ek = |f: EK| -> Box<dyn FnOnce(String) -> ParseWertFehler> { Box::new(f) };
    let map_ew = |f: EW| -> Box<dyn FnOnce(String) -> ParseWertFehler> { Box::new(f) };
    let map_em = |f: EM| -> Box<dyn FnOnce(String) -> ParseWertFehler> { Box::new(f) };
    let map_ep = |f: EP| -> Box<dyn FnOnce(String) -> ParseWertFehler> { Box::new(f) };
    let map_ei = |f: EI| -> Box<dyn FnOnce(String) -> ParseWertFehler> { Box::new(f) };
    let map_ed = |f: ED| -> Box<dyn FnOnce(String) -> ParseWertFehler> { Box::new(f) };
    use ParseWertFehler::*;
    let mut lang_namen = None;
    let mut kurz_namen = KurzNamen::Keiner;
    macro_rules! setze_feld_argument {
        ($wert: expr, $sub_arg: expr) => {
            if let Some(var) = feld_argument.as_mut() {
                **var = $wert
            } else {
                return Err(Box::new(|arg_name| NichtUnterstützt {
                    arg_name,
                    argument: $sub_arg,
                }));
            }
        };
    }
    for Argument { name, wert } in sub_args {
        match wert {
            ArgumentWert::KeinWert => match name.as_str() {
                "kurz" | "short" => kurz_namen = KurzNamen::Auto,
                "glätten" | "flatten" => {
                    setze_feld_argument!(FeldArgument::Parse, Argument { name, wert })
                },
                "FromStr" => setze_feld_argument!(FeldArgument::FromStr, Argument { name, wert }),
                "benötigt" | "required" => setze_standard(name, None).map_err(map_ed)?,
                _ => {
                    return Err(Box::new(|arg_name| NichtUnterstützt {
                        arg_name,
                        argument: Argument { name, wert },
                    }))
                },
            },
            ArgumentWert::Liste(liste) => match name.as_str() {
                "lang" | "long" => {
                    let mut namen_iter = liste.into_iter().map(|ts| ts.to_string());
                    let (head, tail) = if let Some(head) = namen_iter.next() {
                        (head, namen_iter.collect())
                    } else {
                        return Err(Box::new(|arg_name| KeinLangName { arg_name, name }));
                    };
                    lang_namen = Some((head, tail));
                },
                "kurz" | "short" => {
                    let namen_iter = liste.into_iter().map(|ts| ts.to_string());
                    kurz_namen = KurzNamen::Namen(namen_iter.collect());
                },
                _ => {
                    return Err(Box::new(|arg_name| NichtUnterstützt {
                        arg_name,
                        argument: Argument { name, wert: ArgumentWert::Liste(liste) },
                    }))
                },
            },
            ArgumentWert::Stream(ts) => match name.as_str() {
                "sprache" | "language" => setze_sprache(name, ts).map_err(map_es)?,
                "lang_präfix" | "long_prefix" => setze_lang_präfix(name, ts).map_err(map_el)?,
                "kurz_präfix" | "short_prefix" => setze_kurz_präfix(name, ts).map_err(map_ek)?,
                "invertiere_präfix" | "invert_prefix" => {
                    setze_invertiere_präfix(name, ts).map_err(map_ep)?
                },
                "invertiere_infix" | "invert_infix" => {
                    setze_invertiere_infix(name, ts).map_err(map_ei)?
                },
                "wert_infix" | "value_infix" => setze_wert_infix(name, ts).map_err(map_ew)?,
                "meta_var" => setze_meta_var(name, ts).map_err(map_em)?,
                "standard" | "default" => setze_standard(name, Some(ts)).map_err(map_ed)?,
                "lang" | "long" => lang_namen = Some((ts.to_string(), Vec::new())),
                "kurz" | "short" => kurz_namen = KurzNamen::Namen(vec![ts.to_string()]),
                _ => {
                    return Err(Box::new(|arg_name| NichtUnterstützt {
                        arg_name,
                        argument: Argument { name, wert: ArgumentWert::Stream(ts) },
                    }))
                },
            },
            wert @ ArgumentWert::Unterargument(_) => {
                return Err(Box::new(|arg_name| NichtUnterstützt {
                    arg_name,
                    argument: Argument { name, wert },
                }))
            },
        }
    }
    Ok((lang_namen, kurz_namen))
}

fn wert_argument_error_message(
    name: String,
    wert_ts: TokenStream,
) -> Result<(), impl FnOnce(String) -> ParseWertFehler> {
    Err(move |arg_name: String| {
        let argument = Argument { name, wert: ArgumentWert::Stream(wert_ts) };
        ParseWertFehler::NichtUnterstützt { arg_name, argument }
    })
}

fn standard_error_message(
    name: String,
    standard: Option<TokenStream>,
) -> Result<(), impl FnOnce(String) -> ParseWertFehler> {
    Err(move |arg_name: String| {
        let wert = match standard {
            Some(ts) => ArgumentWert::Stream(ts),
            None => ArgumentWert::KeinWert,
        };
        ParseWertFehler::NichtUnterstützt { arg_name, argument: Argument { name, wert } }
    })
}

#[derive(Debug)]
pub(crate) enum TypNichtUnterstützt {
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
pub(crate) enum Fehler {
    Syn(syn::Error),
    SplitArgumente(SplitArgumenteFehler),
    ParseWert(ParseWertFehler),
    KeinStruct { typ: TypNichtUnterstützt, input: TokenStream },
    Generics { anzahl: usize, where_clause: bool },
    NichtUnterstützt(Argument),
    FeldOhneName,
    LeererFeldName(Ident),
}

impl Display for Fehler {
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

impl From<syn::Error> for Fehler {
    fn from(input: syn::Error) -> Fehler {
        Fehler::Syn(input)
    }
}

impl From<SplitArgumenteFehler> for Fehler {
    fn from(input: SplitArgumenteFehler) -> Fehler {
        Fehler::SplitArgumente(input)
    }
}

impl From<ParseWertFehler> for Fehler {
    fn from(input: ParseWertFehler) -> Fehler {
        Fehler::ParseWert(input)
    }
}

macro_rules! unwrap_or_call_return {
    ($result:expr $(, $($arg:expr)+)? $(,)?) => {
        match $result {
            Ok(wert) => wert,
            Err(f) => return Err(f($($($arg)+)?).into()),
        }
    };
}

pub(crate) fn derive_parse(input: TokenStream) -> Result<TokenStream, Fehler> {
    use Fehler::*;
    use TypNichtUnterstützt::*;
    let derive_input: DeriveInput = parse2(input.clone())?;
    let DataStruct { fields, .. } = match derive_input.data {
        Data::Struct(data_struct) => data_struct,
        Data::Enum(_) => return Err(KeinStruct { typ: Enum, input }),
        Data::Union(_) => return Err(KeinStruct { typ: Union, input }),
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
    let mut lang_präfix = None;
    let mut kurz_präfix = None;
    let mut invertiere_präfix = None;
    let mut invertiere_infix = None;
    let mut wert_infix = None;
    let mut meta_var = None;
    let crate_name = base_name();
    for Argument { name, wert } in args {
        match wert {
            ArgumentWert::KeinWert => match name.as_str() {
                "version" => {
                    erstelle_version = Some(Box::new(erstelle_version_methode(None, None)))
                },
                "hilfe" => erstelle_hilfe = Some(Box::new(erstelle_hilfe_methode(Deutsch, None))),
                "help" => erstelle_hilfe = Some(Box::new(erstelle_hilfe_methode(English, None))),
                _ => return Err(NichtUnterstützt(Argument { name, wert })),
            },
            ArgumentWert::Unterargument(sub_args) => {
                let verarbeiten: Box<
                    dyn FnOnce(
                        Option<Sprache>,
                        Option<String>,
                        TokenStream,
                        Option<String>,
                        TokenStream,
                    ),
                > = match name.as_str() {
                    "hilfe" | "help" => {
                        let standard_sprache = if name == "hilfe" { Deutsch } else { English };
                        Box::new(
                            |sub_sprache,
                             sub_lang_präfix,
                             lang_namen,
                             sub_kurz_präfix,
                             kurz_namen| {
                                erstelle_hilfe = Some(Box::new(erstelle_hilfe_methode(
                                    sub_sprache.unwrap_or(standard_sprache),
                                    Some((
                                        sub_lang_präfix,
                                        lang_namen,
                                        sub_kurz_präfix,
                                        kurz_namen,
                                    )),
                                )))
                            },
                        )
                    },
                    "version" => Box::new(
                        |sub_sprache, sub_lang_präfix, lang_namen, sub_kurz_präfix, kurz_namen| {
                            erstelle_version = Some(Box::new(erstelle_version_methode(
                                sub_sprache,
                                Some((sub_lang_präfix, lang_namen, sub_kurz_präfix, kurz_namen)),
                            )))
                        },
                    ),
                    _ => {
                        return Err(NichtUnterstützt(Argument {
                            name,
                            wert: ArgumentWert::Unterargument(sub_args),
                        }))
                    },
                };
                let mut sub_sprache = None;
                let mut sub_lang_präfix = None;
                let mut sub_kurz_präfix = None;
                let (lang, kurz) = unwrap_or_call_return!(
                    parse_wert_arg(
                        sub_args,
                        |_sub_arg_name, ts| -> Result<(), fn(String) -> ParseWertFehler> {
                            sub_sprache = Some(parse_sprache(ts));
                            Ok(())
                        },
                        |_sub_arg_name, ts| -> Result<(), fn(String) -> ParseWertFehler> {
                            sub_lang_präfix = Some(ts.to_string());
                            Ok(())
                        },
                        |_sub_arg_name, ts| -> Result<(), fn(String) -> ParseWertFehler> {
                            sub_kurz_präfix = Some(ts.to_string());
                            Ok(())
                        },
                        wert_argument_error_message,
                        wert_argument_error_message,
                        wert_argument_error_message,
                        wert_argument_error_message,
                        standard_error_message,
                        None,
                    ),
                    name
                );
                let (lang_namen, erster) = match lang.as_ref() {
                    Some((head, tail)) => (
                        quote!(
                            #crate_name::NonEmpty {
                                head: #head,
                                tail: vec![#(#tail),*]
                            }
                        ),
                        head,
                    ),
                    None => (quote!(#name), &name),
                };
                let kurz_namen = kurz.to_vec_ts(erster);
                verarbeiten(sub_sprache, sub_lang_präfix, lang_namen, sub_kurz_präfix, kurz_namen)
            },
            wert @ ArgumentWert::Liste(_) => {
                return Err(NichtUnterstützt(Argument { name, wert }))
            },
            ArgumentWert::Stream(ts) => match name.as_str() {
                "sprache" | "language" => sprache = parse_sprache(ts),
                "lang_präfix" | "long_prefix" => lang_präfix = Some(ts.to_string()),
                "kurz_präfix" | "short_prefix" => kurz_präfix = Some(ts.to_string()),
                "invertiere_präfix" | "invert_prefix" => invertiere_präfix = Some(ts.to_string()),
                "invertiere_infix" | "invert_infix" => invertiere_infix = Some(ts.to_string()),
                "wert_infix" | "value_infix" => wert_infix = Some(ts.to_string()),
                "meta_var" => meta_var = Some(ts.to_string()),
                _ => {
                    return Err(NichtUnterstützt(
                        Argument { name, wert: ArgumentWert::Stream(ts) },
                    ))
                },
            },
        }
    }
    let sprache_ts = sprache.clone().token_stream();
    let lang_präfix = if let Some(präfix) = lang_präfix {
        quote!(#präfix)
    } else {
        quote!(#sprache_ts.lang_präfix)
    };
    let kurz_präfix = if let Some(präfix) = kurz_präfix {
        quote!(#präfix)
    } else {
        quote!(#sprache_ts.kurz_präfix)
    };
    let invertiere_präfix = if let Some(präfix) = invertiere_präfix {
        quote!(#präfix)
    } else {
        quote!(#sprache_ts.invertiere_präfix)
    };
    let invertiere_infix = if let Some(infix) = invertiere_infix {
        quote!(#infix)
    } else {
        quote!(#sprache_ts.invertiere_infix)
    };
    let wert_infix =
        if let Some(infix) = wert_infix { quote!(#infix) } else { quote!(#sprache_ts.wert_infix) };
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
        if ident_str.is_empty() {
            return Err(LeererFeldName(ident));
        }
        let mut lang = quote!(#ident_str);
        let mut kurz = quote!(None::<&str>);
        let mut standard = quote!(#crate_name::parse::ParseArgument::standard());
        let mut feld_argument = FeldArgument::EnumArgument;
        let mut feld_lang_präfix = lang_präfix.clone();
        let mut feld_kurz_präfix = kurz_präfix.clone();
        let mut feld_invertiere_präfix = invertiere_präfix.clone();
        let mut feld_invertiere_infix = invertiere_infix.clone();
        let mut feld_wert_infix = wert_infix.clone();
        let mut feld_meta_var = meta_var.clone();
        for attr in attrs {
            if attr.path.is_ident("doc") {
                let args_str = attr.tokens.to_string();
                if let Some(stripped) =
                    args_str.strip_prefix("= \"").and_then(|s| s.strip_suffix('"'))
                {
                    let trimmed = stripped.trim();
                    if !trimmed.is_empty() {
                        hilfe_lits.push(trimmed.to_owned());
                    }
                }
            } else if attr.path.is_ident("kommandozeilen_argumente") {
                let mut feld_args = Vec::new();
                split_klammer_argumente(Vec::new(), &mut feld_args, attr.tokens)?;
                let (lang_namen, kurz_namen) = unwrap_or_call_return!(
                    parse_wert_arg(
                        feld_args,
                        wert_argument_error_message,
                        |_name, wert_ts| -> Result<(), fn(String) -> ParseWertFehler> {
                            let wert_string = wert_ts.to_string();
                            feld_lang_präfix = quote!(#wert_string);
                            Ok(())
                        },
                        |_name, wert_ts| -> Result<(), fn(String) -> ParseWertFehler> {
                            let wert_string = wert_ts.to_string();
                            feld_kurz_präfix = quote!(#wert_string);
                            Ok(())
                        },
                        |_name, wert_ts| -> Result<(), fn(String) -> ParseWertFehler> {
                            let wert_string = wert_ts.to_string();
                            feld_invertiere_präfix = quote!(#wert_string);
                            Ok(())
                        },
                        |_name, wert_ts| -> Result<(), fn(String) -> ParseWertFehler> {
                            let wert_string = wert_ts.to_string();
                            feld_invertiere_infix = quote!(#wert_string);
                            Ok(())
                        },
                        |_name, wert_ts| -> Result<(), fn(String) -> ParseWertFehler> {
                            let wert_string = wert_ts.to_string();
                            feld_wert_infix = quote!(#wert_string);
                            Ok(())
                        },
                        |_name, wert_ts| -> Result<(), fn(String) -> ParseWertFehler> {
                            let wert_string = wert_ts.to_string();
                            feld_meta_var = quote!(#wert_string);
                            Ok(())
                        },
                        |_name, opt_wert_ts| -> Result<(), fn(String) -> ParseWertFehler> {
                            standard = match opt_wert_ts {
                                None => quote!(None),
                                Some(wert_ts) => quote!(Some(#wert_ts)),
                            };
                            Ok(())
                        },
                        Some(&mut feld_argument),
                    ),
                    ident_str
                );
                let erster = match lang_namen.as_ref() {
                    Some((head, tail)) => {
                        lang = quote!(
                            #crate_name::NonEmpty {
                                head: #head,
                                tail: vec![#(#tail),*]
                            }
                        );
                        head
                    },
                    None => {
                        lang = quote!(#ident_str);
                        &ident_str
                    },
                };
                kurz = kurz_namen.to_vec_ts(erster);
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
            quote!(None::<&str>)
        } else {
            quote!(Some(#hilfe_string))
        };
        // TODO case sensitive für namen, präfix, infix
        let erstelle_beschreibung = quote!(
            let beschreibung = #crate_name::Beschreibung::neu(
                #feld_lang_präfix,
                #lang,
                #feld_kurz_präfix,
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
                        #feld_invertiere_infix,
                        #feld_wert_infix,
                        #feld_meta_var
                    )
                })
            },
            FeldArgument::FromStr => {
                quote!({
                    #erstelle_beschreibung
                    #crate_name::Argumente::wert_from_str_display(
                        beschreibung,
                        #feld_wert_infix,
                        #feld_meta_var,
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
        #crate_name::kombiniere!(|#(#idents),*| Self {#(#idents),*}, #(#idents),*)
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

            fn kommandozeilen_argumente<'t>() -> #crate_name::Argumente<'t, Self, Self::Fehler> {
                #nach_hilfe
            }
        }
    };
    Ok(ts)
}
