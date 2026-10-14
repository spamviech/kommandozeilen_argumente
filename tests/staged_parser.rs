//! End-to-end coverage for the staged parser.

#![expect(unused_crate_dependencies, reason = "dependencies of the lib")]
#![expect(clippy::tests_outside_test_module, reason = "Integration test.")]

use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    result,
};

use kommandozeilen_argumente::{
    ArgumentInput, Arguments, Description, EarlyExit, Parse, ParseArgument, ParseError, Result,
    Value, arguments::single_argument::SingleArgument, dyn_to_owned::Parse as ParseCallback,
    nonempty,
};

const EMPTY_REMAINING: [ArgumentInput; 0] = [];

#[test]
fn parses_a_single_flag() {
    let arguments =
        <bool as ParseArgument>::new(Description::new("--", "verbose", "-", "v", None, None));

    let (result, remaining) = arguments.parse([OsString::from("--verbose")].into_iter());

    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Value(true)));
}

#[test]
fn parses_a_single_value() {
    let arguments =
        <u16 as ParseArgument>::new(Description::new("--", "port", "-", "p", None, None));

    let (result, remaining) = arguments.parse([OsString::from("--port=8080")].into_iter());

    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Value(8080)));
}

#[derive(Debug, PartialEq, Eq, Parse)]
struct Configuration {
    verbose: bool,
    port: u16,
}

#[test]
fn derives_and_accumulates_a_combined_struct() {
    let (result, remaining) = Configuration::arguments().parse(
        [OsString::from("--verbose"), OsString::from("--port"), OsString::from("8080")].into_iter(),
    );

    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Value(Configuration { verbose: true, port: 8080 })));
}

fn reject_value(_: &OsStr) -> result::Result<String, ParseError<String>> {
    Err(ParseError::ParseError(String::from("rejected by the first alternative")))
}

fn accept_value(value: &OsStr) -> result::Result<String, ParseError<String>> {
    value.to_str().map(String::from).ok_or_else(|| ParseError::InvalidString(value.to_owned()))
}

#[expect(clippy::ptr_arg, reason = "Required interface for Show.")]
fn string_display(value: &String) -> String {
    value.clone()
}

fn alternative_value(
    parse: fn(&OsStr) -> result::Result<String, ParseError<String>>,
) -> Value<'static, String, String> {
    let parse: Box<dyn ParseCallback<'static, String, ParseError<String>>> = Box::new(parse);
    Value {
        description: Description::new("--", "choice", "-", "c", None, None),
        value_infix: "=".into(),
        meta_var: "CHOICE",
        possible_values: None,
        parse: Cow::Owned(parse),
        display: Cow::Borrowed(&string_display),
        display_error: Cow::Borrowed(&string_display),
    }
}

#[test]
fn tries_the_next_alternative_after_a_value_conversion_failure() {
    let arguments = Arguments::alternatives(nonempty![
        Arguments::from(alternative_value(reject_value)),
        Arguments::from(alternative_value(accept_value)),
    ]);

    let (result, remaining) = arguments.parse([OsString::from("--choice=accepted")].into_iter());

    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Value(value) if value == "accepted"));
}

#[test]
fn converts_errors_for_single_arguments() {
    let arguments = Arguments::from(alternative_value(reject_value))
        .convert_error(|error| error.len(), |error| error.to_string());

    let (result, remaining) = arguments.parse([OsString::from("--choice=rejected")].into_iter());

    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Error(errors) if matches!(
        &errors.head,
        kommandozeilen_argumente::Error::ParseError(error)
            if matches!(error.error, ParseError::ParseError(33))
    )));
}

#[test]
fn early_exit_short_circuits_result_accumulation() {
    let early_exit =
        EarlyExit::new(Description::new("--", "help", "-", "h", None, None), "show help");
    let arguments = Arguments::single_argument(SingleArgument::early_exit(early_exit));

    let (result, remaining) = arguments.parse([OsString::from("--help")].into_iter());

    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::EarlyExit(messages) if messages.head == "show help"));
}
