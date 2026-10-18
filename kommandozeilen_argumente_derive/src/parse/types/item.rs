//! Parsed derive inputs and related errors.

use std::fmt::{self, Display, Formatter};

use proc_macro2::TokenStream;
use venial::{Attribute, Fields, GenericParamList, Item, WhereClause};

use crate::{
    parse::{Error, UnsupportedType},
    utility::{Argument, ArgumentValue},
};

/// The fields needed to derive [`Parse`](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.Parse.html).
pub(in crate::parse) struct ParseItem {
    /// Name of the struct or enum.
    pub(in crate::parse) name: proc_macro2::Ident,
    /// Generic parameters, which are not supported by the derive macro.
    pub(in crate::parse) generic_params: Option<GenericParamList>,
    /// `where` clause, which is not supported by the derive macro.
    pub(in crate::parse) where_clause: Option<WhereClause>,
    /// Attributes applied to the struct or enum.
    pub(in crate::parse) attributes: Vec<Attribute>,
    /// Named alternatives, where `None` represents the one struct alternative.
    pub(in crate::parse) variants: Vec<(Option<proc_macro2::Ident>, Fields)>,
}

/// Extract a supported item from a parsed [`Item`].
pub(in crate::parse) fn try_into_parse_item(
    input: TokenStream,
    item: Item,
) -> Result<ParseItem, Error> {
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "The venial item enum is non-exhaustive; only structs and enums are supported."
    )]
    match item {
        Item::Struct(struct_) => Ok(ParseItem {
            name: struct_.name,
            generic_params: struct_.generic_params,
            where_clause: struct_.where_clause,
            attributes: struct_.attributes,
            variants: vec![(None, struct_.fields)],
        }),
        Item::Enum(enum_) => Ok(ParseItem {
            name: enum_.name,
            generic_params: enum_.generic_params,
            where_clause: enum_.where_clause,
            attributes: enum_.attributes,
            variants: enum_
                .variants
                .inner
                .into_iter()
                .map(|(variant, _punct)| (Some(variant.name), variant.fields))
                .collect(),
        }),
        Item::Union(union) => Err(Error::NotStruct {
            typ: UnsupportedType::Union,
            span: union.tk_union.span(),
            input,
        }),
        _ => Err(Error::NotStruct {
            typ: UnsupportedType::Unknown,
            span: input
                .clone()
                .into_iter()
                .next()
                .map_or_else(proc_macro2::Span::call_site, |tt| tt.span()),
            input,
        }),
    }
}

/// Error while parsing a value attribute.
#[derive(Debug)]
pub(crate) enum ParseValueError {
    /// The attribute is unsupported.
    Unsupported {
        /// Name of the field on which the argument was specified.
        arg_name: Option<String>,
        /// The unknown argument.
        argument: Argument,
    },
    /// No long name was specified.
    NoLongName {
        /// Name of the field on which the argument was specified.
        arg_name: Option<String>,
        /// String parsed as a long name.
        name: String,
        /// Location of the empty long-name list.
        span: proc_macro2::Span,
    },
}

impl Display for ParseValueError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentValue::{List, NoValue, Stream, SubArgument};
        use ParseValueError::{NoLongName, Unsupported};
        match self {
            Unsupported { arg_name, argument: Argument { name, value: NoValue, .. } } => {
                write!(formatter, "Unsupported argument")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {name}")
            },
            Unsupported {
                arg_name,
                argument: Argument { name, value: wert @ SubArgument(_), .. },
            } => {
                write!(formatter, "Unsupported sub-argument of {name}")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {wert}")
            },
            Unsupported { arg_name, argument: Argument { name, value: wert @ List(_), .. } } => {
                write!(formatter, "Unsupported list argument {name}")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {wert}")
            },
            Unsupported { arg_name, argument: Argument { name, value: wert @ Stream(_), .. } } => {
                write!(formatter, "Unsupported named argument {name}")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {wert}")
            },
            NoLongName { arg_name, name, .. } => {
                write!(formatter, "No long name")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, " specified in explicit list containing {name}")
            },
        }
    }
}

/// Function that creates a [`ParseValueError`] from an argument name.
pub(in crate::parse) type CreateError = Box<dyn FnOnce(Option<String>) -> ParseValueError>;
