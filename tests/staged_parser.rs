//! End-to-end coverage for the staged parser.

#![expect(unused_crate_dependencies, reason = "dependencies of the lib")]
#![expect(clippy::tests_outside_test_module, reason = "Integration test.")]

use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    result,
    string::ToString,
};

use kommandozeilen_argumente::{
    ArgumentInput, Arguments, Description, EarlyExit, Parse, ParseArgument, ParseError, Result,
    Value, arguments::single_argument::SingleArgument, combine,
    dyn_to_owned::Parse as ParseCallback, nonempty,
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
        .convert_error(|error| error.len(), ToString::to_string);

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

#[test]
fn parses_canonically_equivalent_unicode_names_in_all_public_forms() {
    let flag = <bool as ParseArgument>::new(Description::new("--", "f\u{f6}o", "-", "f", None, None));
    let value = <u16 as ParseArgument>::new(Description::new("--", "p\u{f6}rt", "-", "p", None, None));
    let arguments = combine!(|flag, value| (flag, value), flag, value);
    let (result, remaining) = arguments.parse(
        [OsString::from("--fo\u{308}o"), OsString::from("-p8080")].into_iter(),
    );
    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Value((true, 8080))));

    let early_exit = EarlyExit::new(
        Description::new("--", "h\u{e9}lp", "-", "h", None, None),
        "unicode help",
    );
    let (result, remaining) = Arguments::single_argument(SingleArgument::early_exit(early_exit))
        .parse([OsString::from("--he\u{301}lp")].into_iter());
    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::EarlyExit(messages) if messages.head == "unicode help"));
}

#[test]
fn merged_short_names_consume_whole_multiscalar_graphemes() {
    let accent = <bool as ParseArgument>::new(Description::new("--", "accent", "-", "e\u{301}", None, None));
    let x = <bool as ParseArgument>::new(Description::new("--", "x", "-", "x", None, None));
    let (result, remaining) = combine!(|accent, x| (accent, x), accent, x)
        .parse([OsString::from("-e\u{301}x")].into_iter());
    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Value((true, true))));
}

#[test]
fn multigrapheme_short_names_are_not_merged() {
    let argument = <bool as ParseArgument>::new(Description::new("--", "alpha", "-", "ab", None, None));
    let (result, remaining) = argument.parse([OsString::from("-ab")].into_iter());
    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Value(true)));

    let argument = <bool as ParseArgument>::new(Description::new("--", "alpha", "-", "ab", None, None));
    let (result, remaining) = argument.parse([OsString::from("-a")].into_iter());
    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Error(_)));
}

#[test]
fn adjusted_merged_remainders_continue_merged_short_recognition() {
    let flag = <bool as ParseArgument>::new(Description::new("--", "flag", "-", "f", None, None));
    let value = <u16 as ParseArgument>::new(Description::new("--", "value", "-", "x", None, None));
    let (result, remaining) = combine!(|flag, value| (flag, value), flag, value)
        .parse([OsString::from("-fx"), OsString::from("42")].into_iter());
    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Value((true, 42))));
}

#[test]
fn values_report_missing_use_defaults_and_keep_the_last_occurrence() {
    let required = <u16 as ParseArgument>::new(Description::new("--", "port", "-", "p", None, None));
    let (result, _) = required.parse(std::iter::empty());
    assert!(matches!(result, Result::Error(errors) if matches!(errors.head, kommandozeilen_argumente::Error::MissingValue { .. })));

    let defaulted = <u16 as ParseArgument>::new(Description::new("--", "port", "-", "p", None, Some(80)));
    let (result, remaining) = defaulted.parse(std::iter::empty());
    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Value(80)));

    let repeated = <u16 as ParseArgument>::new(Description::new("--", "port", "-", "p", None, None));
    let (result, remaining) = repeated.parse(
        [OsString::from("--port=80"), OsString::from("--port=8080")].into_iter(),
    );
    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Value(8080)));
}

#[cfg(unix)]
#[test]
fn non_utf8_raw_values_reach_the_typed_value_parser() {
    use std::os::unix::ffi::OsStringExt as _;

    let argument = <String as ParseArgument>::new(Description::new("--", "text", "-", "t", None, None));
    let (result, remaining) = argument.parse(
        [OsString::from("--text"), OsString::from_vec(vec![0xff])].into_iter(),
    );
    assert_eq!(remaining, EMPTY_REMAINING);
    assert!(matches!(result, Result::Error(errors) if matches!(
        &errors.head,
        kommandozeilen_argumente::Error::ParseError(error)
            if matches!(error.error, ParseError::InvalidString(_))
    )));
}
