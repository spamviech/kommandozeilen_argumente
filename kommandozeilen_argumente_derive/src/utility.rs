//! Datentypen und Funktionen um Komma-separierte Argumente aus einem [TokenStream] zu parsen,
//! sowie einige allgemeine Utility-Funktionen/Typen.

use std::{
    fmt::{self, Display, Formatter},
    iter,
};

use proc_macro2::{Delimiter, Ident, Punct, Spacing, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};

////////////////////////////////////////////////////////

pub(crate) fn base_name() -> Ident {
    format_ident!("{}", "kommandozeilen_argumente")
}

////////////////////////////////////////////////////////

pub(crate) enum GenauEinesFehler<T, I> {
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

pub(crate) fn genau_eines<T, I: Iterator<Item = T>>(
    mut iter: I,
) -> Result<T, GenauEinesFehler<T, I>> {
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

////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub(crate) enum Case {
    Sensitive,
    Insensitive,
}

impl Default for Case {
    fn default() -> Self {
        Case::Insensitive
    }
}

impl ToTokens for Case {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = base_name();
        let ts = match self {
            Case::Sensitive => quote!(#crate_name::unicode::Case::Sensitive),
            Case::Insensitive => quote!(#crate_name::unicode::Case::Insensitive),
        };
        tokens.extend(ts)
    }
}

impl Case {
    pub(crate) fn parse(ts: &TokenStream) -> Option<Case> {
        match ts.to_string().as_str() {
            "sensitive" => Some(Case::Sensitive),
            "insensitive" => Some(Case::Insensitive),
            _ => None,
        }
    }
}

////////////////////////////////////////////////////////

#[inline(always)]
fn punct_is_char(punct: &Punct, c: char) -> bool {
    punct.as_char() == c && punct.spacing() == Spacing::Alone
}

// TODO entferne Clone-derive
#[derive(Debug, Clone)]
pub(crate) enum ArgumentWert {
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

// TODO entferne Clone-derive
#[derive(Debug, Clone)]
pub(crate) struct Argument {
    pub(crate) name: String,
    pub(crate) wert: ArgumentWert,
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Argument { name, wert } = self;
        f.write_str(name)?;
        write_argument_wert(f, true, wert)
    }
}

#[derive(Debug)]
pub(crate) enum SplitArgumenteFehler {
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

pub(crate) fn split_klammer_argumente(
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
