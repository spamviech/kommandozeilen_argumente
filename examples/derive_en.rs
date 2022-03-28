use std::{
    fmt::{Debug, Display},
    num::NonZeroI32,
};

use kommandozeilen_argumente::{EnumArgument, Parse};

#[derive(Debug, Clone, EnumArgument)]
#[kommandozeilen_argumente(case: insensitive)]
enum Enumeration {
    One,
    Two,
    Three,
}

impl Display for Enumeration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug, Parse)]
#[kommandozeilen_argumente(help(description: "program description.", short))]
#[kommandozeilen_argumente(version, language: english)]
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Args { flag, renamed, required, value, enumeration } = self;
        write!(f, "flag: {flag}\n")?;
        write!(f, "renamed: {renamed}\n")?;
        write!(f, "required: {required}\n")?;
        write!(f, "value: {value}\n")?;
        write!(f, "enumeration: {enumeration}\n")
    }
}

fn main() {
    let args = Args::parse_with_error_message_from_env(NonZeroI32::new(1).expect("1 != 0"));
    println!("{:?}", args)
}
