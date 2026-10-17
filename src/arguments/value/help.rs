//! Help-text and string-rendering support for value arguments.

use std::{borrow::Cow, ffi::OsStr};

use nonempty::NonEmpty;

use crate::{
    arguments::{help::Help, value::Value},
    description::{Description, Name},
    dyn_to_owned::{Parse, Show},
    outcome::ParseError,
    unicode::Compare,
};

/// Appends comma-separated displayed values to `string`.
#[expect(
    clippy::ptr_arg,
    reason = "Lifetimes don't work out (easily) using as_ref and &dyn Show<'_,T>"
)]
pub(super) fn show_elements<'t, T: 't>(
    string: &mut String,
    display: &Cow<'_, dyn Show<'_, T>>,
    values: impl IntoIterator<Item = &'t T>,
) {
    for (index, value) in values.into_iter().enumerate() {
        if index != 0 {
            string.push_str(", ");
        }
        string.push_str(&display(value));
    }
}
/// Converts a typed value argument into its string-rendered representation.
#[expect(
    clippy::ptr_arg,
    clippy::too_many_arguments,
    reason = "Internal function, we are using Cow everywhere. Takes all configuration options \
              individually."
)]
pub(super) fn as_string_value<'a, 't, T, E, PE: Into<ParseError<E>>>(
    name: &'a Name<'t>,
    help: Option<&'t str>,
    default: Option<&'a T>,
    value_infix: Compare<'t>,
    meta_var: &'t str,
    possible_values: Option<&'a NonEmpty<T>>,
    parse: &'a Cow<'t, dyn Parse<'t, T, PE>>,
    display: &'a Cow<'t, dyn Show<'t, T>>,
    display_error: &'a Cow<'t, dyn Show<'t, E>>,
) -> Value<'a, String, String>
where
    't: 'a,
{
    let parse_boxed: Box<dyn 'a + Parse<'a, String, ParseError<String>>> =
        Box::new(|value: &OsStr| {
            parse(value)
                .map(|value| display(&value))
                .map_err(|error| error.into().convert(|error| display_error(&error)))
        });
    Value {
        description: Description { name: name.clone(), help, default: default.map(&**display) },
        value_infix,
        meta_var,
        possible_values: NonEmpty::from_vec(
            possible_values.into_iter().flatten().map(&**display).collect(),
        ),
        parse: Cow::Owned(parse_boxed),
        display: Cow::Borrowed(&Clone::clone),
        display_error: Cow::Borrowed(&Clone::clone),
    }
}
/// Creates syntax and help text from the shared parts of a German or English value argument.
#[expect(
    clippy::ptr_arg,
    clippy::too_many_arguments,
    reason = "Internal method, we are using Cow everywhere."
)]
pub(super) fn create_value_help_text<T>(
    name: &Name<'_>,
    help: Option<&str>,
    default: Option<&T>,
    value_infix: &str,
    meta_var: &str,
    possible_values: Option<&NonEmpty<T>>,
    display: &Cow<'_, dyn Show<'_, T>>,
    meta_default: &str,
    meta_allowed_values: &str,
) -> Help {
    let Name { long_prefix, long, short_prefix, short } = name;
    let mut syntax = long_prefix.as_str().to_owned();
    Name::alternatives_as_regex(&long.head, &long.tail, &mut syntax);
    syntax.push_str("( |");
    syntax.push_str(value_infix);
    syntax.push(')');
    syntax.push_str(meta_var);
    if let Some((head, tail)) = short.split_first() {
        syntax.push_str(" | ");
        syntax.push_str(short_prefix.as_str());
        Name::alternatives_as_regex(head, tail, &mut syntax);
        syntax.push_str("[ |");
        syntax.push_str(value_infix);
        syntax.push(']');
        syntax.push_str(meta_var);
    }
    let suffix = match (default, possible_values) {
        (None, None) => None,
        (None, Some(values)) => {
            let mut text = format!("[{meta_allowed_values}: ");
            show_elements(&mut text, display, values);
            text.push(']');
            Some(text)
        },
        (Some(value), None) => Some(format!("[{meta_default}: {}]", display(value))),
        (Some(value), Some(values)) => {
            let mut text = format!("[{meta_default}: {}, {meta_allowed_values}: ", display(value));
            show_elements(&mut text, display, values);
            text.push(']');
            Some(text)
        },
    };
    let help = match (help, suffix) {
        (None, suffix) => suffix,
        (Some(help), None) => Some(help.to_owned()),
        (Some(help), Some(suffix)) => Some(format!("{help} {suffix}")),
    };
    Help { syntax, help }
}
