//! Field parsing and argument generation for `#[derive(Parse)]`.

use std::iter;

use venial::{Attribute, Fields, NamedField};

use crate::parse::{
    Error, Ident, TokenStream, crate_ident, parse_value_argument, path_is_ident, quote,
    split_parenthesized_arguments,
    types::{
        DefaultValue, FieldArgument, InvertInfix, InvertPrefix, Language, LongNames, LongPrefix,
        MetaVar, ShortNames, ShortPrefix, ValueInfix,
    },
};

/// Obtain the [`Ok`] value or return the result of calling the [`Err`] value immediately.
macro_rules! unwrap_or_return {
    ($result:expr $(, $($arg:expr)+)? $(,)?) => {
        match $result {
            Ok(wert) => wert,
            Err(function) => return Err(function($($($arg)+)?).into()),
        }
    };
}

/// Shared settings used to derive the arguments for each field.
pub(super) struct FieldSettings<'a> {
    /// Effective language of the derive invocation.
    pub(super) language: &'a Language,
    /// Default prefix for long field names.
    pub(super) long_prefix: &'a LongPrefix,
    /// Default prefix for short field names.
    pub(super) short_prefix: &'a ShortPrefix,
    /// Default prefix for inverted field names.
    pub(super) invert_prefix: &'a InvertPrefix,
    /// Default infix for inverted field names.
    pub(super) invert_infix: &'a InvertInfix,
    /// Default infix preceding field values.
    pub(super) value_infix: &'a ValueInfix,
    /// Default metavariable used in field help text.
    pub(super) meta_var: &'a TokenStream,
}

/// Mutable state assembled from a field's attributes.
struct FieldAttributeSettings {
    /// Documentation fragments used as help text.
    help_literals: Vec<String>,
    /// Long argument names.
    long_names: TokenStream,
    /// Short argument names.
    short_names: TokenStream,
    /// Prefix for long argument names.
    long_prefix: LongPrefix,
    /// Prefix for short argument names.
    short_prefix: ShortPrefix,
    /// Prefix for inverted argument names.
    invert_prefix: InvertPrefix,
    /// Infix for inverted argument names.
    invert_infix: InvertInfix,
    /// Infix preceding argument values.
    value_infix: ValueInfix,
    /// Metavariable for argument values.
    meta_var: Option<MetaVar>,
    /// Default value for the argument.
    default_value: DefaultValue,
    /// Strategy for parsing the argument value.
    argument: FieldArgument,
}

/// Parse one documentation or `kommandozeilen_argumente` attribute for a field.
fn parse_field_attribute(
    attr: Attribute,
    field_ident: &Ident,
    field_ident_str: &str,
    crate_ident: &Ident,
    settings: &mut FieldAttributeSettings,
) -> Result<(), Error> {
    if path_is_ident(&attr, "doc") {
        let args_str = quote!(#(attr.meta)).to_string();
        if let Some(stripped) =
            args_str.strip_prefix("= \"").and_then(|string| string.strip_suffix('\"'))
        {
            let trimmed = stripped.trim();
            if !trimmed.is_empty() {
                settings.help_literals.push(trimmed.to_owned());
            }
        }
    } else if path_is_ident(&attr, "kommandozeilen_argumente") {
        let mut field_args = Vec::new();
        split_parenthesized_arguments(vec![field_ident.to_string()], &mut field_args, attr.value)?;
        let mut long_names = LongNames::default();
        let mut short_names = ShortNames::default();
        unwrap_or_return!(
            parse_value_argument(
                field_args,
                None,
                None,
                None,
                None,
                None,
                Some(&mut settings.long_prefix),
                Some(&mut long_names),
                Some(&mut settings.short_prefix),
                Some(&mut short_names),
                Some(&mut settings.invert_prefix),
                Some(&mut settings.invert_infix),
                Some(&mut settings.value_infix),
                Some(&mut settings.meta_var),
                Some(&mut settings.default_value),
                Some(&mut settings.argument),
            ),
            Some(field_ident_str.to_owned())
        );
        let first = if let Some((head, tail)) = long_names.namen.as_ref() {
            settings.long_names = if let Some(case) = long_names.case {
                quote!(
                    ::#crate_ident::NonEmpty {
                        head: ::#crate_ident::Compare {
                            string: ::#crate_ident::Normalized::new(#head),
                            case: #case,
                        },
                        tail: vec![#(::#crate_ident::Compare {
                            string: ::#crate_ident::Normalized::new(#tail),
                            case: #case,
                        }),*],
                    }
                )
            } else {
                quote!(
                    ::#crate_ident::NonEmpty {
                        head: #head,
                        tail: vec![#(#tail),*]
                    }
                )
            };
            head
        } else {
            settings.long_names = quote!(#field_ident_str);
            field_ident_str
        };
        settings.short_names = short_names.into_vec_tokens(first, long_names.case);
    } else {
        // unused attribute
    }
    Ok(())
}

/// Generate the argument constructor for one named field.
fn derive_field(
    field: NamedField,
    settings: &FieldSettings<'_>,
) -> Result<(Ident, TokenStream), Error> {
    use Error::EmptyFieldName;
    let crate_ident = crate_ident();
    let NamedField { attributes: field_attrs, name: field_ident, ty: field_type, .. } = field;
    let field_ident_str = field_ident.to_string();
    if field_ident_str.is_empty() {
        return Err(EmptyFieldName(field_ident));
    }
    let mut field_settings = FieldAttributeSettings {
        help_literals: Vec::new(),
        long_names: quote!(#field_ident_str),
        short_names: quote!(None::<&str>),
        long_prefix: settings.long_prefix.clone(),
        short_prefix: settings.short_prefix.clone(),
        invert_prefix: settings.invert_prefix.clone(),
        invert_infix: settings.invert_infix.clone(),
        value_infix: settings.value_infix.clone(),
        meta_var: None,
        default_value: DefaultValue(
            quote!(<#field_type as ::#crate_ident::ParseArgument>::default()),
        ),
        argument: FieldArgument::EnumArgument,
    };
    for attr in field_attrs {
        parse_field_attribute(
            attr,
            &field_ident,
            &field_ident_str,
            &crate_ident,
            &mut field_settings,
        )?;
    }
    let field_long_prefix = field_settings.long_prefix.token_stream(settings.language);
    let field_short_prefix = field_settings.short_prefix.token_stream(settings.language);
    let field_invert_prefix = field_settings.invert_prefix.token_stream(settings.language);
    let field_invert_infix = field_settings.invert_infix.token_stream(settings.language);
    let field_value_infix = field_settings.value_infix.token_stream(settings.language);
    let field_meta_var = if let Some(MetaVar(string)) = field_settings.meta_var {
        quote!(#string)
    } else {
        settings.meta_var.clone()
    };
    let help_string = field_settings.help_literals.join(" ");
    let hilfe =
        if help_string.is_empty() { quote!(None::<&str>) } else { quote!(Some(#help_string)) };
    let long_names = field_settings.long_names;
    let short_names = field_settings.short_names;
    let default_value = field_settings.default_value;
    let create_description = quote!(
        let description = ::#crate_ident::Description::new(
            #field_long_prefix,
            #long_names,
            #field_short_prefix,
            #short_names,
            #hilfe,
            #default_value,
        );
    );
    let create_arguments = field_settings.argument.create_arguments(
        &create_description,
        &field_invert_prefix,
        &field_invert_infix,
        &field_value_infix,
        &field_meta_var,
        &field_type,
    );
    Ok((field_ident, create_arguments))
}

/// Parse the attributes and generate the argument constructor for every struct field.
pub(super) fn derive_fields(
    fields: Fields,
    settings: &FieldSettings<'_>,
) -> Result<Vec<(Ident, TokenStream)>, Error> {
    use Error::UnnamedFields;
    let iter: Box<dyn Iterator<Item = NamedField>> = match fields {
        Fields::Unit => Box::new(iter::empty()),
        Fields::Named(named_fields) => {
            Box::new(named_fields.fields.inner.into_iter().map(|(field, _punct)| field))
        },
        Fields::Tuple(tuple_fields) => return Err(UnnamedFields(tuple_fields.tk_parens.span)),
    };
    iter.map(|field| derive_field(field, settings)).collect()
}
