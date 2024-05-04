//! Datentypen und Funktionen um Komma-separierte Argumente aus einem [`TokenStream`] zu parsen,
//! sowie einige allgemeine Utility-Funktionen/Typen.

use std::{
    fmt::{self, Display, Formatter},
    iter,
};

use proc_macro2::{Delimiter, Ident, Punct, Spacing, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};

/// Ident für den crate-Namen von `kommandozeilen_argumente`.
pub(crate) fn crate_name() -> Ident {
    format_ident!("{}", "kommandozeilen_argumente")
}

/// Es war nicht genau ein Element.
pub(crate) enum GenauEinesFehler<T, I> {
    /// Kein Element gegeben.
    Leer,
    /// Mehr als ein Element gegeben.
    MehrAlsEins {
        /// Das erste Element. Wird vom ersten [`Iterator::next`]-Aufruf auf [`None`] gesetzt.
        erstes: Option<T>,
        /// Das zweite Element. Wird vom zweiten [`Iterator::next`]-Aufruf auf [`None`] gesetzt.
        zweites: Option<T>,
        /// Alle weiteren Elemente.
        rest: I,
    },
}

impl<T, I: Iterator<Item = T>> Iterator for GenauEinesFehler<T, I> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        use GenauEinesFehler::{Leer, MehrAlsEins};
        match self {
            Leer => None,
            MehrAlsEins { erstes, zweites, rest } => {
                erstes.take().or_else(|| zweites.take()).or_else(|| rest.next())
            },
        }
    }
}

/// Erwarte einen [`Iterator`] mit genau einem Element.
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

/// Wird das Argument unter Berücksichtigung von Groß-/Kleinschreibung geparst?
#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum Case {
    /// Berücksichtige Groß-/Kleinschreibung.
    Sensitive,
    /// Ignoriere Groß-/Kleinschreibung.
    #[default]
    Insensitive,
}

impl ToTokens for Case {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = crate_name();
        let ts = match self {
            Case::Sensitive => quote!(#crate_name::unicode::Case::Sensitive),
            Case::Insensitive => quote!(#crate_name::unicode::Case::Insensitive),
        };
        tokens.extend(ts);
    }
}

impl Case {
    /// Parse [`Case`] aus einem [`TokenStream`].
    pub(crate) fn parse(ts: &TokenStream) -> Option<Case> {
        match ts.to_string().as_str() {
            "sensitive" => Some(Case::Sensitive),
            "insensitive" => Some(Case::Insensitive),
            _ => None,
        }
    }
}

/// Ist der [`Punct`] ein einzelner [`char`]?
fn punct_is_char(punct: &Punct, char: char) -> bool {
    punct.as_char() == char && punct.spacing() == Spacing::Alone
}

/// Der Wert eines Arguments.
#[derive(Debug, Clone)]
pub(crate) enum ArgumentWert {
    /// Kein Wert.
    /// wert
    KeinWert,
    /// Ein Unterargument, abgegrenzt durch Klammern.
    /// wert(unterargument)
    Unterargument(Vec<Argument>),
    /// Ein Listenargument.
    /// wert: [elem0, elem1]
    Liste(Vec<TokenStream>),
    /// Ein allgemeines Argument.
    /// wert: argument als stream
    Stream(TokenStream),
}

/// Schreibe eine Element-Listen, abgegrenzt mit `open` und `close`.
fn write_liste<T: Display>(
    formatter: &mut Formatter<'_>,
    open: &str,
    list: impl IntoIterator<Item = T>,
    close: &str,
) -> fmt::Result {
    formatter.write_str(open)?;
    let mut first = true;
    for elem in list {
        if first {
            first = false;
        } else {
            write!(formatter, ", ")?;
        }
        write!(formatter, "{elem}")?;
    }
    formatter.write_str(close)?;
    Ok(())
}

/// Schreiben einen [`ArgumentWert`].
fn write_argument_wert(
    formatter: &mut Formatter<'_>,
    colon: bool,
    wert: &ArgumentWert,
) -> fmt::Result {
    use ArgumentWert::{KeinWert, Liste, Stream, Unterargument};
    match wert {
        KeinWert => Ok(()),
        Unterargument(args) => write_liste(formatter, "(", args, ")"),
        Liste(tts) => write_liste(formatter, if colon { ": [" } else { "[" }, tts, "]"),
        Stream(ts) => write!(formatter, "{}{ts}", if colon { ": " } else { "" }),
    }
}

impl Display for ArgumentWert {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write_argument_wert(formatter, false, self)
    }
}

/// Ein Argument für ein Feld.
#[derive(Debug, Clone)]
pub(crate) struct Argument {
    /// Der Name des Arguments.
    pub(crate) name: String,
    /// Der Wert des Arguments.
    pub(crate) wert: ArgumentWert,
}

impl Display for Argument {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let Argument { name, wert } = self;
        formatter.write_str(name)?;
        write_argument_wert(formatter, true, wert)
    }
}

/// Fehler beim trennen der Argumente.
#[derive(Debug)]
pub(crate) enum SplitArgumenteFehler {
    /// Die Argumente sind nicht in Klammern eingeschlossen.
    NichtInKlammer {
        /// Der Pfad des Arguments.
        parent: Vec<String>,
        /// Der angegebene [`TokenStream`].
        ts: TokenStream,
    },
    /// Kein Argument angegeben.
    LeeresArgument {
        /// Der Pfad des Arguments.
        parent: Vec<String>,
        /// Der angegebene [`TokenStream`].
        ts: TokenStream,
    },
    /// Invalider Name für ein Argument.
    InvaliderArgumentName {
        /// Der Pfad des Arguments.
        parent: Vec<String>,
        /// Der [`TokenTree`], wo ein Name erwartet wurde.
        tt: TokenTree,
    },
    /// Invalider Wert für ein Argument.
    InvaliderArgumentWert {
        /// Der Pfad des Arguments.
        parent: Vec<String>,
        /// Der Name des Arguments.
        name: String,
        /// Wurde der Wert über einen Doppelpunkt abgegrenzt.
        doppelpunkt: bool,
        /// Der [`TokenStream`] für den Wert.
        wert: TokenStream,
    },
}

impl Display for SplitArgumenteFehler {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        /// Schreibe den Pfad des Arguments.
        fn write_parent(
            formatter: &mut Formatter<'_>,
            parent: &[String],
            präposition: &str,
        ) -> fmt::Result {
            if !parent.is_empty() {
                write!(formatter, " {präposition} ")?;
                let mut first = true;
                for name in parent {
                    if first {
                        first = false;
                    } else {
                        write!(formatter, "::")?;
                    }
                    write!(formatter, "{name}")?;
                }
            }
            Ok(())
        }
        use SplitArgumenteFehler::{
            InvaliderArgumentName, InvaliderArgumentWert, LeeresArgument, NichtInKlammer,
        };
        match self {
            NichtInKlammer { parent, ts } => {
                write!(formatter, "Argumente")?;
                write_parent(formatter, parent, "für")?;
                write!(formatter, " nicht in Klammern eingeschlossen: {ts}")
            },
            LeeresArgument { parent, ts } => {
                write!(formatter, "Leeres Argument")?;
                write_parent(formatter, parent, "für")?;
                write!(formatter, ": {ts}")
            },
            InvaliderArgumentName { parent, tt } => {
                write!(formatter, "Invalider Name für ein Argument")?;
                write_parent(formatter, parent, "von")?;
                write!(formatter, ": {tt}")
            },
            InvaliderArgumentWert { parent, name, doppelpunkt, wert } => {
                write!(formatter, "Invalider Wert für Argument {name}")?;
                write_parent(formatter, parent, "von")?;
                write!(formatter, "!\n{name} ")?;
                if *doppelpunkt {
                    write!(formatter, ": ")?;
                }
                write!(formatter, "{wert}")
            },
        }
    }
}

/// Ist der [`TokenTree`] KEIN Komma-Character?
fn tt_is_not_comma(tt: &TokenTree) -> bool {
    if let TokenTree::Punct(punct) = tt {
        !punct_is_char(punct, ',')
    } else {
        true
    }
}

/// Argumente getrennt durch Kommas, Unterargumente mit () angegeben, potentiell mit Kommas, z.B. help.
/// Argumente können Werte haben, getrennt durch `:`.
/// Wert-Argumente können Listen (angegeben durch `[`, `]`, potentiell mit Kommas) sein.
/// Argumente werden nicht weiter behandelt.
fn split_argumente(
    parent: Vec<String>,
    args: &mut Vec<Argument>,
    args_ts: TokenStream,
) -> Result<(), SplitArgumenteFehler> {
    use SplitArgumenteFehler::{InvaliderArgumentName, InvaliderArgumentWert, LeeresArgument};
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
                        let mut acc = Vec::new();
                        let mut current = TokenStream::new();
                        for tt in group.stream() {
                            match tt {
                                TokenTree::Punct(tt_punct) if punct_is_char(&tt_punct, ',') => {
                                    acc.push(current);
                                    current = TokenStream::new();
                                },
                                TokenTree::Group(_)
                                | TokenTree::Ident(_)
                                | TokenTree::Punct(_)
                                | TokenTree::Literal(_) => current.extend(iter::once(tt)),
                            }
                        }
                        if !current.is_empty() {
                            acc.push(current);
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

/// Teile Argumente, die in Klammern eingeschlossen sind.
///
/// Argumente getrennt durch Kommas, Unterargumente mit () angegeben, potentiell mit Kommas, z.B. help.
/// Argumente können Werte haben, getrennt durch `:`.
/// Wert-Argumente können Listen (angegeben durch `[`, `]`, potentiell mit Kommas) sein.
/// Argumente werden nicht weiter behandelt.
pub(crate) fn split_klammer_argumente(
    parent: Vec<String>,
    args: &mut Vec<Argument>,
    args_ts: TokenStream,
) -> Result<(), SplitArgumenteFehler> {
    use SplitArgumenteFehler::NichtInKlammer;
    let group = match genau_eines(args_ts.into_iter()) {
        Ok(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => group,
        Ok(tt) => return Err(NichtInKlammer { parent, ts: tt.into() }),
        Err(fehler) => return Err(NichtInKlammer { parent, ts: fehler.collect() }),
    };
    split_argumente(parent, args, group.stream())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_split_argumente() {
        let mut args: Vec<Argument> = Vec::new();
        let args_ts =
            "(hello(hi), world: [it's, a, big, world!])".parse().expect("Valider TokenStream");
        let world_wert = "[it's, a, big, world!]"
            .parse::<TokenStream>()
            .expect("world_wert")
            .to_string()
            .replace(' ', "");
        let world_string = format!("world:{world_wert}");
        split_klammer_argumente(Vec::new(), &mut args, args_ts)
            .expect("Argumente sind wohlgeformt");
        let args_str: Vec<_> = args.iter().map(|arg| arg.to_string().replace(' ', "")).collect();
        assert_eq!(args_str, vec!["hello(hi)", &world_string]);
    }
}
