//! Implementation of the derive macro for the `EnumArgument` trait.

use std::fmt::{self, Display, Formatter};

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use venial::{parse_item, Attribute, Enum, EnumVariant, Fields, Item};

use crate::utility::{
    crate_ident, path_is_ident, split_parenthesized_arguments, Argument, ArgumentValue, Case,
    SplitArgumentsError,
};

/// Unsupported type for the derive macro: only enums are supported.
#[derive(Debug)]
pub(crate) enum UnsupportedType {
    /// struct
    Struct,
    /// union
    Union,
    /// Unknown
    Unknown,
}

impl Display for UnsupportedType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use UnsupportedType::{Struct, Unknown, Union};
        formatter.write_str(match self {
            Struct => "struct",
            Union => "union",
            Unknown => "Unknown",
        })
    }
}

/// Error while parsing an enum and its attributes.
pub(crate) enum Error {
    /// Error returned when a [`syn`] parser cannot parse the input tokens.
    Venial(venial::Error),
    /// The type is not an `enum`.
    NotEnum {
        /// Parsed kind of type.
        typ: UnsupportedType,
        /// Macro input.
        input: TokenStream,
    },
    /// Type with generics as macro input.
    Generics {
        /// Number of generic parameters.
        anzahl: usize,
        /// `where` clause of the type.
        where_clause: bool,
    },
    /// A variant with data was found.
    DataVariant {
        /// Field name.
        variante: Ident,
    },
    /// Error while splitting arguments.
    SplitArguments(SplitArgumentsError),
    /// The attribute is unsupported.
    Unsupported(Argument),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentValue::{NoValue, List, Stream, SubArgument};
        use Error::{
            DataVariant, Generics, NotEnum, Unsupported, SplitArguments, Venial
        };
        match self {
            Venial(error) => write!(formatter, "{error}"),
            NotEnum { typ, input } => {
                write!(formatter, "Only enums are supported, but received {typ}: {input}")
            },
            Generics { anzahl, where_clause } => {
                write!(formatter, "Only enums without generics are supported, but received {anzahl} parameter(s)")?;
                if *where_clause {
                    write!(formatter, " and a where clause")?;
                }
                write!(formatter, ".")
            },
            DataVariant { variante } => {
                write!(formatter, "Only unit variants are supported, but {variante} contains data.")
            },
            SplitArguments(error) => write!(formatter, "{error}"),
            Unsupported(Argument { name, wert: NoValue }) => {
                write!(formatter, "Unsupported argument: {name}")
            },
            Unsupported(Argument { name, wert: wert @ SubArgument(_sub_args) }) => {
                write!(formatter, "Unsupported sub-argument of {name}: {wert}")
            },
            Unsupported(Argument { name, wert: wert @ List(_list) }) => {
                write!(formatter, "Unsupported list argument {name}: {wert}")
            },
            Unsupported(Argument { name, wert: wert @ Stream(_tokens) }) => {
                write!(formatter, "Unsupported named argument {name}: {wert}")
            },
        }
    }
}

impl From<venial::Error> for Error {
    fn from(input: venial::Error) -> Error {
        Error::Venial(input)
    }
}

impl From<SplitArgumentsError> for Error {
    fn from(input: SplitArgumentsError) -> Error {
        Error::SplitArguments(input)
    }
}

/// Parse attributes on an enum or a variant.
fn parse_attributes(field: Option<&Ident>, attrs: Vec<Attribute>) -> Result<Option<Case>, Error> {
    let mut args = Vec::new();
    for attr in attrs {
        if path_is_ident(&attr, "kommandozeilen_argumente") {
            split_parenthesized_arguments(
                field.iter().map(ToString::to_string).collect(),
                &mut args,
                attr.value,
            )?;
        }
    }
    let mut case = None;
    for arg in args {
        match arg {
            Argument { name, wert: ArgumentValue::Stream(ts) } if name == "case" => {
                case = Some(Case::parse(&ts).ok_or({
                    Error::Unsupported(Argument { name, wert: ArgumentValue::Stream(ts) })
                })?);
            },
            _ => return Err(Error::Unsupported(arg)),
        }
    }
    Ok(case)
}

/// Implementation of the derive macro for the [`EnumArgument`] trait.
pub(crate) fn derive_enum_argument(input: TokenStream) -> Result<TokenStream, Error> {
    use Error::{DataVariant, Generics, NotEnum};
    let item = parse_item(input.clone())?;
    // Item als #[non_exhaustive] markiert
    #[allow(clippy::wildcard_enum_match_arm)]
    let Enum { variants, name, generic_params, where_clause, attributes, .. } = match item {
        Item::Enum(enum_) => enum_,
        Item::Struct(_) => return Err(NotEnum { typ: UnsupportedType::Struct, input }),
        Item::Union(_) => return Err(NotEnum { typ: UnsupportedType::Union, input }),
        _ => return Err(NotEnum { typ: UnsupportedType::Unknown, input }),
    };

    let crate_ident = crate_ident();
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
            return Err(DataVariant { variante: variant_ident });
        }
    }
    let varianten_ts = if varianten.is_empty() {
        quote!(None)
    } else {
        quote!(Some(::#crate_ident::nonempty![#(Self::#varianten),*]))
    };
    let varianten_str: Vec<_> = varianten.iter().map(ToString::to_string).collect();
    let instance = quote!(
        impl #crate_ident::EnumArgument for #name {
            fn varianten() -> Option<::#crate_ident::NonEmpty<Self>> {
                #varianten_ts
            }

            fn parse_enum(arg: &::std::ffi::OsStr) -> Result<Self, ::#crate_ident::ParseFehler<String>> {
                if let Some(string) = arg.to_str() {
                    #(
                        if ::#crate_ident::unicode::Normalized::new(#varianten_str).eq_with_case(string, #cases)
                        {
                            Ok(Self::#varianten)
                        } else
                    )*
                    {
                        Err(::#crate_ident::ParseFehler::ParseFehler(
                            format!("Unbekannte Variante: {}", string))
                        )
                    }
                } else {
                    Err(::#crate_ident::ParseFehler::InvaliderString(::std::ffi::OsString::from(arg)))
                }
            }
        }
    );
    Ok(instance)
}
