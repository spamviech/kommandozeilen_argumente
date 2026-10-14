//! Tests for automatically generated help.

#![expect(unused_crate_dependencies, reason = "Dependencies of the library.")]
#![expect(clippy::tests_outside_test_module, clippy::print_stdout, reason = "Integration test.")]

use std::{
    ffi::OsString,
    fmt::{self, Debug, Formatter},
    iter, result,
};

use kommandozeilen_argumente::{
    Arguments, Description, Language, Result,
    arguments::{flag::Flag, help::Default, single_argument::SingleArgument},
};

// Wrapper around String with a Debug=Display implementation.
struct DString(String);

impl Debug for DString {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[test]
fn help_test() -> result::Result<(), DString> {
    let language = Language::ENGLISH;
    let flag = Flag::new_with_language(
        Description::new_with_language("test", None::<&str>, Some("help"), Some(false), language),
        language,
    );
    let argument = Arguments::single_argument(SingleArgument::flag(flag));
    let arguments_with_help = argument.with_help_early_exit_with_language(
        &Default,
        "program",
        Some("My great program."),
        Some("0.test"),
        language,
    );

    match arguments_with_help.parse(iter::once(OsString::from("--help"))) {
        (Result::EarlyExit(messages), unused) if unused.is_empty() => {
            for message in messages {
                println!("{message}");
            }
            Ok(())
        },
        (_result, unused) if !unused.is_empty() => {
            Err(DString(format!("Unused arguments: {unused:?}")))
        },
        result => Err(DString(format!("Unexpected result: {result:?}"))),
    }
}
