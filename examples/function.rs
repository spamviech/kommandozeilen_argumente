use std::{
    ffi::OsString,
    fmt::{Debug, Display},
    num::NonZeroI32,
};

use kommandozeilen_argumente::{
    combine, crate_name, crate_version, Arguments, Description, EnumArgument, Language, NonEmpty,
    ParseArgument, ParseError,
};

#[derive(Debug, Clone)]
enum Enumeration {
    One,
    Two,
    Three,
}

impl EnumArgument for Enumeration {
    fn varianten() -> Vec<Self> {
        use Enumeration::*;
        vec![One, Two, Three]
    }

    fn parse_enum(arg: OsString) -> Result<Self, kommandozeilen_argumente::ParseFehler<String>> {
        use Enumeration::*;
        if let Some(string) = arg.to_str() {
            // Target strings only contain ASCII-characters.
            // Therefore, all others can be ignored.
            let lowercase = string.to_ascii_lowercase();
            match lowercase.as_str() {
                "one" => Ok(One),
                "two" => Ok(Two),
                "three" => Ok(Three),
                _ => Err(ParseError::ParseFehler(format!("Unknown variant: {}", string))),
            }
        } else {
            Err(ParseError::InvaliderString(arg))
        }
    }
}

impl Display for Enumeration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug)]
struct Args {
    flag: bool,
    renamed: bool,
    required: bool,
    value: String,
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
    let language = Language::ENGLISH;
    let flag = Arguments::flag_bool_with_language(
        Description::new_with_language(
            "flag",
            None::<&str>,
            Some("A flag with default settings."),
            Some(false),
            language,
        ),
        language,
    );
    let renamed = Arguments::flag_bool_with_language(
        Description::new_with_language(
            NonEmpty { head: "other", tail: vec!["names"] },
            "u",
            Some("A flag with alternative names."),
            Some(false),
            language,
        ),
        language,
    );
    let required = Arguments::flag_bool(
        Description::new_with_language(
            "required",
            "r",
            Some("A flag without default value, with alternative prefix to invert the flag."),
            None,
            language,
        ),
        "kein",
        language.invertiere_infix,
    );
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
    let enumeration = Arguments::value_enum_display(
        Description::new_with_language(
            "enumeration",
            "e",
            Some("An Enumeration-value with default value and alternative meta variable."),
            Some(Enumeration::Two),
            language,
        ),
        language.wert_infix,
        "VAR",
    );
    let merge = |flag, renamed, required, value, enumeration| Args {
        flag,
        renamed,
        required,
        value,
        enumeration,
    };
    let argumente = combine!(merge, flag, renamed, required, value, enumeration)
        .hilfe_und_version_mit_sprache(
            crate_name!(),
            Some("Programm-Description."),
            crate_version!(),
            language,
        );
    let args = argumente
        .parse_vollständig_mit_sprache_aus_env(NonZeroI32::new(1).expect("1 != 0"), language);
    println!("{:?}", args)
}
