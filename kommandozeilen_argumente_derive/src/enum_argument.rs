//! Implementierung für das derive-Macro des EnumArgument-Traits.

use std::fmt::{self, Display, Formatter};

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use venial::{parse_item, Attribute, Enum, EnumVariant, Fields, Item};

use crate::utility::{
    crate_name, path_is_ident, split_klammer_argumente, Argument, ArgumentWert, Case,
    SplitArgumenteFehler,
};

/// Nicht unterstützter Typ für das derive-Macro: Nur enums sind unterstützt.
#[derive(Debug)]
pub(crate) enum TypNichtUnterstützt {
    /// struct
    Struct,
    /// union
    Union,
    /// Unbekannt
    Unbekannt,
}

impl Display for TypNichtUnterstützt {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use TypNichtUnterstützt::{Struct, Unbekannt, Union};
        formatter.write_str(match self {
            Struct => "struct",
            Union => "union",
            Unbekannt => "Unbekannt",
        })
    }
}

/// Fehler beim Parsen des enums inklusive Attribute.
pub(crate) enum Fehler {
    /// Error returned when a [`syn`] parser cannot parse the input tokens.
    Venial(venial::Error),
    /// Der Typ ist kein `enum`.
    KeinEnum {
        /// Die geparste Typ-Art.
        typ: TypNichtUnterstützt,
        /// Der Macro-Input.
        input: TokenStream,
    },
    /// Typ mit Generics als Macro-Argument.
    Generics {
        /// Anzahl der Generic-Parameter.
        anzahl: usize,
        /// `where`-Klausel des Typs.
        where_clause: bool,
    },
    /// Eine Variante mit Daten gefunden.
    DatenVariante {
        /// Der Feld-Name.
        variante: Ident,
    },
    /// Fehler beim teilen der Argumente.
    SplitArgumente(SplitArgumenteFehler),
    /// Das Attribut wurde nicht unterstützt.
    NichtUnterstützt(Argument),
}

impl Display for Fehler {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentWert::{KeinWert, Liste, Stream, Unterargument};
        use Fehler::{
            DatenVariante, Generics, KeinEnum, NichtUnterstützt, SplitArgumente, Venial
        };
        match self {
            Venial(error) => write!(formatter, "{error}"),
            KeinEnum { typ, input } => {
                write!(formatter, "Nur structs unterstützt, aber {typ} bekommen: {input}")
            },
            Generics { anzahl, where_clause } => {
                write!(
                    formatter,
                    "Nur Structs ohne Generics unterstützt, aber {anzahl} Parameter "
                )?;
                if *where_clause {
                    write!(formatter, "und eine where-Klausel ")?;
                }
                write!(formatter, "bekommen.")
            },
            DatenVariante { variante } => {
                write!(
                    formatter,
                    "Nur Enums mit Unit-Varianten unterstützt, aber {variante} hält Daten."
                )
            },
            SplitArgumente(fehler) => write!(formatter, "{fehler}"),
            NichtUnterstützt(Argument { name, wert: KeinWert }) => {
                write!(formatter, "Argument nicht unterstützt: {name}")
            },
            NichtUnterstützt(Argument { name, wert: wert @ Unterargument(_sub_args) }) => {
                write!(formatter, "Unterargument von {name} nicht unterstützt: {wert}")
            },
            NichtUnterstützt(Argument { name, wert: wert @ Liste(_liste) }) => {
                write!(formatter, "Listen-Argument {name} nicht unterstützt: {wert}")
            },
            NichtUnterstützt(Argument { name, wert: wert @ Stream(_ts) }) => {
                write!(formatter, "Benanntes Argument {name} nicht unterstützt: {wert}")
            },
        }
    }
}

impl From<venial::Error> for Fehler {
    fn from(input: venial::Error) -> Fehler {
        Fehler::Venial(input)
    }
}

impl From<SplitArgumenteFehler> for Fehler {
    fn from(input: SplitArgumenteFehler) -> Fehler {
        Fehler::SplitArgumente(input)
    }
}

/// Parse Attribute beim enum oder einer Variante.
fn parse_attributes(feld: Option<&Ident>, attrs: Vec<Attribute>) -> Result<Option<Case>, Fehler> {
    let mut args = Vec::new();
    for attr in attrs {
        if path_is_ident(&attr, "kommandozeilen_argumente") {
            split_klammer_argumente(
                feld.iter().map(ToString::to_string).collect(),
                &mut args,
                attr.value,
            )?;
        }
    }
    let mut case = None;
    for arg in args {
        match arg {
            Argument { name, wert: ArgumentWert::Stream(ts) } if name == "case" => {
                case = Some(Case::parse(&ts).ok_or({
                    Fehler::NichtUnterstützt(Argument { name, wert: ArgumentWert::Stream(ts) })
                })?);
            },
            _ => return Err(Fehler::NichtUnterstützt(arg)),
        }
    }
    Ok(case)
}

/// Implementierung für das derive-Macro des [`EnumArgument`]-traits.
pub(crate) fn derive_enum_argument(input: TokenStream) -> Result<TokenStream, Fehler> {
    use Fehler::{DatenVariante, Generics, KeinEnum};
    let item = parse_item(input.clone())?;
    // Item als #[non_exhaustive] markiert
    #[allow(clippy::wildcard_enum_match_arm)]
    let Enum { variants, name, generic_params, where_clause, attributes, .. } = match item {
        Item::Enum(enum_) => enum_,
        Item::Struct(_) => return Err(KeinEnum { typ: TypNichtUnterstützt::Struct, input }),
        Item::Union(_) => return Err(KeinEnum { typ: TypNichtUnterstützt::Union, input }),
        _ => return Err(KeinEnum { typ: TypNichtUnterstützt::Unbekannt, input }),
    };

    let crate_name = crate_name();
    let param_count = generic_params.map_or(0, |param_list| param_list.params.len());
    let has_where_clause = where_clause.is_some();
    if (param_count > 0) || has_where_clause {
        return Err(Generics { anzahl: param_count, where_clause: has_where_clause });
    }
    let standard_case = parse_attributes(None, attributes)?;
    let mut varianten = Vec::new();
    let mut cases = Vec::new();
    for (enum_variant, _punct) in variants.inner {
        let EnumVariant { name: variant_ident, fields, attributes: variant_attrs, .. } =
            enum_variant;
        if let Fields::Unit = fields {
            let case = parse_attributes(Some(&variant_ident), variant_attrs)?;
            cases.push(case.or(standard_case).unwrap_or_default());
            varianten.push(variant_ident);
        } else {
            return Err(DatenVariante { variante: variant_ident });
        }
    }
    let varianten_ts = if varianten.is_empty() {
        quote!(None)
    } else {
        quote!(Some(::#crate_name::nonempty![#(Self::#varianten),*]))
    };
    let varianten_str: Vec<_> = varianten.iter().map(ToString::to_string).collect();
    let instance = quote!(
        impl #crate_name::EnumArgument for #name {
            fn varianten() -> Option<::#crate_name::NonEmpty<Self>> {
                #varianten_ts
            }

            fn parse_enum(arg: &::std::ffi::OsStr) -> Result<Self, ::#crate_name::ParseFehler<String>> {
                if let Some(string) = arg.to_str() {
                    #(
                        if ::#crate_name::unicode::Normalisiert::neu(#varianten_str).eq_mit_case(string, #cases)
                        {
                            Ok(Self::#varianten)
                        } else
                    )*
                    {
                        Err(::#crate_name::ParseFehler::ParseFehler(
                            format!("Unbekannte Variante: {}", string))
                        )
                    }
                } else {
                    Err(::#crate_name::ParseFehler::InvaliderString(::std::ffi::OsString::from(arg)))
                }
            }
        }
    );
    Ok(instance)
}
