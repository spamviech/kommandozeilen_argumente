//! Implementierung für das derive-Macro des EnumArgument-Traits.

use std::fmt::{self, Display, Formatter};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Attribute, Data, DataEnum, DeriveInput, Fields, Ident, Variant};

use crate::utility::{
    base_name, split_klammer_argumente, Argument, ArgumentWert, Case, SplitArgumenteFehler,
};

#[derive(Debug)]
pub(crate) enum TypNichtUnterstützt {
    Struct,
    Union,
}

impl Display for TypNichtUnterstützt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use TypNichtUnterstützt::*;
        f.write_str(match self {
            Struct => "struct",
            Union => "union",
        })
    }
}

pub(crate) enum Fehler {
    Syn(syn::Error),
    KeinEnum { typ: TypNichtUnterstützt, input: TokenStream },
    Generics { anzahl: usize, where_clause: bool },
    DatenVariante { variante: Ident },
    SplitArgumente(SplitArgumenteFehler),
    NichtUnterstützt(Argument),
}

impl Display for Fehler {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentWert::*;
        use Fehler::*;
        match self {
            Syn(error) => write!(f, "{error}"),
            KeinEnum { typ, input } => {
                write!(f, "Nur structs unterstützt, aber {typ} bekommen: {input}")
            },
            Generics { anzahl, where_clause } => {
                write!(f, "Nur Structs ohne Generics unterstützt, aber {anzahl} Parameter ")?;
                if *where_clause {
                    write!(f, "und eine where-Klausel ")?;
                }
                write!(f, "bekommen.")
            },
            DatenVariante { variante } => {
                write!(f, "Nur Enums mit Unit-Varianten unterstützt, aber {variante} hält Daten.")
            },
            SplitArgumente(fehler) => write!(f, "{fehler}"),
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

fn parse_attributes(feld: Option<&Ident>, attrs: Vec<Attribute>) -> Result<Option<Case>, Fehler> {
    let mut args = Vec::new();
    for attr in attrs {
        if attr.path.is_ident("kommandozeilen_argumente") {
            split_klammer_argumente(
                feld.iter().map(ToString::to_string).collect(),
                &mut args,
                attr.tokens,
            )?;
        }
    }
    let mut case = None;
    for arg in args {
        match arg {
            Argument { name, wert: ArgumentWert::Stream(ts) } if name == "case" => {
                case = Some(Case::parse(ts).map_err(|ts| {
                    Fehler::NichtUnterstützt(Argument { name, wert: ArgumentWert::Stream(ts) })
                })?)
            },
            _ => return Err(Fehler::NichtUnterstützt(arg)),
        }
    }
    Ok(case)
}

pub(crate) fn derive_enum_argument(input: TokenStream) -> Result<TokenStream, Fehler> {
    use Fehler::*;
    use TypNichtUnterstützt::*;
    let DeriveInput { ident, data, generics, attrs, .. } = parse2(input.clone())?;
    let DataEnum { variants, .. } = match data {
        Data::Enum(data_enum) => data_enum,
        Data::Struct(_) => return Err(KeinEnum { typ: Struct, input }),
        Data::Union(_) => return Err(KeinEnum { typ: Union, input }),
    };
    let crate_name = base_name();
    let has_where_clause = generics.where_clause.is_some();
    if !generics.params.is_empty() || has_where_clause {
        return Err(Generics { anzahl: generics.params.len(), where_clause: has_where_clause });
    }
    let standard_case = parse_attributes(None, attrs)?;
    let mut varianten = Vec::new();
    let mut cases = Vec::new();
    for Variant { ident, fields, attrs, .. } in variants {
        if let Fields::Unit = fields {
            let case = parse_attributes(Some(&ident), attrs)?;
            cases.push(case.or(standard_case).unwrap_or_default());
            varianten.push(ident);
        } else {
            return Err(DatenVariante { variante: ident });
        }
    }
    let varianten_str: Vec<_> = varianten.iter().map(ToString::to_string).collect();
    let instance = quote!(
        impl #crate_name::EnumArgument for #ident {
            fn varianten() -> Vec<Self> {
                vec![#(Self::#varianten),*]
            }

            fn parse_enum(arg: &std::ffi::OsStr) -> Result<Self, #crate_name::ParseFehler<String>> {
                if let Some(string) = arg.to_str() {
                    #(
                        if #crate_name::unicode::Normalisiert::neu(#varianten_str).eq(string, #cases)
                        {
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
    Ok(instance)
}
