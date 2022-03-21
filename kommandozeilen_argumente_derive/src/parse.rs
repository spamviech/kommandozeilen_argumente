//! Implementierung für das derive-Macro des Parse-Traits.

use std::fmt::{self, Display, Formatter};

use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{parse2, Data, DataStruct, DeriveInput, Field, Ident};
use unicode_segmentation::UnicodeSegmentation;

use crate::utility::{
    base_name, genau_eines, split_klammer_argumente, Argument, ArgumentWert, Case,
    SplitArgumenteFehler,
};

#[derive(Debug, Clone)]
enum Sprache {
    Deutsch,
    English,
    TokenStream(TokenStream),
}
use Sprache::{Deutsch, English};

impl Sprache {
    fn parse(ts: TokenStream) -> Sprache {
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

    fn token_stream(&self) -> TokenStream {
        use Sprache::*;
        let crate_name = base_name();
        match self {
            Deutsch => quote!(#crate_name::Sprache::DEUTSCH),
            English => quote!(#crate_name::Sprache::ENGLISH),
            TokenStream(ts) => ts.clone(),
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
    namen: Option<(LangPräfix, TokenStream, KurzPräfix, TokenStream)>,
) -> impl FnOnce(TokenStream, Sprache) -> TokenStream {
    let crate_name = base_name();
    move |item, standard_sprache| {
        let sprache = feste_sprache.unwrap_or(standard_sprache);
        let sprache_ts = sprache.token_stream();
        let lang_standard = quote!(#sprache_ts.version_lang);
        let kurz_standard = quote!(#sprache_ts.version_kurz);
        let (lang_präfix, lang_namen, kurz_präfix, kurz_namen) = namen.unwrap_or_else(|| {
            (LangPräfix::default(), lang_standard, KurzPräfix::default(), kurz_standard)
        });
        let lang_präfix = lang_präfix.token_stream(&sprache);
        let kurz_präfix = kurz_präfix.token_stream(&sprache);
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
    namen: Option<(LangPräfix, TokenStream, KurzPräfix, TokenStream)>,
) -> impl Fn(TokenStream) -> TokenStream {
    let crate_name = base_name();
    let sprache_ts = sprache.token_stream();
    let lang_standard = quote!(#sprache_ts.hilfe_lang);
    let kurz_standard = quote!(#sprache_ts.hilfe_kurz);
    let (lang_präfix, lang_namen, kurz_präfix, kurz_namen) = namen.unwrap_or_else(|| {
        (LangPräfix::default(), lang_standard, KurzPräfix::default(), kurz_standard)
    });
    let lang_präfix = lang_präfix.token_stream(&sprache);
    let kurz_präfix = kurz_präfix.token_stream(&sprache);
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
type ErstelleFehler = Box<dyn FnOnce(String) -> ParseWertFehler>;

macro_rules! newtype_ts {
    ($($name: ident),* $(,)?) => {
        $(
            #[derive(Debug, Clone)]
            struct $name(TokenStream);

            impl ToTokens for $name {
                fn to_tokens(&self, tokens: &mut TokenStream) {
                    self.0.to_tokens(tokens)
                }

                fn into_token_stream(self) -> TokenStream {
                    self.0
                }
            }
        )*
    };
}

newtype_ts! {
    MetaVar,
    Standard,
}

macro_rules! vergleich_typen {
    ($($name: ident ($sprache_ident: ident)),* $(,)?) => {
        $(
            #[derive(Debug, Clone)]
            struct $name { string: Option<String>, case: Option<Case> }

            impl Default for $name {
                fn default() -> Self {
                    $name { string: None, case: None }
                }
            }

            impl $name {
                fn token_stream(&self, sprache: &Sprache) -> TokenStream {
                    let crate_name = base_name();
                    let string = if let Some(string) = self.string {
                        quote!(#string)
                    } else {
                        let sprache_ts = sprache.token_stream();
                        quote!(#sprache_ts.$sprache_ident)
                    };
                    let case = self.case.unwrap_or_default();
                    quote!(#crate_name::unicode::Vergleich {
                        string: #string,
                        case: #case,
                    })
                }
            }
        )*
    };
}

vergleich_typen! {
    LangPräfix(lang_präfix),
    KurzPräfix(kurz_präfix),
    InvertierePräfix(invertiere_präfix),
    InvertiereInfix(invertiere_infix),
    WertInfix(wert_infix),
}

fn parse_wert_arg(
    sub_args: Vec<Argument>,
    mut sprache: Option<&mut Option<Sprache>>,
    mut lang_präfix: Option<&mut LangPräfix>,
    mut kurz_präfix: Option<&mut KurzPräfix>,
    mut invertiere_präfix: Option<&mut InvertierePräfix>,
    mut invertiere_infix: Option<&mut InvertiereInfix>,
    mut wert_infix: Option<&mut WertInfix>,
    mut meta_var: Option<&mut MetaVar>,
    mut standard: Option<&mut Standard>,
    mut feld_argument: Option<&mut FeldArgument>,
) -> Result<(Option<LangNamen>, KurzNamen), ErstelleFehler> {
    use ParseWertFehler::*;
    let mut lang_namen = None;
    let mut kurz_namen = KurzNamen::Keiner;
    macro_rules! setze_argument {
        ($mut_var: expr, $wert: expr, $sub_arg: expr) => {
            if let Some(var) = $mut_var.as_mut() {
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
        // TODO case sensitive für namen, präfix, infix
        // case-sensitive, case-insensitive alleine?
        // case: (in)sensitive
        // case(lang_präfix: (in)sensitive, lang: ...)
        match wert {
            ArgumentWert::KeinWert => match name.as_str() {
                "kurz" | "short" => kurz_namen = KurzNamen::Auto,
                "glätten" | "flatten" => {
                    setze_argument!(feld_argument, FeldArgument::Parse, Argument { name, wert })
                },
                "FromStr" => {
                    setze_argument!(feld_argument, FeldArgument::FromStr, Argument { name, wert })
                },
                "benötigt" | "required" => {
                    setze_argument!(standard, Standard(quote!(None)), Argument { name, wert })
                },
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
                "sprache" | "language" => setze_argument!(
                    sprache,
                    Some(Sprache::parse(ts)),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "lang_präfix" | "long_prefix" => setze_argument!(
                    lang_präfix,
                    LangPräfix::from(ts.to_string()),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "kurz_präfix" | "short_prefix" => setze_argument!(
                    kurz_präfix,
                    KurzPräfix::from(ts.to_string()),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "invertiere_präfix" | "invert_prefix" => setze_argument!(
                    invertiere_präfix,
                    InvertierePräfix::from(ts.to_string()),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "invertiere_infix" | "invert_infix" => setze_argument!(
                    invertiere_infix,
                    InvertiereInfix::from(ts.to_string()),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "wert_infix" | "value_infix" => setze_argument!(
                    wert_infix,
                    WertInfix::from(ts.to_string()),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "meta_var" => setze_argument!(
                    meta_var,
                    MetaVar(ts),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "standard" | "default" => setze_argument!(
                    standard,
                    Standard(quote!(Some(#ts))),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "lang" | "long" => lang_namen = Some((ts.to_string(), Vec::new())),
                "kurz" | "short" => kurz_namen = KurzNamen::Namen(vec![ts.to_string()]),
                "case" => todo!(),
                _ => {
                    return Err(Box::new(|arg_name| NichtUnterstützt {
                        arg_name,
                        argument: Argument { name, wert: ArgumentWert::Stream(ts) },
                    }))
                },
            },
            ArgumentWert::Unterargument(sub_arg) => match name.as_str() {
                "case" => todo!(),
                _ => {
                    return Err(Box::new(|arg_name| NichtUnterstützt {
                        arg_name,
                        argument: Argument { name, wert: ArgumentWert::Unterargument(sub_arg) },
                    }))
                },
            },
        }
    }
    Ok((lang_namen, kurz_namen))
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
    let mut args = Vec::new();
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
    let mut lang_präfix = LangPräfix::default();
    let mut kurz_präfix = KurzPräfix::default();
    let mut invertiere_präfix = InvertierePräfix::default();
    let mut invertiere_infix = InvertiereInfix::default();
    let mut wert_infix = WertInfix::default();
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
                    dyn FnOnce(Option<Sprache>, LangPräfix, TokenStream, KurzPräfix, TokenStream),
                > = match name.as_str() {
                    "hilfe" | "help" => {
                        let standard_sprache = if name == "hilfe" { Deutsch } else { English };
                        Box::new(
                            |s_sprache, s_lang_präfix, lang_namen, s_kurz_präfix, kurz_namen| {
                                erstelle_hilfe = Some(Box::new(erstelle_hilfe_methode(
                                    s_sprache.unwrap_or(standard_sprache),
                                    Some((s_lang_präfix, lang_namen, s_kurz_präfix, kurz_namen)),
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
                    "case" => todo!(),
                    _ => {
                        return Err(NichtUnterstützt(Argument {
                            name,
                            wert: ArgumentWert::Unterargument(sub_args),
                        }))
                    },
                };
                let mut sub_sprache = None;
                let mut sub_lang_präfix = LangPräfix::default();
                let mut sub_kurz_präfix = KurzPräfix::default();
                let (lang, kurz) = unwrap_or_call_return!(
                    parse_wert_arg(
                        sub_args,
                        Some(&mut sub_sprache),
                        Some(&mut sub_lang_präfix),
                        Some(&mut sub_kurz_präfix),
                        None,
                        None,
                        None,
                        None,
                        None,
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
                // TODO case sensitive für namen, präfix, infix
                // case-sensitive, case-insensitive alleine?
                // case: (in)sensitive
                // case(lang_präfix: (in)sensitive, lang: ...)
                "sprache" | "language" => sprache = Sprache::parse(ts),
                "lang_präfix" | "long_prefix" => lang_präfix = LangPräfix::from(ts.to_string()),
                "kurz_präfix" | "short_prefix" => kurz_präfix = KurzPräfix::from(ts.to_string()),
                "invertiere_präfix" | "invert_prefix" => {
                    invertiere_präfix = InvertierePräfix::from(ts.to_string())
                },
                "invertiere_infix" | "invert_infix" => {
                    invertiere_infix = InvertiereInfix::from(ts.to_string())
                },
                "wert_infix" | "value_infix" => wert_infix = WertInfix::from(ts.to_string()),
                "meta_var" => meta_var = Some(ts.to_string()),
                "case" => todo!(),
                _ => {
                    return Err(NichtUnterstützt(
                        Argument { name, wert: ArgumentWert::Stream(ts) },
                    ))
                },
            },
        }
    }
    let meta_var = if let Some(meta_var) = meta_var {
        quote!(#meta_var)
    } else {
        let sprache_ts = sprache.token_stream();
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
        let mut feld_lang_präfix = lang_präfix.clone();
        let mut feld_kurz_präfix = kurz_präfix.clone();
        let mut feld_invertiere_präfix = invertiere_präfix.clone();
        let mut feld_invertiere_infix = invertiere_infix.clone();
        let mut feld_wert_infix = wert_infix.clone();
        let mut feld_meta_var = MetaVar(meta_var.clone());
        let mut standard = Standard(quote!(#crate_name::parse::ParseArgument::standard()));
        let mut feld_argument = FeldArgument::EnumArgument;
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
                split_klammer_argumente(vec![ident.to_string()], &mut feld_args, attr.tokens)?;
                let (lang_namen, kurz_namen) = unwrap_or_call_return!(
                    parse_wert_arg(
                        feld_args,
                        None,
                        Some(&mut feld_lang_präfix),
                        Some(&mut feld_kurz_präfix),
                        Some(&mut feld_invertiere_präfix),
                        Some(&mut feld_invertiere_infix),
                        Some(&mut feld_wert_infix),
                        Some(&mut feld_meta_var),
                        Some(&mut standard),
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
        let feld_lang_präfix = feld_lang_präfix.token_stream(&sprache);
        let feld_kurz_präfix = feld_kurz_präfix.token_stream(&sprache);
        let feld_invertiere_präfix = feld_invertiere_präfix.token_stream(&sprache);
        let feld_invertiere_infix = feld_invertiere_infix.token_stream(&sprache);
        let feld_wert_infix = feld_wert_infix.token_stream(&sprache);
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
