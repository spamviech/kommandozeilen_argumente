//! Implementierung für das derive-Macro des EnumArgument-Traits.

use std::fmt::{self, Display, Formatter};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Data, DataEnum, DeriveInput, Fields, Ident};

use crate::base_name;

#[derive(Debug)]
enum TypNichtUnterstützt {
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

enum Fehler {
    Syn(syn::Error),
    KeinEnum { typ: TypNichtUnterstützt, input: TokenStream },
    Generics { anzahl: usize, where_clause: bool },
    DatenVariante { variante: Ident },
}

impl Display for Fehler {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
        }
    }
}

impl From<syn::Error> for Fehler {
    fn from(input: syn::Error) -> Fehler {
        Fehler::Syn(input)
    }
}

pub(crate) fn derive_enum_argument(input: TokenStream) -> Result<TokenStream, Fehler> {
    use Fehler::*;
    let DeriveInput { ident, data, generics, .. } = parse2(input)?;
    let DataEnum { variants, .. } = match data {
        Data::Enum(data_enum) => data_enum,
        _ => todo!(),
    };
    let crate_name = base_name();
    let has_where_clause = generics.where_clause.is_some();
    if !generics.params.is_empty() || has_where_clause {
        return Err(Generics { anzahl: generics.params.len(), where_clause: has_where_clause });
    }
    let mut varianten = Vec::new();
    for variant in variants {
        if let Fields::Unit = variant.fields {
            varianten.push(variant.ident);
        } else {
            return Err(DatenVariante { variante: variant.ident });
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
    Ok(instance)
}
