//! An example using the function-api with english names.

#![expect(unused_crate_dependencies, reason = "dependencies of the lib")]

use std::{
    borrow::Cow,
    convert::identity,
    ffi::{OsStr, OsString},
    fmt::{self, Debug, Display},
    num::NonZeroI32,
};

use kommandozeilen_argumente::{
    Compare, Description, EnumArgument, Language, NonEmpty, ParseArgument, ParseError,
    arguments::{flag::Flag, help::Default, value::Value},
    combine, crate_name, crate_version,
};
use nonempty::nonempty;

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

    fn parse_enum(arg: &OsStr) -> Result<Self, ParseError<String>> {
        use Enumeration::{One, Three, Two};
        if let Some(string) = arg.to_str() {
            // Target strings only contain ASCII-characters.
            // Therefore, all others can be ignored.
            let lowercase = string.to_ascii_lowercase();
            match lowercase.as_str() {
                "one" => Ok(One),
                "two" => Ok(Two),
                "three" => Ok(Three),
                _ => Err(ParseError::ParseError(format!("Unknown variant: {string}"))),
            }
        } else {
            Err(ParseError::InvalidString(OsString::from(arg)))
        }
    }
}

impl Display for Enumeration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

/// An inner alternative selected by its required argument.
#[derive(Debug)]
enum Inner {
    /// Select the flag alternative.
    Flag {
        /// Whether the inner flag is enabled.
        inner_flag: bool,
    },
    /// Select the number alternative.
    Number {
        /// The inner number.
        inner_number: i8,
    },
}

impl Display for Inner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Flag { inner_flag } => write!(formatter, "Flag {{ inner_flag: {inner_flag} }}"),
            Self::Number { inner_number } => {
                write!(formatter, "Number {{ inner_number: {inner_number} }}")
            },
        }
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
    /// A required inner alternative.
    inner: Inner,
}

impl Display for Args {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Args { flag, renamed, required, value, enumeration, inner } = self;
        writeln!(formatter, "flag: {flag}")?;
        writeln!(formatter, "renamed: {renamed}")?;
        writeln!(formatter, "required: {required}")?;
        writeln!(formatter, "value: {value}")?;
        writeln!(formatter, "enumeration: {enumeration}")?;
        writeln!(formatter, "inner: {inner}")
    }
}

#[expect(clippy::too_many_lines, reason = "The example constructs every argument explicitly.")]
fn main() {
    let language = Language::ENGLISH;
    let flag = Flag::new_with_language(
        Description::new_with_language(
            "flag",
            None::<&str>,
            Some("A flag with default settings."),
            Some(false),
            language,
        ),
        language,
    );
    let renamed = Flag::new_with_language(
        Description::new_with_language(
            NonEmpty { head: "other", tail: vec!["names"] },
            "u",
            Some("A flag with alternative names."),
            Some(false),
            language,
        ),
        language,
    );
    let required = Flag {
        description: Description::new_with_language(
            "required",
            "r",
            Some("A flag without default value, with alternative prefix to invert the flag."),
            None,
            language,
        ),
        invert_prefix: Compare::from("kein"),
        invert_infix: Compare::from(language.invert_infix),
        convert: Cow::Borrowed(&identity),
        display: Cow::Borrowed(&ToString::to_string),
    };
    let value = String::arguments_with_language(
        Description::new_with_language(
            "value",
            None::<&str>,
            Some("A String value."),
            None,
            language,
        ),
        language,
    );
    let inner_flag = Flag::new_with_language(
        Description::new_with_language(
            "inner-flag",
            None::<&str>,
            Some("Select the flag alternative."),
            None,
            language,
        ),
        language,
    );
    let inner_number = i8::arguments_with_language(
        Description::new_with_language(
            "inner-number",
            None::<&str>,
            Some("Select the number alternative."),
            None,
            language,
        ),
        language,
    );
    #[expect(
        clippy::shadow_unrelated,
        reason = "The combine callbacks bind parsed values with their argument names."
    )]
    let inner = kommandozeilen_argumente::Arguments::alternatives(nonempty![
        combine!(|inner_flag| Inner::Flag { inner_flag }, inner_flag),
        combine!(|inner_number| Inner::Number { inner_number }, inner_number),
    ]);
    let enumeration = Value::new_enum_with_language(
        Description::new_with_language(
            "enumeration",
            "e",
            Some("An Enumeration-value with default value and alternative meta variable."),
            Some(Enumeration::Two),
            language,
        ),
        language,
    );
    #[expect(
        clippy::shadow_unrelated,
        reason = "lambda arguments, linked by calling with arguments."
    )]
    let merge = |flag, renamed, required, value, enumeration, inner| Args {
        flag,
        renamed,
        required,
        value,
        enumeration,
        inner,
    };
    let argumente = combine!(merge, flag, renamed, required, value, enumeration, inner)
        .with_help_and_version_early_exit_with_language(
            &Default,
            crate_name!(),
            Some("Program-Description."),
            crate_version!(),
            language,
        );
    let args = argumente
        .parse_complete_with_language_from_env(NonZeroI32::new(1).expect("1 != 0"), language);
    #[expect(clippy::print_stdout, reason = "example implementation.")]
    {
        println!("{args}");
    }
}
