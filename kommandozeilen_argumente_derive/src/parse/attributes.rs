//! Attribute parsing for `#[derive(Parse)]`.

use crate::{
    parse::{
        TokenStream, crate_ident, quote,
        types::{
            CreateError, CreateHelp, CreateVersion, DefaultValue, Deutsch, English, FieldArgument,
            InvertInfix, InvertPrefix, Language, LongNames, LongPrefix, MetaVar, ParseValueError,
            ProgramDescription, ProgramName, ProgramSettings, ProgramVersion, ShortNames,
            ShortNamesKind, ShortPrefix, ValueInfix, create_help_method, create_version_method,
            literal_or_to_string,
        },
    },
    utility::{Argument, ArgumentValue, Case},
};

/// Generated prefixes and names for a nested help or version attribute.
type SubArgumentNames = (LongPrefix, TokenStream, ShortPrefix, TokenStream);

/// Parse the common names and program settings of nested help or version attributes.
fn parse_named_sub_arguments(
    sub_args: Vec<Argument>,
    span: proc_macro2::Span,
    name: &str,
    program_settings: Option<&mut ProgramSettings<Option<ProgramDescription>>>,
) -> Result<(Option<Language>, SubArgumentNames), CreateError> {
    use ParseValueError::Unsupported;
    let crate_ident = crate_ident();
    let mut language = None;
    let mut long_prefix = LongPrefix::default();
    let mut long_names = LongNames::default();
    let mut short_prefix = ShortPrefix::default();
    let mut short_names = ShortNames::default();
    if let Err(create_error) = parse_value_argument(
        sub_args,
        Some(&mut language),
        None,
        None,
        None,
        program_settings,
        Some(&mut long_prefix),
        Some(&mut long_names),
        Some(&mut short_prefix),
        Some(&mut short_names),
        None,
        None,
        None,
        None,
        None,
        None,
    ) {
        let name = name.to_owned();
        return Err(Box::new(move |arg_name| match create_error(arg_name) {
            Unsupported { arg_name, argument } => Unsupported {
                arg_name,
                argument: Argument {
                    span,
                    name,
                    value: ArgumentValue::SubArgument(vec![argument]),
                },
            },
            error @ ParseValueError::NoLongName { .. } => error,
        }));
    }
    let (long_names, first, long_names_case) = match long_names.namen.as_ref() {
        Some((head, tail)) => (
            quote!(
                #crate_ident::NonEmpty {
                    head: #head,
                    tail: vec![#(#tail),*]
                }
            ),
            head.clone(),
            long_names.case,
        ),
        None => (quote!(#name), name.to_owned(), long_names.case),
    };
    let short_names = short_names.into_vec_tokens(&first, long_names_case);
    Ok((language, (long_prefix, long_names, short_prefix, short_names)))
}

/// Wrap an unsupported `case(...)` setting in its parent attribute.
fn case_error_argument(
    span: proc_macro2::Span,
    name: &str,
    sub_name: &str,
    value: &TokenStream,
) -> Argument {
    Argument {
        span,
        name: name.to_owned(),
        value: ArgumentValue::SubArgument(vec![Argument {
            span,
            name: sub_name.to_owned(),
            value: ArgumentValue::Stream(value.clone()),
        }]),
    }
}

/// Apply case-sensitivity settings from a nested `case(...)` attribute.
#[expect(
    clippy::too_many_arguments,
    reason = "Case settings can target each supported argument component."
)]
fn parse_case_sub_arguments(
    span: proc_macro2::Span,
    name: String,
    sub_args: Vec<Argument>,
    long_prefix: &mut Option<&mut LongPrefix>,
    long_names: &mut Option<&mut LongNames>,
    short_prefix: &mut Option<&mut ShortPrefix>,
    short_names: &mut Option<&mut ShortNames>,
    invert_prefix: &mut Option<&mut InvertPrefix>,
    invert_infix: &mut Option<&mut InvertInfix>,
    value_infix: &mut Option<&mut ValueInfix>,
) -> Result<(), CreateError> {
    use ParseValueError::Unsupported;
    macro_rules! set_case {
        ($target:expr, $case:expr, $argument:expr) => {
            if let Some(target) = $target.as_mut() {
                target.case = Some($case);
            } else {
                return Err(Box::new(move |arg_name| Unsupported {
                    arg_name,
                    argument: $argument,
                }));
            }
        };
    }
    for sub_arg in sub_args {
        let Argument { name: sub_name, value: ArgumentValue::Stream(ts), .. } = sub_arg else {
            return Err(Box::new(move |arg_name| Unsupported {
                arg_name,
                argument: Argument { span, name, value: ArgumentValue::SubArgument(vec![sub_arg]) },
            }));
        };
        let error_argument = case_error_argument(span, &name, &sub_name, &ts);
        let Some(case) = Case::parse(&ts) else {
            return Err(Box::new(move |arg_name| Unsupported {
                arg_name,
                argument: error_argument,
            }));
        };
        match sub_name.as_str() {
            "lang_präfix" | "long_prefix" => set_case!(long_prefix, case, error_argument),
            "lang" | "long" => set_case!(long_names, case, error_argument),
            "kurz_präfix" | "short_prefix" => set_case!(short_prefix, case, error_argument),
            "kurz" | "short" => set_case!(short_names, case, error_argument),
            "invertiere_präfix" | "invert_prefix" => {
                set_case!(invert_prefix, case, error_argument);
            },
            "invertiere_infix" | "invert_infix" => {
                set_case!(invert_infix, case, error_argument);
            },
            "wert_infix" | "value_infix" => set_case!(value_infix, case, error_argument),
            _ => {
                return Err(Box::new(move |arg_name| Unsupported {
                    arg_name,
                    argument: error_argument,
                }));
            },
        }
    }
    Ok(())
}

/// Configure a help early exit from a nested `help(...)` or `hilfe(...)` attribute.
fn parse_help_sub_arguments(
    span: proc_macro2::Span,
    name: &str,
    sub_args: Vec<Argument>,
    create_help: &mut CreateHelp,
) -> Result<(), CreateError> {
    let mut program_settings = ProgramSettings {
        name: ProgramName(None),
        version: ProgramVersion::Unspecified,
        description: Some(ProgramDescription(None)),
    };
    let (language, names) =
        parse_named_sub_arguments(sub_args, span, name, Some(&mut program_settings))?;
    let description = ProgramSettings {
        name: program_settings.name,
        version: program_settings.version,
        description: program_settings
            .description
            .expect("Some-Wert für Programm-Description wird bei rekursiven Aufruf nie None!"),
    };
    let default_language = if name == "hilfe" { Deutsch } else { English };
    *create_help = CreateHelp(Some(Box::new(create_help_method(
        &language.unwrap_or(default_language),
        Some(names),
        description,
    ))));
    Ok(())
}

/// Configure a version early exit from a nested `version(...)` attribute.
fn parse_version_sub_arguments(
    span: proc_macro2::Span,
    name: &str,
    sub_args: Vec<Argument>,
    create_version: &mut CreateVersion,
) -> Result<(), CreateError> {
    let mut program_settings = ProgramSettings {
        name: ProgramName(None),
        version: ProgramVersion::Unspecified,
        description: None,
    };
    let (language, names) =
        parse_named_sub_arguments(sub_args, span, name, Some(&mut program_settings))?;
    let settings = ProgramSettings {
        name: program_settings.name,
        version: program_settings.version,
        description: (),
    };
    *create_version =
        CreateVersion(Some(Box::new(create_version_method(language, Some(names), settings))));
    Ok(())
}

/// Configure default program settings from a nested `program(...)` or `programm(...)` attribute.
fn parse_program_sub_arguments(
    span: proc_macro2::Span,
    name: String,
    sub_args: Vec<Argument>,
    default_program_settings: &mut ProgramSettings<ProgramDescription>,
) -> Result<(), CreateError> {
    use ParseValueError::{NoLongName, Unsupported};
    let mut program_settings = ProgramSettings {
        name: ProgramName(None),
        version: ProgramVersion::Unspecified,
        description: Some(ProgramDescription(None)),
    };
    if let Err(create_error) = parse_value_argument(
        sub_args,
        None,
        None,
        None,
        None,
        Some(&mut program_settings),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ) {
        return Err(Box::new(move |arg_name| match create_error(arg_name) {
            Unsupported { arg_name, argument } => Unsupported {
                arg_name,
                argument: Argument {
                    span,
                    name,
                    value: ArgumentValue::SubArgument(vec![argument]),
                },
            },
            error @ NoLongName { .. } => error,
        }));
    }
    *default_program_settings = ProgramSettings {
        name: program_settings.name,
        version: program_settings.version,
        description: program_settings
            .description
            .expect("Some-Wert für Programm-Description wird bei rekursivem Aufruf nie None!"),
    };
    Ok(())
}

/// Parse a nested attribute group such as `help(...)`, `version(...)`, or `case(...)`.
#[expect(
    clippy::too_many_arguments,
    reason = "Nested attributes selectively update the supported parser settings."
)]
fn parse_sub_argument(
    span: proc_macro2::Span,
    name: String,
    sub_args: Vec<Argument>,
    default_program_settings: &mut Option<&mut ProgramSettings<ProgramDescription>>,
    create_help: &mut Option<&mut CreateHelp>,
    create_version: &mut Option<&mut CreateVersion>,
    long_prefix: &mut Option<&mut LongPrefix>,
    long_names: &mut Option<&mut LongNames>,
    short_prefix: &mut Option<&mut ShortPrefix>,
    short_names: &mut Option<&mut ShortNames>,
    invert_prefix: &mut Option<&mut InvertPrefix>,
    invert_infix: &mut Option<&mut InvertInfix>,
    value_infix: &mut Option<&mut ValueInfix>,
) -> Result<(), CreateError> {
    use ParseValueError::Unsupported;

    match (
        name.as_str(),
        create_help.as_mut(),
        create_version.as_mut(),
        default_program_settings.as_mut(),
    ) {
        ("hilfe" | "help", Some(create_help), _, _) => {
            parse_help_sub_arguments(span, &name, sub_args, create_help)?;
        },
        ("version", _, Some(create_version), _) => {
            parse_version_sub_arguments(span, &name, sub_args, create_version)?;
        },
        ("programm" | "program", _, _, Some(default_program_settings)) => {
            parse_program_sub_arguments(span, name, sub_args, default_program_settings)?;
        },
        ("case", _, _, _) => parse_case_sub_arguments(
            span,
            name,
            sub_args,
            long_prefix,
            long_names,
            short_prefix,
            short_names,
            invert_prefix,
            invert_infix,
            value_infix,
        )?,
        _ => {
            return Err(Box::new(move |arg_name| Unsupported {
                arg_name,
                argument: Argument { span, name, value: ArgumentValue::SubArgument(sub_args) },
            }));
        },
    }

    Ok(())
}

/// Set the argument value or return [`ParseValueError::Unsupported`].
macro_rules! set_argument {
    (< $([$mut_var: expr, $value: expr $(,)?]),+ $(,)?>, $sub_arg: expr $(,)?) => {
        $(
            if let Some(var) = $mut_var.as_mut() {
                **var = $value;
            } else
        )+
        {
            return Err(Box::new(move |arg_name| Unsupported {
                arg_name,
                argument: $sub_arg,
            }));
        }
    };
    ($mut_var: expr, $value: expr, $sub_arg: expr $(,)?) => {
        set_argument!(<[$mut_var, $value]>, $sub_arg)
    };
}

/// Helper macro for the argument-setting macros below.
macro_rules! set_argument_field {
    ($mut_var:expr, $feld:ident, $value:expr, $sub_arg:expr) => {
        if let Some(var) = $mut_var.as_mut() {
            var.$feld = $value;
        } else {
            return Err(Box::new(move |arg_name| Unsupported { arg_name, argument: $sub_arg }));
        }
    };
}

/// Set [`LongNames`] or [`ShortNames`] for an argument.
macro_rules! set_argument_names {
    ($mut_var:expr, $value:expr, $sub_arg:expr) => {
        set_argument_field!($mut_var, namen, $value, $sub_arg)
    };
}

/// Set a [`String`] value for an argument.
macro_rules! set_argument_string {
    ($mut_var:expr, $value:expr, $sub_arg:expr) => {
        set_argument_field!($mut_var, string, Some($value), $sub_arg)
    };
}

/// Set the [`Case`] value for an argument.
macro_rules! set_argument_case {
    ($mut_var:expr, $value:expr, $sub_arg:expr) => {
        set_argument_field!($mut_var, case, Some($value), $sub_arg)
    };
}

/// Parse the attributes for a value argument.
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "The attribute parser handles every supported argument setting in one pass."
)]
pub(super) fn parse_value_argument(
    args: Vec<Argument>,
    mut language: Option<&mut Option<Language>>,
    mut default_program_settings: Option<&mut ProgramSettings<ProgramDescription>>,
    mut create_help: Option<&mut CreateHelp>,
    mut create_version: Option<&mut CreateVersion>,
    mut program_settings: Option<&mut ProgramSettings<Option<ProgramDescription>>>,
    mut long_prefix: Option<&mut LongPrefix>,
    mut long_names: Option<&mut LongNames>,
    mut short_prefix: Option<&mut ShortPrefix>,
    mut short_names: Option<&mut ShortNames>,
    mut invert_prefix: Option<&mut InvertPrefix>,
    mut invert_infix: Option<&mut InvertInfix>,
    mut value_infix: Option<&mut ValueInfix>,
    mut meta_var: Option<&mut Option<MetaVar>>,
    mut default_value: Option<&mut DefaultValue>,
    mut field_argument: Option<&mut FieldArgument>,
) -> Result<(), CreateError> {
    use ParseValueError::{NoLongName, Unsupported};
    for Argument { span, name, value } in args {
        match value {
            ArgumentValue::NoValue => match name.as_str() {
                "name" => set_argument!(
                    program_settings
                        .as_mut()
                        .map(|program_settings| { &mut program_settings.name }),
                    ProgramName(None),
                    Argument { span, name, value }
                ),
                "version" => set_argument!(
                    <
                        [
                            program_settings.as_mut().map(
                                |program_settings| { &mut program_settings.version }
                            ),
                            ProgramVersion::CrateMacro,
                        ],
                        [
                            create_version,
                            CreateVersion(Some(Box::new(create_version_method(
                                None,
                                None,
                                ProgramSettings {
                                    name: ProgramName(None),
                                    version: ProgramVersion::Unspecified,
                                    description: ()
                                }
                            )))),
                        ],
                    >,
                    Argument { span, name, value }
                ),
                "hilfe" => set_argument!(
                    create_help,
                    CreateHelp(Some(Box::new(create_help_method(
                        &Deutsch,
                        None,
                        ProgramSettings {
                            name: ProgramName(None),
                            version: ProgramVersion::HelpWithoutSubArgument,
                            description: ProgramDescription(None)
                        }
                    )))),
                    Argument { span, name, value }
                ),
                "help" => set_argument!(
                    create_help,
                    CreateHelp(Some(Box::new(create_help_method(
                        &English,
                        None,
                        ProgramSettings {
                            name: ProgramName(None),
                            version: ProgramVersion::HelpWithoutSubArgument,
                            description: ProgramDescription(None)
                        }
                    )))),
                    Argument { span, name, value }
                ),
                "kurz" | "short" => {
                    set_argument_names!(
                        short_names,
                        ShortNamesKind::Auto,
                        Argument { span, name, value }
                    );
                },
                "glätten" | "flatten" => {
                    set_argument!(
                        field_argument,
                        FieldArgument::Parse,
                        Argument { span, name, value }
                    );
                },
                "FromStr" => {
                    set_argument!(
                        field_argument,
                        FieldArgument::FromStr,
                        Argument { span, name, value }
                    );
                },
                "benötigt" | "required" => {
                    set_argument!(
                        default_value,
                        DefaultValue(quote!(None)),
                        Argument { span, name, value }
                    );
                },
                _ => {
                    return Err(Box::new(move |arg_name| Unsupported {
                        arg_name,
                        argument: Argument { span, name, value },
                    }));
                },
            },
            ArgumentValue::List(liste) => match name.as_str() {
                "lang" | "long" => {
                    let mut namen_iter = liste.iter().map(literal_or_to_string);
                    let (head, tail) = if let Some(head) = namen_iter.next() {
                        (head, namen_iter.collect())
                    } else {
                        return Err(Box::new(move |arg_name| NoLongName { arg_name, name, span }));
                    };
                    set_argument_names!(
                        long_names,
                        Some((head, tail)),
                        Argument { span, name, value: ArgumentValue::List(liste) }
                    );
                },
                "kurz" | "short" => {
                    let namen_iter = liste.iter().map(literal_or_to_string);
                    set_argument_names!(
                        short_names,
                        ShortNamesKind::Names(namen_iter.collect()),
                        Argument { span, name, value: ArgumentValue::List(liste) }
                    );
                },
                _ => {
                    return Err(Box::new(move |arg_name| Unsupported {
                        arg_name,
                        argument: Argument { span, name, value: ArgumentValue::List(liste) },
                    }));
                },
            },
            ArgumentValue::Stream(ts) => match name.as_str() {
                "sprache" | "language" => {
                    set_argument!(
                        language,
                        Some(Language::parse(ts)),
                        Argument { span, name, value: ArgumentValue::Stream(ts) }
                    );
                },
                "name" => set_argument!(
                    program_settings
                        .as_mut()
                        .map(|program_settings| { &mut program_settings.name }),
                    ProgramName(Some(literal_or_to_string(&ts))),
                    Argument { span, name, value: ArgumentValue::Stream(ts) }
                ),
                "version" => set_argument!(
                    program_settings
                        .as_mut()
                        .map(|program_settings| { &mut program_settings.version }),
                    ProgramVersion::Specified(literal_or_to_string(&ts)),
                    Argument { span, name, value: ArgumentValue::Stream(ts) }
                ),
                "beschreibung" | "description" => set_argument!(
                    program_settings
                        .as_mut()
                        .and_then(|program_settings| { program_settings.description.as_mut() }),
                    ProgramDescription(Some(literal_or_to_string(&ts))),
                    Argument { span, name, value: ArgumentValue::Stream(ts) }
                ),
                "lang_präfix" | "long_prefix" => {
                    set_argument_string!(
                        long_prefix,
                        literal_or_to_string(&ts),
                        Argument { span, name, value: ArgumentValue::Stream(ts) }
                    );
                },
                "kurz_präfix" | "short_prefix" => {
                    set_argument_string!(
                        short_prefix,
                        literal_or_to_string(&ts),
                        Argument { span, name, value: ArgumentValue::Stream(ts) }
                    );
                },
                "invertiere_präfix" | "invert_prefix" => {
                    set_argument_string!(
                        invert_prefix,
                        literal_or_to_string(&ts),
                        Argument { span, name, value: ArgumentValue::Stream(ts) }
                    );
                },
                "invertiere_infix" | "invert_infix" => {
                    set_argument_string!(
                        invert_infix,
                        literal_or_to_string(&ts),
                        Argument { span, name, value: ArgumentValue::Stream(ts) }
                    );
                },
                "wert_infix" | "value_infix" => {
                    set_argument_string!(
                        value_infix,
                        literal_or_to_string(&ts),
                        Argument { span, name, value: ArgumentValue::Stream(ts) }
                    );
                },
                "meta_var" => {
                    set_argument!(
                        meta_var,
                        Some(MetaVar(literal_or_to_string(&ts))),
                        Argument { span, name, value: ArgumentValue::Stream(ts) }
                    );
                },
                "standard" | "default" => {
                    set_argument!(
                        default_value,
                        DefaultValue(quote!(Some(#ts))),
                        Argument { span, name, value: ArgumentValue::Stream(ts) }
                    );
                },
                "lang" | "long" => set_argument_names!(
                    long_names,
                    Some((literal_or_to_string(&ts), Vec::new())),
                    Argument { span, name, value: ArgumentValue::Stream(ts) }
                ),
                "kurz" | "short" => set_argument_names!(
                    short_names,
                    ShortNamesKind::Names(vec![literal_or_to_string(&ts)]),
                    Argument { span, name, value: ArgumentValue::Stream(ts) }
                ),
                "case" => {
                    let Some(case) = Case::parse(&ts) else {
                        return Err(Box::new(move |arg_name| Unsupported {
                            arg_name,
                            argument: Argument { span, name, value: ArgumentValue::Stream(ts) },
                        }));
                    };
                    let argument = Argument { span, name, value: ArgumentValue::Stream(ts) };
                    set_argument_case!(long_prefix, case, argument);
                    set_argument_case!(long_names, case, argument);
                    set_argument_case!(short_prefix, case, argument);
                    set_argument_case!(short_names, case, argument);
                    set_argument_case!(invert_prefix, case, argument);
                    set_argument_case!(invert_infix, case, argument);
                    set_argument_case!(value_infix, case, argument);
                },
                _ => {
                    return Err(Box::new(move |arg_name| Unsupported {
                        arg_name,
                        argument: Argument { span, name, value: ArgumentValue::Stream(ts) },
                    }));
                },
            },
            ArgumentValue::SubArgument(sub_args) => parse_sub_argument(
                span,
                name,
                sub_args,
                &mut default_program_settings,
                &mut create_help,
                &mut create_version,
                &mut long_prefix,
                &mut long_names,
                &mut short_prefix,
                &mut short_names,
                &mut invert_prefix,
                &mut invert_infix,
                &mut value_infix,
            )?,
        }
    }
    Ok(())
}
