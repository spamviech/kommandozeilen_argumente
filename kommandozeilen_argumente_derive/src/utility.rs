//! Types and functions for parsing comma-separated arguments from a [`TokenStream`],
//! along with general utility functions and types.

use std::{
    fmt::{self, Display, Formatter},
    iter,
};

use proc_macro2::{Delimiter, Ident, Punct, Spacing, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use venial::{Attribute, AttributeValue, GroupSpan};

/// Identifier for the `kommandozeilen_argumente` crate name.
pub(crate) fn crate_ident() -> Ident {
    format_ident!("{}", "kommandozeilen_argumente")
}

/// The iterator did not contain exactly one element.
pub(crate) enum ExactlyOneError<T, I> {
    /// No element was provided.
    Empty,
    /// More than one element was provided.
    MoreThanOne {
        /// First element. The first [`Iterator::next`] call changes it to [`None`].
        first: Option<T>,
        /// Second element. The second [`Iterator::next`] call changes it to [`None`].
        second: Option<T>,
        /// All remaining elements.
        rest: I,
    },
}

impl<T, I: Iterator<Item = T>> Iterator for ExactlyOneError<T, I> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        use ExactlyOneError::{Empty, MoreThanOne};
        match self {
            Empty => None,
            MoreThanOne { first, second, rest } => {
                first.take().or_else(|| second.take()).or_else(|| rest.next())
            },
        }
    }
}

/// Require an [`Iterator`] to contain exactly one element.
pub(crate) fn exactly_one<T, I: Iterator<Item = T>>(
    mut iter: I,
) -> Result<T, ExactlyOneError<T, I>> {
    let first = iter.next().ok_or(ExactlyOneError::Empty)?;
    if let Some(second) = iter.next() {
        Err(ExactlyOneError::MoreThanOne { first: Some(first), second: Some(second), rest: iter })
    } else {
        Ok(first)
    }
}

/// Whether an argument is parsed case-sensitively.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum Case {
    /// Match case-sensitively.
    Sensitive,
    /// Match case-insensitively.
    #[default]
    Insensitive,
}

impl ToTokens for Case {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_ident = crate_ident();
        let ts = match self {
            Case::Sensitive => quote!(#crate_ident::unicode::Case::Sensitive),
            Case::Insensitive => quote!(#crate_ident::unicode::Case::Insensitive),
        };
        tokens.extend(ts);
    }
}

impl Case {
    /// Parse [`Case`] from a [`TokenStream`].
    pub(crate) fn parse(ts: &TokenStream) -> Option<Case> {
        match ts.to_string().as_str() {
            "sensitive" => Some(Case::Sensitive),
            "insensitive" => Some(Case::Insensitive),
            _ => None,
        }
    }
}

/// Whether a [`Punct`] is one [`char`] matching the requested character.
fn punct_is_char(punct: &Punct, char: char) -> bool {
    punct.as_char() == char && punct.spacing() == Spacing::Alone
}

/// Whether an [`Attribute`] path is a single [`Ident`] matching `value`.
pub(crate) fn path_is_ident(attr: &Attribute, wert: &str) -> bool {
    attr.get_single_path_segment().is_some_and(|ident| ident == wert)
}

/// Value of an argument.
#[derive(Debug, Clone)]
pub(crate) enum ArgumentValue {
    /// No value.
    /// wert
    NoValue,
    /// Sub-argument delimited by parentheses.
    /// wert(unterargument)
    SubArgument(Vec<Argument>),
    /// List argument.
    /// wert: [elem0, elem1]
    List(Vec<TokenStream>),
    /// General argument.
    /// wert: argument als stream
    Stream(TokenStream),
}

/// Write a list of elements delimited by `open` and `close`.
fn write_list<T: Display>(
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

/// Write an [`ArgumentValue`].
fn write_argument_value(
    formatter: &mut Formatter<'_>,
    colon: bool,
    value: &ArgumentValue,
) -> fmt::Result {
    use ArgumentValue::{List, NoValue, Stream, SubArgument};
    match value {
        NoValue => Ok(()),
        SubArgument(args) => write_list(formatter, "(", args, ")"),
        List(tts) => write_list(formatter, if colon { ": [" } else { "[" }, tts, "]"),
        Stream(ts) => write!(formatter, "{}{ts}", if colon { ": " } else { "" }),
    }
}

impl Display for ArgumentValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write_argument_value(formatter, false, self)
    }
}

/// Argument for a field.
#[derive(Debug, Clone)]
pub(crate) struct Argument {
    /// Argument name.
    pub(crate) name: String,
    /// Argument value.
    pub(crate) value: ArgumentValue,
}

impl Display for Argument {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let Argument { name, value } = self;
        formatter.write_str(name)?;
        write_argument_value(formatter, true, value)
    }
}

/// Error while splitting arguments.
#[derive(Debug)]
pub(crate) enum SplitArgumentsError {
    /// Arguments are not enclosed in parentheses.
    NotParenthesized {
        /// Argument path.
        parent: Vec<String>,
        /// Specified [`TokenStream`].
        ts: TokenStream,
    },
    /// No argument was specified.
    EmptyArgument {
        /// Argument path.
        parent: Vec<String>,
        /// Specified [`TokenStream`].
        ts: TokenStream,
    },
    /// Invalid argument name.
    InvalidArgumentName {
        /// Argument path.
        parent: Vec<String>,
        /// [`TokenTree`] where a name was expected.
        tt: TokenTree,
    },
    /// Invalid argument value.
    InvalidArgumentValue {
        /// Argument path.
        parent: Vec<String>,
        /// Argument name.
        name: String,
        /// Whether the value was separated with a colon.
        colon: bool,
        /// [`TokenStream`] for the value.
        value: TokenStream,
    },
}

impl Display for SplitArgumentsError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        /// Write the argument path.
        fn write_parent(
            formatter: &mut Formatter<'_>,
            parent: &[String],
            preposition: &str,
        ) -> fmt::Result {
            if !parent.is_empty() {
                write!(formatter, " {preposition} ")?;
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
        use SplitArgumentsError::{
            EmptyArgument, InvalidArgumentName, InvalidArgumentValue, NotParenthesized,
        };
        match self {
            NotParenthesized { parent, ts } => {
                write!(formatter, "Arguments")?;
                write_parent(formatter, parent, "for")?;
                write!(formatter, " are not enclosed in parentheses: {ts}")
            },
            EmptyArgument { parent, ts } => {
                write!(formatter, "Empty argument")?;
                write_parent(formatter, parent, "for")?;
                write!(formatter, ": {ts}")
            },
            InvalidArgumentName { parent, tt } => {
                write!(formatter, "Invalid argument name")?;
                write_parent(formatter, parent, "for")?;
                write!(formatter, ": {tt}")
            },
            InvalidArgumentValue { parent, name, colon, value } => {
                write!(formatter, "Invalid value for argument {name}")?;
                write_parent(formatter, parent, "for")?;
                write!(formatter, ":\n{name} ")?;
                if *colon {
                    write!(formatter, ": ")?;
                }
                write!(formatter, "{value}")
            },
        }
    }
}

/// Whether a [`TokenTree`] is not a comma character.
fn token_tree_is_not_comma(tt: &TokenTree) -> bool {
    if let TokenTree::Punct(punct) = tt { !punct_is_char(punct, ',') } else { true }
}

/// Arguments separated by commas; sub-arguments use `()` and may contain commas, e.g. `help`.
/// Arguments can have values separated by `:`.
/// Value arguments can be lists delimited by `[` and `]` and may contain commas.
/// Arguments are not processed further.
fn split_arguments(
    parent: Vec<String>,
    args: &mut Vec<Argument>,
    tokens: impl IntoIterator<Item = TokenTree>,
) -> Result<(), SplitArgumentsError> {
    use SplitArgumentsError::{EmptyArgument, InvalidArgumentName, InvalidArgumentValue};
    let mut iter = tokens.into_iter().peekable();
    let iter_mut_ref = iter.by_ref();
    while iter_mut_ref.peek().is_some() {
        let mut arg_iter = iter_mut_ref.take_while(token_tree_is_not_comma).peekable();
        // extract name
        let name = match arg_iter.next() {
            Some(TokenTree::Ident(ident)) => ident.to_string(),
            Some(tt) => return Err(InvalidArgumentName { parent, tt }),
            None => return Err(EmptyArgument { parent, ts: iter_mut_ref.collect() }),
        };
        // determine value
        let value = match arg_iter.next() {
            None => ArgumentValue::NoValue,
            Some(TokenTree::Punct(punct)) if punct_is_char(&punct, ':') => {
                match exactly_one(arg_iter) {
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
                        ArgumentValue::List(acc)
                    },
                    Ok(tt) => ArgumentValue::Stream(tt.into()),
                    Err(fehler) => ArgumentValue::Stream(fehler.collect()),
                }
            },
            Some(TokenTree::Group(group))
                if group.delimiter() == Delimiter::Parenthesis && arg_iter.peek().is_none() =>
            {
                let mut sub_parent = parent.clone();
                sub_parent.push(name.clone());
                let mut sub_args = Vec::new();
                split_arguments(sub_parent, &mut sub_args, group.stream())?;
                ArgumentValue::SubArgument(sub_args)
            },
            Some(first) => {
                return Err(InvalidArgumentValue {
                    parent,
                    name,
                    colon: false,
                    value: iter::once(first).chain(arg_iter).collect(),
                });
            },
        };
        // Argument hinzufügen
        args.push(Argument { name, value });
    }
    Ok(())
}

/// Split arguments enclosed in parentheses.
///
/// Arguments separated by commas; sub-arguments use `()` and may contain commas, e.g. `help`.
/// Arguments can have values separated by `:`.
/// Value arguments can be lists delimited by `[` and `]` and may contain commas.
/// Arguments are not processed further.
pub(crate) fn split_parenthesized_arguments(
    parent: Vec<String>,
    args: &mut Vec<Argument>,
    value: AttributeValue,
) -> Result<(), SplitArgumentsError> {
    use SplitArgumentsError::NotParenthesized;
    let AttributeValue::Group(GroupSpan { delimiter: Delimiter::Parenthesis, span: _ }, tokens) =
        value
    else {
        return Err(NotParenthesized { parent, ts: quote!(#value) });
    };
    split_arguments(parent, args, tokens)
}

#[cfg(test)]
mod test {
    use proc_macro2::{Group, Span};

    use super::*;

    #[test]
    fn test_split_arguments() {
        let mut args: Vec<Argument> = Vec::new();
        let span = Span::call_site();
        let value = AttributeValue::Group(
            GroupSpan { delimiter: Delimiter::Parenthesis, span },
            vec![
                TokenTree::Ident(Ident::new("hello", span)),
                TokenTree::Group(Group::new(
                    Delimiter::Parenthesis,
                    TokenStream::from(TokenTree::Ident(Ident::new("hi", span))),
                )),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Ident(Ident::new("world", span)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Group(Group::new(
                    Delimiter::Bracket,
                    [
                        TokenTree::Ident(Ident::new("it", span)),
                        TokenTree::Punct(Punct::new('\'', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("s", span)),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("a", span)),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("big", span)),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("world", span)),
                        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
                    ]
                    .into_iter()
                    .collect(),
                )),
            ],
        );
        let hello_str = "hello(hi)";
        let world_wert = "[it's, a, big, world!]".replace(' ', "");
        let world_string = format!("world:{world_wert}");
        split_parenthesized_arguments(Vec::new(), &mut args, value)
            .expect("Arguments are well-formed.");
        let args_str: Vec<_> = args.iter().map(|arg| arg.to_string().replace(' ', "")).collect();
        assert_eq!(args_str, vec![hello_str, &world_string]);
    }
}
