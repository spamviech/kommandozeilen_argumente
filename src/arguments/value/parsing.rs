//! Recognition of value argument forms.

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ffi::OsString,
};

use crate::{
    arguments::parsing::{
        ParsedLongForms, ParsedMergedShortForms, ParsedRawValue, ParsedShortForms, ParsedValueName,
    },
    description::{ArgumentInput, Name},
    unicode::Compare,
};

/// Parses non-merged short value forms from the supplied inputs.
pub(super) fn parse_short_value_forms<'argument>(
    name: &Name<'argument>,
    value_infix: &Compare<'argument>,
    args: impl Iterator<Item = ArgumentInput>,
) -> ParsedShortForms<'argument> {
    let mut values = HashMap::new();
    let mut missing_values: HashSet<ParsedValueName<'argument>> = HashSet::default();
    let mut args = args.peekable();
    let mut remaining = Vec::new();
    while let Some(argument) = args.next() {
        match argument {
            ArgumentInput::Unchanged(argument) => {
                let input = Cow::Owned(argument.to_string_lossy().into_owned());
                if let Some((name, inline_value)) =
                    name.parse_short_with_value(value_infix, &argument)
                {
                    let parsed_name = ParsedValueName { name: Cow::Owned(name.into()) };
                    if let Some(value) = inline_value
                        .map(|normalized| OsString::from(normalized.as_str()))
                        .or_else(|| {
                            args.next_if(|arg| matches!(arg, ArgumentInput::Unchanged(_))).and_then(
                                |arg| match arg {
                                    ArgumentInput::Unchanged(value) => Some(value),
                                    ArgumentInput::AdjustedMergedShortNames(_) => None,
                                },
                            )
                        })
                    {
                        let _ = values.insert(parsed_name, ParsedRawValue { value, input });
                    } else {
                        let _ = missing_values.insert(parsed_name);
                    }
                } else {
                    remaining.push(ArgumentInput::Unchanged(argument));
                }
            },
            adjusted @ ArgumentInput::AdjustedMergedShortNames(_) => remaining.push(adjusted),
        }
    }
    ParsedShortForms {
        early_exits: Vec::new(),
        flags: Vec::new(),
        missing_values,
        values,
        remaining,
        alternative_branches: Vec::new(),
    }
}

/// Parses long value forms from the supplied inputs.
pub(super) fn parse_long_value_forms<'argument>(
    name: &Name<'argument>,
    value_infix: &Compare<'argument>,
    args: impl Iterator<Item = ArgumentInput>,
) -> ParsedLongForms<'argument> {
    let mut values = HashMap::new();
    let mut missing_values: HashSet<ParsedValueName<'argument>> = HashSet::default();
    let mut args = args.peekable();
    let mut remaining = Vec::new();
    while let Some(argument) = args.next() {
        match argument {
            ArgumentInput::Unchanged(argument) => {
                let input = Cow::Owned(argument.to_string_lossy().into_owned());
                if let Some((name, inline_value)) =
                    name.parse_long_with_value(value_infix, &argument)
                {
                    let parsed_name = ParsedValueName { name: Cow::Owned(name.into()) };
                    if let Some(value) = inline_value
                        .map(|normalized| OsString::from(normalized.as_str()))
                        .or_else(|| {
                            args.next_if(|arg| matches!(arg, ArgumentInput::Unchanged(_))).and_then(
                                |arg| match arg {
                                    ArgumentInput::Unchanged(value) => Some(value),
                                    ArgumentInput::AdjustedMergedShortNames(_) => None,
                                },
                            )
                        })
                    {
                        let _ = values.insert(parsed_name, ParsedRawValue { value, input });
                    } else {
                        let _ = missing_values.insert(parsed_name);
                    }
                } else {
                    remaining.push(ArgumentInput::Unchanged(argument));
                }
            },
            adjusted @ ArgumentInput::AdjustedMergedShortNames(_) => remaining.push(adjusted),
        }
    }
    ParsedLongForms {
        early_exits: Vec::new(),
        flags: Vec::new(),
        missing_values,
        values,
        remaining,
        alternative_branches: Vec::new(),
    }
}

/// Parses merged short value forms from the supplied inputs.
pub(super) fn parse_value_merged_short_forms<'argument>(
    name: &Name<'argument>,
    value_infix: &Compare<'argument>,
    args: impl Iterator<Item = ArgumentInput>,
) -> ParsedMergedShortForms<'argument> {
    let mut values = HashMap::new();
    let mut missing_values: HashSet<ParsedValueName<'argument>> = HashSet::default();
    let mut args = args.peekable();
    let mut remaining = Vec::new();
    while let Some(argument) = args.next() {
        let input = match &argument {
            ArgumentInput::Unchanged(argument) => {
                Cow::Owned(argument.to_string_lossy().into_owned())
            },
            ArgumentInput::AdjustedMergedShortNames(_) => Cow::Borrowed(""),
        };
        if let Some(parsed) = name.parse_with_value_merge_short_forms(value_infix, &argument) {
            let value = parsed.inline_value.map_or_else(
                || {
                    args.next_if(|arg| matches!(arg, ArgumentInput::Unchanged(_))).and_then(|arg| {
                        match arg {
                            ArgumentInput::Unchanged(value) => Some(value),
                            ArgumentInput::AdjustedMergedShortNames(_) => None,
                        }
                    })
                },
                |value| Some(OsString::from(value.into_string())),
            );
            let parsed_name = ParsedValueName { name: Cow::Owned(parsed.name.into()) };
            if let Some(value) = value {
                let _ = values.insert(parsed_name, ParsedRawValue { value, input });
            } else {
                let _ = missing_values.insert(parsed_name);
            }
            if let Some(remaining_input) = parsed.remaining {
                remaining.push(remaining_input);
            }
        } else {
            remaining.push(argument);
        }
    }
    ParsedMergedShortForms {
        early_exits: Vec::new(),
        flags: Vec::new(),
        missing_values,
        values,
        remaining,
        alternative_branches: Vec::new(),
    }
}
