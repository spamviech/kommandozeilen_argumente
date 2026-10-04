//! Implementation of the derive macro for the `EnumArgument` trait.

use std::fmt::{self, Display, Formatter};

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use venial::{Attribute, Enum, EnumVariant, Fields, Item, parse_item};

use crate::utility::{
    Argument, ArgumentValue, Case, SplitArgumentsError, crate_ident, path_is_ident,
    split_parenthesized_arguments,
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
        use UnsupportedType::{Struct, Union, Unknown};
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
        count: usize,
        /// `where` clause of the type.
        where_clause: bool,
    },
    /// A variant with data was found.
    DataVariant {
        /// Field name.
        variant: Ident,
    },
    /// Error while splitting arguments.
    SplitArguments(SplitArgumentsError),
    /// The attribute is unsupported.
    Unsupported(Argument),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentValue::{List, NoValue, Stream, SubArgument};
        use Error::{DataVariant, Generics, NotEnum, SplitArguments, Unsupported, Venial};
        match self {
            Venial(error) => write!(formatter, "{error}"),
            NotEnum { typ, input } => {
                write!(formatter, "Only enums are supported, but received {typ}: {input}")
            },
            Generics { count, where_clause } => {
                write!(
                    formatter,
                    "Only enums without generics are supported, but received {count} parameter(s)"
                )?;
                if *where_clause {
                    write!(formatter, " and a where clause")?;
                }
                write!(formatter, ".")
            },
            DataVariant { variant } => {
                write!(formatter, "Only unit variants are supported, but {variant} contains data.")
            },
            SplitArguments(error) => write!(formatter, "{error}"),
            Unsupported(Argument { name, value: NoValue }) => {
                write!(formatter, "Unsupported argument: {name}")
            },
            Unsupported(Argument { name, value: wert @ SubArgument(_sub_args) }) => {
                write!(formatter, "Unsupported sub-argument of {name}: {wert}")
            },
            Unsupported(Argument { name, value: wert @ List(_list) }) => {
                write!(formatter, "Unsupported list argument {name}: {wert}")
            },
            Unsupported(Argument { name, value: wert @ Stream(_tokens) }) => {
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
            Argument { name, value: ArgumentValue::Stream(ts) } if name == "case" => {
                case = Some(Case::parse(&ts).ok_or({
                    Error::Unsupported(Argument { name, value: ArgumentValue::Stream(ts) })
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
        return Err(Generics { count: param_count, where_clause: has_where_clause });
    }
    let standard_case = parse_attributes(None, attributes)?;
    let mut idents = Vec::new();
    let mut cases = Vec::new();
    for (enum_variant, _punct) in variants.inner {
        let EnumVariant { name: variant_ident, fields, attributes: variant_attrs, .. } =
            enum_variant;
        if let Fields::Unit = fields {
            let case = parse_attributes(Some(&variant_ident), variant_attrs)?;
            cases.push(case.or(standard_case).unwrap_or_default());
            idents.push(variant_ident);
        } else {
            return Err(DataVariant { variant: variant_ident });
        }
    }
    let idents_ts = if idents.is_empty() {
        quote!(None)
    } else {
        quote!(Some(::#crate_ident::nonempty![#(Self::#idents),*]))
    };
    let idents_str: Vec<_> = idents.iter().map(ToString::to_string).collect();
    let instance = quote!(
        impl #crate_ident::EnumArgument for #name {
            fn varianten() -> Option<::#crate_ident::NonEmpty<Self>> {
                #idents_ts
            }

            fn parse_enum(
                arg: &::std::ffi::OsStr,
            ) -> ::std::result::Result<Self, ::#crate_ident::ParseError<String>> {
                if let Some(string) = arg.to_str() {
                    #(
                        if ::#crate_ident::unicode::Normalized::new(#idents_str).eq_with_case(string, #cases)
                        {
                            Ok(Self::#idents)
                        } else
                    )*
                    {
                        Err(::#crate_ident::ParseError::ParseError(
                            format!("Unknown variant: {}", string)
                        ))
                    }
                } else {
                    Err(::#crate_ident::ParseError::InvalidString(::std::ffi::OsString::from(arg)))
                }
            }
        }
    );
    Ok(instance)
}
