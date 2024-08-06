//! An example using the "derive"-feature with english names.

// dependencies of the lib
#![allow(unused_crate_dependencies)]
// checking if the derive macro triggers any lint
#![allow(clippy::blanket_clippy_restriction_lints)]
#![warn(clippy::restriction)]
// Only works when disabled for the whole module (can't be don in the macro).
// Required for the use of some [`Language`]-fields.
#![allow(clippy::disallowed_script_idents)]

use core::{
    fmt::{self, Debug, Display},
    num::NonZeroI32,
};

use kommandozeilen_argumente::{EnumArgument, Parse};

/// An example enum, to show the use of [`EnumArgument`].
#[derive(Debug, Clone, EnumArgument)]
#[kommandozeilen_argumente(case: insensitive)]
enum Enumeration {
    /// one
    One,
    /// two
    Two,
    /// three
    Three,
}

impl Display for Enumeration {
    #[allow(clippy::renamed_function_params)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[allow(clippy::implicit_return)]
        Debug::fmt(self, formatter)
    }
}

/// struct to define the command line arguments.
#[derive(Debug, Parse)]
#[kommandozeilen_argumente(help(description: "program description.", short))]
#[kommandozeilen_argumente(version, language: english, program(version))]
struct Args {
    /// A flag with default settings.
    flag: bool,
    /// A flag with alternative names.
    #[kommandozeilen_argumente(long: [other, names], short: u)]
    renamed: bool,
    /// A flag without default value, with alternative prefix to invert the flag.
    #[kommandozeilen_argumente(required, short, invert_prefix: kein)]
    required: bool,
    /// A String value.
    value: String,
    /// An Enumeration-value with default value and alternative meta variable.
    #[kommandozeilen_argumente(kurz, standard: Enumeration::Two, meta_var: VAR)]
    enumeration: Enumeration,
}

impl Display for Args {
    #[allow(
        clippy::pattern_type_mismatch,
        clippy::question_mark_used,
        clippy::implicit_return,
        clippy::renamed_function_params
    )]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Args { flag, renamed, required, value, enumeration } = self;
        writeln!(formatter, "flag: {flag}")?;
        writeln!(formatter, "renamed: {renamed}")?;
        writeln!(formatter, "required: {required}")?;
        writeln!(formatter, "value: {value}")?;
        writeln!(formatter, "enumeration: {enumeration}")
    }
}

fn main() {
    #[allow(clippy::expect_used, clippy::separated_literal_suffix)]
    let args = Args::parse_with_error_message_from_env(NonZeroI32::new(1_i32).expect("1 != 0"));
    #[allow(clippy::print_stdout, clippy::use_debug)]
    {
        println!("{args:?}");
    }
}
