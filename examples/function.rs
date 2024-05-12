//! An example using the function-api with english names.

// dependencies of the lib
#![allow(unused_crate_dependencies)]

use std::{
    borrow::Cow,
    convert::identity,
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
    num::NonZeroI32,
};

use nonempty::nonempty;

use kommandozeilen_argumente::{
    argumente::{flag::Flag, hilfe::Default, wert::Wert},
    combine, crate_name, crate_version, Arguments, Compare, Description, EnumArgument, Language,
    NonEmpty, ParseArgument, ParseError,
};

/// An example enum, to show the use of [`EnumArgument`].
#[derive(Debug, Clone)]
enum Enumeration {
    /// one
    One,
    /// two
    Two,
    /// three
    Three,
}

impl EnumArgument for Enumeration {
    fn varianten() -> Option<NonEmpty<Self>> {
        use Enumeration::{One, Three, Two};
        Some(nonempty![One, Two, Three])
    }

    fn parse_enum(arg: &OsStr) -> Result<Self, kommandozeilen_argumente::ParseFehler<String>> {
        use Enumeration::{One, Three, Two};
        if let Some(string) = arg.to_str() {
            // Target strings only contain ASCII-characters.
            // Therefore, all others can be ignored.
            let lowercase = string.to_ascii_lowercase();
            match lowercase.as_str() {
                "one" => Ok(One),
                "two" => Ok(Two),
                "three" => Ok(Three),
                _ => Err(ParseError::ParseFehler(format!("Unknown variant: {string}"))),
            }
        } else {
            Err(ParseError::InvaliderString(OsString::from(arg)))
        }
    }
}

impl Display for Enumeration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

/// struct representing the parsed command line arguments.
#[derive(Debug)]
struct Args {
    /// A flag with default settings.
    flag: bool,
    /// A flag with alternative names.
    renamed: bool,
    /// A flag without default value, with alternative prefix to invert the flag.
    required: bool,
    /// A String value.
    value: String,
    /// An Enumeration-value with default value and alternative meta variable.
    enumeration: Enumeration,
}

impl Display for Args {
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
    let language = Language::ENGLISH;
    let flag = Arguments::from(Flag::new_with_language(
        Description::new_with_language(
            "flag",
            None::<&str>,
            Some("A flag with default settings."),
            Some(false),
            language,
        ),
        language,
    ));
    let renamed = Arguments::from(Flag::new_with_language(
        Description::new_with_language(
            NonEmpty { head: "other", tail: vec!["names"] },
            "u",
            Some("A flag with alternative names."),
            Some(false),
            language,
        ),
        language,
    ));
    let required = Arguments::from(Flag {
        beschreibung: Description::new_with_language(
            "required",
            "r",
            Some("A flag without default value, with alternative prefix to invert the flag."),
            None,
            language,
        ),
        invertiere_präfix: Compare::from("kein"),
        invertiere_infix: Compare::from(language.invertiere_infix),
        konvertiere: Cow::Borrowed(&identity),
        anzeige: Cow::Borrowed(&ToString::to_string),
    });
    let value = Arguments::from(String::arguments_with_language(
        Description::new_with_language(
            "value",
            None::<&str>,
            Some("A String value."),
            None,
            language,
        ),
        language,
    ));
    let enumeration = Arguments::from(Wert::new_enum_with_language(
        Description::new_with_language(
            "enumeration",
            "e",
            Some("An Enumeration-value with default value and alternative meta variable."),
            Some(Enumeration::Two),
            language,
        ),
        language,
    ));
    #[allow(clippy::shadow_unrelated)]
    let merge = |flag, renamed, required, value, enumeration| Args {
        flag,
        renamed,
        required,
        value,
        enumeration,
    };
    let argumente = combine!(merge, flag, renamed, required, value, enumeration)
        .with_help_and_version_early_exit_with_language(
            &Default,
            crate_name!(),
            Some("Programm-Description."),
            crate_version!(),
            language,
        );
    let args = argumente
        .parse_vollständig_mit_sprache_aus_env(NonZeroI32::new(1).expect("1 != 0"), language);
    #[allow(clippy::print_stdout, clippy::use_debug)]
    {
        println!("{args:?}");
    }
}
