//! [`ParseArgument`] support for [`Option<T>`].

use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
};

use nonempty::NonEmpty;

use super::argument::ParseArgument;
use crate::{
    arguments::{
        Arguments,
        combine::Combine,
        flag::Flag,
        help::{self, CreateHelpText},
        parsing::{ParsedLongForms, ParsedMergedShortForms, ParsedShortForms, RecognizedArguments},
        single_argument::SingleArgument,
        value::Value,
    },
    description::{ArgumentInput, Description},
    dyn_to_owned::{self, Bool, Show},
    outcome::{Error, ParseError, Result},
    unicode::Compare,
};

/// Helper type for the [`ParseArgument`] implementation of [`Option<T>`].
struct OptionArguments<'t, F, T, E> {
    /// Adjusts the result of parsing an [`Option<T>`].
    adjust_result: F,
    /// Argument definition used to parse an [`Option<T>`].
    arguments: Arguments<'t, T, E>,
}

impl<'t, T: Debug, E: Debug, F: FnOnce(Result<'t, T, E>) -> Result<'t, T, E>> Combine<'t, T, E>
    for OptionArguments<'t, F, T, E>
{
    #[inline]
    fn create_help_text(
        &self,
        variant: &dyn CreateHelpText,
        meta_default: &str,
        meta_allowed_values: &str,
    ) -> NonEmpty<help::Alternatives> {
        self.arguments.create_help_text(variant, meta_default, meta_allowed_values)
    }

    #[inline]
    fn parse_merged_short_forms(
        &self,
        args: Box<dyn Iterator<Item = OsString> + '_>,
    ) -> NonEmpty<ParsedMergedShortForms<'t>> {
        self.arguments.parse_merged_short_forms(args)
    }

    #[inline]
    fn parse_merged_short_inputs(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedMergedShortForms<'t>> {
        self.arguments.parse_merged_short_inputs(args)
    }

    #[inline]
    fn parse_short_forms(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedShortForms<'t>> {
        self.arguments.parse_short_forms(args)
    }

    #[inline]
    fn parse_long_forms(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedLongForms<'t>> {
        self.arguments.parse_long_forms(args)
    }

    #[inline]
    fn validate(&self, recognized: RecognizedArguments<'t>) -> Result<'t, (), E> {
        self.arguments.validate(recognized)
    }

    #[inline]
    fn accumulate(self: Box<Self>, recognized: RecognizedArguments<'t>) -> Result<'t, T, E> {
        let Self { adjust_result, arguments } = *self;
        adjust_result(arguments.accumulate(recognized))
    }

    #[inline]
    fn debug_fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OptionArguments")
            .field("adjust_result", &"<closure>")
            .field("arguments", &self.arguments)
            .finish()
    }
}

/// Creates the [`Show`] closure for [`Option<T>`] as a trait object.
fn create_boxed_option_display<'t, T>(
    display: Cow<'t, dyn Show<'t, T>>,
) -> Box<dyn 't + Show<'t, Option<T>>> {
    Box::new(move |opt: &Option<T>| {
        #[expect(clippy::min_ident_chars, reason = "t: T")]
        if let Some(t) = opt { display(t) } else { String::from("None") }
    })
}

/// Creates the closure that adjusts the final [`Result`] for an [`Option<T>`].
fn create_result_adjuster<'t, T: Clone>(
    default: Option<T>,
) -> impl FnOnce(Result<'t, T, String>) -> Result<'t, T, String> {
    |result| match (result, default) {
        (Result::Error(errors), Some(default)) => {
            let mut final_result = Some(Result::Value(default.clone()));
            for error in &errors {
                if let Error::ParseError(_parse_error) = error {
                    final_result = None;
                    break;
                }
            }
            if let Some(final_result) = final_result { final_result } else { Result::Error(errors) }
        },
        (result, _) => result,
    }
}

/// Creates the [`Description`] used to invoke [`ParseArgument::arguments`] for an [`Option<T>`].
fn create_description<'t, T>(description: &Description<'t, Option<T>>) -> Description<'t, T> {
    let name_long_prefix = description.name.long_prefix.clone();
    let long = description.name.long.clone();
    let name_short_prefix = description.name.short_prefix.clone();
    let short = description.name.short.clone();
    Description::new(
        name_long_prefix,
        long.clone(),
        name_short_prefix,
        short.clone(),
        None::<&str>,
        None,
    )
}

impl<T: ParseArgument + Clone + Debug + Display> ParseArgument for Option<T> {
    #[inline]
    fn arguments<'t>(
        description: Description<'t, Self>,
        invert_prefix: impl Into<Compare<'t>>,
        invert_infix: impl Into<Compare<'t>>,
        value_infix: impl Into<Compare<'t>>,
        meta_var: &'t str,
    ) -> Arguments<'t, Self, String> {
        let value_infix_compare = value_infix.into();
        let arguments = <T as ParseArgument>::arguments(
            create_description(&description),
            invert_prefix,
            invert_infix,
            value_infix_compare,
            meta_var,
        );
        let adjust_result = create_result_adjuster(description.default.clone());
        #[expect(
            clippy::shadow_unrelated,
            reason = "Macro-generated bindings intentionally reuse user-facing argument names."
        )]
        match arguments {
            Arguments::Single(SingleArgument::Flag(Flag {
                description: _,
                invert_prefix,
                invert_infix,
                convert,
                display,
            })) => {
                let boxed_convert: Box<dyn Bool<'t, Option<T>>> =
                    Box::new(move |value| Some(convert(value)));
                Arguments::Single(SingleArgument::Flag(Flag {
                    description,
                    invert_prefix,
                    invert_infix,
                    convert: Cow::Owned(boxed_convert),
                    display: Cow::Owned(create_boxed_option_display(display)),
                }))
            },
            Arguments::Single(SingleArgument::EarlyExit { early_exit, value, display }) => {
                // `default` is guaranteed to be [`None`], so `description` can be moved.
                Arguments::Single(SingleArgument::EarlyExit {
                    early_exit,
                    value: Some(value),
                    display: Cow::Owned(create_boxed_option_display(display)),
                })
            },
            Arguments::Single(SingleArgument::Value(Value {
                description: _,
                value_infix,
                meta_var,
                possible_values,
                parse,
                display,
                display_error,
            })) => {
                let boxed_parse: Box<dyn dyn_to_owned::Parse<'t, Option<T>, ParseError<String>>> =
                    Box::new(move |os_str: &OsStr| match parse(os_str) {
                        Ok(value) => Ok(Some(value)),
                        Err(_error) if os_str == "None" => Ok(None),
                        Err(error) => Err(error),
                    });
                let possible_values = possible_values.map(|nonempty| {
                    let mut nonempty = nonempty.map(Some);
                    nonempty.push(None);
                    nonempty
                });
                let value = Arguments::from(Value {
                    description,
                    value_infix,
                    meta_var,
                    possible_values,
                    parse: Cow::Owned(boxed_parse),
                    display: Cow::Owned(create_boxed_option_display(display)),
                    display_error,
                });
                Arguments::combine(OptionArguments { adjust_result, arguments: value })
            },
            Arguments::Combined(combine) => Arguments::combine(OptionArguments {
                adjust_result,
                arguments: Arguments::combine((Some, Arguments::Combined(combine))),
            }),
            Arguments::Alternatives(alternatives) => Arguments::combine(OptionArguments {
                adjust_result,
                arguments: Arguments::combine((Some, Arguments::Alternatives(alternatives))),
            }),
        }
    }

    #[inline]
    fn default() -> Option<Self> {
        Some(None)
    }
}
