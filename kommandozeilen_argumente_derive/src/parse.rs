//! Implementation of the derive macro for the `Parse` trait.

use std::fmt::{self, Display, Formatter};

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use venial::{Item, Struct, parse_item};

use crate::{
    CompileError,
    parse::{
        attributes::parse_value_argument,
        fields::{FieldSettings, derive_fields, try_into_struct},
        types::{
            CreateHelp, CreateVersion, English, InvertInfix, InvertPrefix, LongPrefix,
            ParseValueError, ProgramDescription, ProgramName, ProgramSettings, ProgramVersion,
            ShortPrefix, ValueInfix,
        },
    },
    utility::{SplitArgumentsError, crate_ident, path_is_ident, split_parenthesized_arguments},
};

mod attributes;
#[macro_use]
mod fields;
mod types;

/// Unsupported type for the derive macro: only structs are supported.
#[derive(Debug)]
pub(crate) enum UnsupportedType {
    /// enum
    Enum,
    /// union
    Union,
    /// Unknown
    Unknown,
}

impl Display for UnsupportedType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use UnsupportedType::{Enum, Union, Unknown};
        formatter.write_str(match self {
            Enum => "enum",
            Union => "union",
            Unknown => "Unknown",
        })
    }
}

/// Error while parsing a struct and its attributes.
#[derive(Debug)]
pub(crate) enum Error {
    /// Error returned when a [`venial`] parser cannot parse the input tokens.
    Venial(venial::Error),
    /// Error while splitting arguments.
    SplitArguments(SplitArgumentsError),
    /// Error while parsing a value.
    ParseValue(ParseValueError),
    /// The type is not a `struct`.
    NotStruct {
        /// Parsed kind of type.
        typ: UnsupportedType,
        /// Location of the unsupported type keyword.
        span: proc_macro2::Span,
        /// Macro input.
        input: TokenStream,
    },
    /// Type with generics as macro input.
    Generics {
        /// Number of generic parameters.
        count: usize,
        /// `where` clause of the type.
        where_clause: bool,
        /// Location of the generic parameters or `where` clause.
        span: proc_macro2::Span,
    },
    /// Unnamed field in the `struct`.
    UnnamedFields(proc_macro2::Span),
    /// Field with an empty name.
    EmptyFieldName(Ident),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use Error::{
            EmptyFieldName, Generics, NotStruct, ParseValue, SplitArguments, UnnamedFields, Venial,
        };
        match self {
            Venial(error) => write!(formatter, "{error}"),
            SplitArguments(error) => write!(formatter, "{error}"),
            ParseValue(error) => write!(formatter, "{error}"),
            NotStruct { typ, input, .. } => {
                write!(formatter, "Only structs are supported, but received {typ}: {input}")
            },
            Generics { count, where_clause, .. } => {
                write!(
                    formatter,
                    "Only structs without generics are supported, but received {count} \
                     parameter(s)"
                )?;
                if *where_clause {
                    write!(formatter, " and a where clause")?;
                }
                write!(formatter, ".")
            },
            UnnamedFields(_) => formatter.write_str("Only named fields are supported."),
            EmptyFieldName(ident) => write!(formatter, "Named field has an empty name: {ident}"),
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

impl From<ParseValueError> for Error {
    fn from(input: ParseValueError) -> Error {
        Error::ParseValue(input)
    }
}

impl CompileError for Error {
    fn span(&self) -> proc_macro2::Span {
        match self {
            Self::ParseValue(ParseValueError::Unsupported { argument, .. }) => argument.span,
            Self::ParseValue(ParseValueError::NoLongName { span, .. })
            | Self::NotStruct { span, .. }
            | Self::Generics { span, .. }
            | Self::UnnamedFields(span) => *span,
            Self::Venial(error) => error.span(),
            Self::SplitArguments(error) => error.span(),
            Self::EmptyFieldName(ident) => ident.span(),
        }
    }
}

/// Implementation of the derive macro for the [`Parse`](https://docs.rs/kommandozeilen_argumente/latest/kommandozeilen_argumente/trait.Parse.html) trait.
#[expect(
    clippy::too_many_lines,
    reason = "Already split up, so this is mostly orchestration. Good enough for now."
)]
pub(crate) fn derive_parse(input: TokenStream) -> Result<TokenStream, Error> {
    use Error::Generics;
    let item = parse_item(input.clone())?;
    let Struct { fields, name, generic_params, where_clause, attributes, .. } =
        try_into_struct(input, item)?;
    let param_count = generic_params.as_ref().map_or(0, |param_list| param_list.params.len());
    let has_where_clause = where_clause.is_some();
    if (param_count > 0) || has_where_clause {
        let span = generic_params.as_ref().map_or_else(
            || {
                where_clause
                    .expect("where clause exists when generic parameters do not")
                    .tk_where
                    .span()
            },
            |params| params.tk_l_bracket.span(),
        );
        return Err(Generics { count: param_count, where_clause: has_where_clause, span });
    }
    let mut args = Vec::new();
    for attr in attributes {
        if path_is_ident(&attr, "kommandozeilen_argumente") {
            split_parenthesized_arguments(Vec::new(), &mut args, attr.value)?;
        }
    }
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    // let version = env!("CARGO_PKG_VERSION");
    // CARGO_PKG_NAME — The name of your package.
    // CARGO_PKG_VERSION — The full version of your package.
    // CARGO_PKG_AUTHORS — Colon separated list of authors from the manifest of your package.
    // CARGO_PKG_DESCRIPTION — The description from the manifest of your package.
    // CARGO_BIN_NAME — The name of the binary that is currently being compiled (if it is a binary).
    // This name does not include any file extension, such as .exe
    let mut create_version = CreateVersion(None);
    let mut create_help = CreateHelp(None);
    let mut language = None;
    let mut default_program_setttings = ProgramSettings {
        name: ProgramName(None),
        version: ProgramVersion::Unspecified,
        description: ProgramDescription(None),
    };
    let mut long_prefix = LongPrefix::default();
    let mut short_prefix = ShortPrefix::default();
    let mut invert_prefix = InvertPrefix::default();
    let mut invert_infix = InvertInfix::default();
    let mut value_infix = ValueInfix::default();
    let mut meta_var = None;
    let crate_ident = crate_ident();
    unwrap_or_return!(
        parse_value_argument(
            args,
            Some(&mut language),
            Some(&mut default_program_setttings),
            Some(&mut create_help),
            Some(&mut create_version),
            None,
            Some(&mut long_prefix),
            None,
            Some(&mut short_prefix),
            None,
            Some(&mut invert_prefix),
            Some(&mut invert_infix),
            Some(&mut value_infix),
            Some(&mut meta_var),
            None,
            None,
        ),
        None
    );
    let language = language.unwrap_or(English);
    let meta_var = if let Some(meta_var) = meta_var {
        quote!(#meta_var)
    } else {
        let language_tokens = language.token_stream();
        quote!(#language_tokens.meta_var)
    };
    let tuples = derive_fields(
        fields,
        &FieldSettings {
            language: &language,
            long_prefix: &long_prefix,
            short_prefix: &short_prefix,
            invert_prefix: &invert_prefix,
            invert_infix: &invert_infix,
            value_infix: &value_infix,
            meta_var: &meta_var,
        },
    )?;
    let (idents, create_arguments): (Vec<_>, Vec<_>) = tuples.into_iter().unzip();
    let combined = quote!(
        #(
            let #idents = ::#crate_ident::Arguments::from(#create_arguments);
        )*
        ::#crate_ident::combine!(|#(#idents),*| Self {#(#idents),*}, #(#idents),*)
    );
    let with_version = if let CreateVersion(Some(add_version)) = create_version {
        add_version(
            combined,
            language,
            &default_program_setttings.name,
            &default_program_setttings.version,
        )
    } else {
        combined
    };
    let with_help = if let CreateHelp(Some(add_help)) = create_help {
        add_help(
            with_version,
            &default_program_setttings.name,
            &default_program_setttings.version,
            &default_program_setttings.description,
        )
    } else {
        with_version
    };
    let ts = quote! {
        #[expect(
            clippy::shadow_unrelated,
            reason = "Generated bindings use user-provided field names."
        )]
        impl ::#crate_ident::Parse for #name {
            type Error = String;

            fn arguments<'t>() -> ::#crate_ident::Arguments<'t, Self, Self::Error> {
                #with_help
            }
        }
    };
    Ok(ts)
}

#[cfg(test)]
mod test {
    use quote::quote;

    use super::*;

    #[test]
    fn derives_named_fields_and_configured_early_exits() {
        let derived = derive_parse(quote! {
            #[kommandozeilen_argumente(
                language: english,
                program(name: "app", version: "1.2.3", description: "An app"),
                version,
                help,
            )]
            struct Arguments {
                /// Enable the feature.
                #[kommandozeilen_argumente(long: "feature", short: f, default: false)]
                feature: bool,
            }
        })
        .expect("supported struct derives successfully")
        .to_string();

        assert!(derived.contains("impl :: kommandozeilen_argumente :: Parse for Arguments"));
        assert!(derived.contains("Description :: new"));
        assert!(derived.contains("\"feature\""));
        assert!(derived.contains("with_version_early_exit"));
        assert!(derived.contains("with_help_early_exit"));
        assert!(derived.contains("\"app\""));
        assert!(derived.contains("\"1.2.3\""));
        assert!(derived.contains("Some (\"An app\")"));
    }

    #[test]
    fn rejects_tuple_fields() {
        let error = derive_parse(quote! { struct Arguments(bool); })
            .expect_err("tuple fields are unsupported");

        assert_eq!(error.to_string(), "Only named fields are supported.");
    }

    #[test]
    fn rejects_generics_and_non_structs() {
        let generic_error = derive_parse(quote! { struct Arguments<T> { value: T } })
            .expect_err("generic structs are unsupported");
        assert_eq!(
            generic_error.to_string(),
            "Only structs without generics are supported, but received 1 parameter(s)."
        );

        let enum_error = derive_parse(quote! { enum Arguments { Value } })
            .expect_err("only structs are supported");
        assert_eq!(
            enum_error.to_string(),
            "Only structs are supported, but received enum: enum Arguments { Value }"
        );
    }
}
