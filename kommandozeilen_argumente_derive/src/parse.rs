//! Implementation of the derive macro for the `Parse` trait.

use std::{
    fmt::{self, Display, Formatter},
    iter,
};

use litrs::StringLit;
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use unicode_segmentation::UnicodeSegmentation;
use venial::{Fields, Item, NamedField, Struct, TypeExpr, parse_item};

use crate::{
    CompileError,
    utility::{
        Argument, ArgumentValue, Case, SplitArgumentsError, crate_ident, exactly_one,
        path_is_ident, split_parenthesized_arguments,
    },
};

/// Language for an argument structure. Influences default values for other settings.
#[derive(Debug, Clone)]
enum Language {
    /// German
    Deutsch,
    /// English
    English,
    /// Unknown language
    TokenStream(TokenStream),
}
use Language::{Deutsch, English};

impl Language {
    /// Parse a language from a [`TokenStream`].
    fn parse(ts: TokenStream) -> Language {
        match exactly_one(ts.into_iter()) {
            Ok(TokenTree::Ident(ident)) => match ident.to_string().as_str() {
                "deutsch" | "german" => Deutsch,
                "englisch" | "english" => English,
                _ => Language::TokenStream(TokenTree::Ident(ident).into()),
            },
            Ok(tt) => Language::TokenStream(tt.into()),
            Err(fehler) => Language::TokenStream(fehler.collect()),
        }
    }

    /// Create a [`TokenStream`] for the current language.
    fn token_stream(&self) -> TokenStream {
        use Language::{Deutsch, English, TokenStream};
        let crate_ident = crate_ident();
        match self {
            Deutsch => quote!(::#crate_ident::Language::GERMAN),
            English => quote!(::#crate_ident::Language::ENGLISH),
            TokenStream(ts) => ts.clone(),
        }
    }
}

/// Strategy for parsing an argument value.
enum FieldArgument {
    /// `EnumArgument`
    EnumArgument,
    /// `FromStr`
    FromStr,
    /// Parse
    Parse,
}

impl FieldArgument {
    /// Create the [`TokenStream`] that constructs the arguments for this [`FieldArgument`].
    fn create_arguments(
        self,
        create_description: &TokenStream,
        field_invert_prefix: &TokenStream,
        field_invert_infix: &TokenStream,
        field_value_infix: &TokenStream,
        field_meta_var: &TokenStream,
        field_type: &TypeExpr,
    ) -> TokenStream {
        let crate_ident = crate_ident();
        match self {
            FieldArgument::EnumArgument => {
                quote!({
                    #create_description
                    <#field_type as ::#crate_ident::ParseArgument>::arguments(
                        description,
                        #field_invert_prefix,
                        #field_invert_infix,
                        #field_value_infix,
                        #field_meta_var
                    )
                })
            },
            FieldArgument::FromStr => {
                quote!({
                    #create_description
                    ::#crate_ident::Value {
                        description,
                        value_infix: ::#crate_ident::Compare::from(#field_value_infix),
                        meta_var: #field_meta_var,
                        possible_values: None,
                        parse: ::std::borrow::Cow::Borrowed(&|os_str: &::std::ffi::OsStr| {
                            if let Some(string) = os_str.to_str() {
                                string.parse::<#field_type>().map_err(
                                    |error| ::#crate_ident::ParseError::ParseError(error.to_string())
                                )
                            } else {
                                Err(::#crate_ident::ParseError::InvalidString(::std::ffi::OsString::from(os_str)))
                            }
                        }),
                        display: ::std::borrow::Cow::Borrowed(&ToString::to_string),
                        display_error: ::std::borrow::Cow::Borrowed(&ToString::to_string),
                    }
                })
            },
            FieldArgument::Parse => {
                quote!(::#crate_ident::Parse::arguments())
            },
        }
    }
}

/// Create a function that adds a `--version` flag to an `item`.
///
/// ## Deutsch
/// Erstelle eine Funktion, die einem `item` eine `--version`-Flag hinzufügt.
fn create_version_method(
    fixed_language: Option<Language>,
    names: Option<(LongPrefix, TokenStream, ShortPrefix, TokenStream)>,
    program_settings: ProgramSettings<()>,
) -> impl FnOnce(TokenStream, Language, &ProgramName, &ProgramVersion) -> TokenStream {
    let crate_ident = crate_ident();
    move |item, default_language, default_name, default_version| {
        let language = fixed_language.unwrap_or(default_language);
        let language_tokens = language.token_stream();
        let default_long = quote!(#language_tokens.version_long);
        let default_short = quote!(#language_tokens.version_short);
        let (long_prefix, long_names, short_prefix, short_names) = names.unwrap_or_else(|| {
            (LongPrefix::default(), default_long, ShortPrefix::default(), default_short)
        });
        let long_prefix = long_prefix.token_stream(&language);
        let short_prefix = short_prefix.token_stream(&language);
        let description = quote!(
            ::#crate_ident::Description::new(
                #long_prefix,
                #long_names,
                #short_prefix,
                #short_names,
                Some(#language_tokens.version_description),
                None,
            )
        );
        let ProgramSettings { name: program_name, version: program_version, description: () } =
            program_settings;
        let program_name = program_name.or(default_name);
        let program_version_display = ProgramVersionDisplay {
            program_version: program_version.or_default(default_version),
            is_option: false,
        };
        quote!(
            #item.with_version_early_exit(
                #description,
                #program_name,
                #program_version_display,
            )
        )
    }
}

/// Create a function that adds a `--help` flag to an `item`.
///
/// ## Deutsch
/// Erstelle eine Funktion, die einem `item` eine `--hilfe`-Flag hinzufügt.
fn create_help_method<'t>(
    language: &Language,
    names: Option<(LongPrefix, TokenStream, ShortPrefix, TokenStream)>,
    program_settings: ProgramSettings<ProgramDescription>,
) -> impl 't + FnOnce(TokenStream, &ProgramName, &ProgramVersion, &ProgramDescription) -> TokenStream
{
    let crate_ident = crate_ident();
    let language_tokens = language.token_stream();
    let default_long = quote!(#language_tokens.help_long);
    let default_short = quote!(#language_tokens.help_short);
    let (long_prefix, long_names, short_prefix, short_names) = names.unwrap_or_else(|| {
        (LongPrefix::default(), default_long, ShortPrefix::default(), default_short)
    });
    let long_prefix = long_prefix.token_stream(language);
    let short_prefix = short_prefix.token_stream(language);
    let description = quote!(
        ::#crate_ident::Description::new(
            #long_prefix,
            #long_names,
            #short_prefix,
            #short_names,
            Some(#language_tokens.help_description),
            None,
        )
    );
    let ProgramSettings {
        name: program_name,
        version: program_version,
        description: program_description,
    } = program_settings;
    move |item, default_name, default_version, default_description| {
        let program_name = program_name.or(default_name);
        let program_version_display = ProgramVersionDisplay {
            program_version: program_version.or_default(default_version),
            is_option: true,
        };
        let program_description = program_description.or(default_description);
        quote!(
            #item.with_help_early_exit(
                &::#crate_ident::arguments::help::Standard,
                #description,
                #program_name,
                #program_description,
                #program_version_display,
                #language_tokens.default,
                #language_tokens.allowed_values,
                #language_tokens.options,
                #language_tokens.syntax_prefix,
                #language_tokens.syntax_padding,
                #language_tokens.alternative_prefix,
                #language_tokens.alternative_separator,
            )
        )
    }
}

/// Error while parsing a value attribute.
#[derive(Debug)]
pub(crate) enum ParseValueError {
    /// The attribute is unsupported.
    Unsupported {
        /// Name of the field on which the argument was specified.
        arg_name: Option<String>,
        /// The unknown argument.
        argument: Argument,
    },
    /// No long name was specified.
    NoLongName {
        /// Name of the field on which the argument was specified.
        arg_name: Option<String>,
        /// String parsed as a long name.
        name: String,
        /// Location of the empty long-name list.
        span: proc_macro2::Span,
    },
}

impl Display for ParseValueError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentValue::{List, NoValue, Stream, SubArgument};
        use ParseValueError::{NoLongName, Unsupported};
        match self {
            Unsupported { arg_name, argument: Argument { name, value: NoValue, .. } } => {
                write!(formatter, "Unsupported argument")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {name}")
            },
            Unsupported {
                arg_name,
                argument: Argument { name, value: wert @ SubArgument(_), .. },
            } => {
                write!(formatter, "Unsupported sub-argument of {name}")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {wert}")
            },
            Unsupported { arg_name, argument: Argument { name, value: wert @ List(_), .. } } => {
                write!(formatter, "Unsupported list argument {name}")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {wert}")
            },
            Unsupported { arg_name, argument: Argument { name, value: wert @ Stream(_), .. } } => {
                write!(formatter, "Unsupported named argument {name}")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {wert}")
            },
            NoLongName { arg_name, name, .. } => {
                write!(formatter, "No long name")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, " specified in explicit list containing {name}")
            },
        }
    }
}

/// Function that creates a [`ParseValueError`] from an argument name.
type CreateError = Box<dyn FnOnce(Option<String>) -> ParseValueError>;

/// Description of the program.
#[derive(Debug, Clone)]
struct ProgramSettings<Description> {
    /// Program name specified through the derive macro.
    name: ProgramName,
    /// Program version specified through the derive macro.
    version: ProgramVersion,
    /// Program description specified through the derive macro.
    description: Description,
}

/// Program name in help/version text.
/// Falls back to the `CARGO_PKG_NAME` environment variable when specifying [`None`].
#[derive(Debug, Clone)]
struct ProgramName(Option<String>);

impl ToTokens for ProgramName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(string) = &self.0 {
            tokens.extend(quote!(#string));
        } else {
            let crate_ident = crate_ident();
            tokens.extend(quote!(::#crate_ident::crate_name!()));
        }
    }
}

impl ProgramName {
    /// Returns self if it contains [`Some`], otherwise returns the `alternative`.
    fn or(mut self, alternative: &Self) -> Self {
        self.0 = self.0.or(alternative.0.clone());
        self
    }
}

/// Program version in help/version text.
#[derive(Debug, Clone)]
enum ProgramVersion {
    /// An explicit string was specified.
    Specified(String),
    /// The value from the `CARGO_PKG_VERSION` variable was requested.
    CrateMacro,
    /// Version for `hilfe`/`help` without a sub-argument.
    /// Use the value from the `CARGO_PKG_VERSION` variable.
    HelpWithoutSubArgument,
    /// No value was specified.
    Unspecified,
}

impl ProgramVersion {
    /// Returns self if it contains [`ProgramVersion::Specified`], or
    /// [`ProgramVersion::CrateMacro`], otherwise returns the `alternative`.
    fn or_default(self, default: &Self) -> Self {
        match self {
            ProgramVersion::Specified(_) | ProgramVersion::CrateMacro => self,
            ProgramVersion::HelpWithoutSubArgument | ProgramVersion::Unspecified => default.clone(),
        }
    }
}

/// Helper that provides alternative [`ToTokens`] implementations for [`ProgramVersion`].
struct ProgramVersionDisplay {
    /// The specified version.
    program_version: ProgramVersion,
    /// Whether the value is specified in an option.
    /// In an option, [`ProgramVersion::Unspecified`] produces [`None`],
    /// otherwise it produces the value from the `CARGO_PKG_VERSION` variable.
    is_option: bool,
}

impl ToTokens for ProgramVersionDisplay {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_ident = crate_ident();
        let add_some_if_option = |token_stream: TokenStream| {
            if self.is_option { quote!(Some(#token_stream)) } else { token_stream }
        };
        match &self.program_version {
            ProgramVersion::Specified(string) => {
                tokens.extend(add_some_if_option(quote!(#string)));
            },
            ProgramVersion::CrateMacro | ProgramVersion::HelpWithoutSubArgument => {
                tokens.extend(add_some_if_option(quote!(::#crate_ident::crate_version!())));
            },
            ProgramVersion::Unspecified => {
                if self.is_option {
                    tokens.extend(quote!(None));
                } else {
                    tokens.extend(add_some_if_option(quote!(::#crate_ident::crate_version!())));
                }
            },
        }
    }
}

/// Program description in help text.
#[derive(Debug, Clone)]
struct ProgramDescription(Option<String>);

impl ToTokens for ProgramDescription {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(string) = &self.0 {
            tokens.extend(quote!(Some(#string)));
        } else {
            tokens.extend(quote!(None));
        }
    }
}

impl ProgramDescription {
    /// Returns self if it contains [`Some`], otherwise returns the `alternative`.
    fn or(mut self, alternative: &Self) -> Self {
        self.0 = self.0.or(alternative.0.clone());
        self
    }
}

/// Function that creates a `--hilfe` flag.
struct CreateHelp(
    #[expect(
        clippy::type_complexity,
        reason = "The callback signature captures the generated help configuration."
    )]
    Option<
        Box<
            dyn FnOnce(
                TokenStream,
                &ProgramName,
                &ProgramVersion,
                &ProgramDescription,
            ) -> TokenStream,
        >,
    >,
);

/// Function that creates a `--version` flag.
struct CreateVersion(
    #[expect(
        clippy::type_complexity,
        reason = "The callback signature captures the generated version configuration."
    )]
    Option<Box<dyn FnOnce(TokenStream, Language, &ProgramName, &ProgramVersion) -> TokenStream>>,
);

/// Create a newtype with an identical [`ToTokens`] implementation.
macro_rules! create_newtype {
    ($($name: ident : $type: ty),* $(,)?) => {
        $(
            #[derive(Debug, Clone)]
            struct $name($type);

            impl ToTokens for $name {
                fn to_tokens(&self, tokens: &mut TokenStream) {
                    self.0.to_tokens(tokens)
                }
            }
        )*
    };
}

create_newtype! {
    MetaVar: String,
    DefaultValue: TokenStream,
}

/// Create newtypes for string-like values wrapped in a [`Compare`] in the generated
/// [`TokenStream`].
macro_rules! compare_types {
    ($($name: ident ($sprache_ident: ident)),* $(,)?) => {
        $(
            #[derive(Debug, Clone)]
            struct $name { string: Option<String>, case: Option<Case> }

            impl Default for $name {
                fn default() -> Self {
                    $name { string: None, case: None }
                }
            }

            impl $name {
                fn token_stream(&self, language: &Language) -> TokenStream {
                    let crate_ident = crate_ident();
                    let string = if let Some(string) = &self.string {
                        quote!(#string)
                    } else {
                        let language_tokens = language.token_stream();
                        quote!(#language_tokens.$sprache_ident)
                    };
                    let case = self.case.unwrap_or_default();
                    quote!(::#crate_ident::unicode::Compare {
                        string: ::#crate_ident::unicode::Normalized::new(#string),
                        case: #case,
                    })
                }
            }
        )*
    };
}

compare_types! {
    LongPrefix(long_prefix),
    ShortPrefix(short_prefix),
    InvertPrefix(invert_prefix),
    InvertInfix(invert_infix),
    ValueInfix(value_infix),
}

/// Long names for an argument.
#[derive(Debug, Default)]
struct LongNames {
    /// The specified long names.
    namen: Option<(String, Vec<String>)>,
    /// Whether matching is case-sensitive.
    case: Option<Case>,
}

/// Specified short names without [`Case`].
#[derive(Debug)]
enum ShortNamesKind {
    /// No short names.
    None,
    /// Automatically derived short name.
    Auto,
    /// Explicitly specified short names.
    Names(Vec<String>),
}

/// Short names for an argument.
#[derive(Debug)]
struct ShortNames {
    /// Specified short names without [`Case`].
    namen: ShortNamesKind,
    /// Whether matching is case-sensitive.
    case: Option<Case>,
}

impl Default for ShortNames {
    fn default() -> Self {
        ShortNames { namen: ShortNamesKind::None, case: None }
    }
}

impl ShortNames {
    /// Convert to a potentially empty [`Vec`] of fixed [`String`]s.
    fn into_vec(
        self,
        lang_name: &str,
        long_names_case: Option<Case>,
    ) -> (Vec<String>, Option<Case>) {
        match self.namen {
            ShortNamesKind::None => (Vec::new(), self.case),
            ShortNamesKind::Auto => (
                vec![
                    lang_name.graphemes(true).next().expect("Langname ohne Graphemes!").to_owned(),
                ],
                self.case.or(long_names_case),
            ),
            ShortNamesKind::Names(namen) => (namen, self.case),
        }
    }

    /// Create a [`TokenStream`] with a [`vec!`] macro for all short names.
    fn into_vec_tokens(self, lang_name: &str, long_names_case: Option<Case>) -> TokenStream {
        let (vec, case) = self.into_vec(lang_name, long_names_case);
        if vec.is_empty() {
            quote!(None::<&str>)
        } else {
            let crate_ident = crate_ident();
            if let Some(case) = case {
                quote!(vec![#(#crate_ident::unicode::Compare {
                    string: ::#crate_ident::unicode::Normalized::new(#vec),
                    case: #case,
                }),*])
            } else {
                quote!(vec![#(#vec),*])
            }
        }
    }
}

/// Return a string literal directly, or otherwise convert the [`TokenStream`] with
/// [`ToString::to_string`].
fn literal_or_to_string(token_stream: &TokenStream) -> String {
    if let Some(string_lit) = exactly_one(token_stream.clone().into_iter())
        .ok()
        .and_then(|literal| StringLit::try_from(literal).ok())
    {
        string_lit.into_value()
    } else {
        token_stream.to_string()
    }
}

/// Parse the attributes for a value argument.
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "The attribute parser handles every supported argument setting in one pass."
)]
fn parse_value_argument(
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
    let crate_ident = crate_ident();
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
            ArgumentValue::SubArgument(sub_args) => {
                /// Parse a sub-argument recursively.
                macro_rules! recursive {
                    ($program_settings: expr, $sub_language: ident, $prefix_and_names: ident $(,)?) => {
                        let mut $sub_language = None;
                        let mut sub_long_prefix = LongPrefix::default();
                        let mut sub_long = LongNames::default();
                        let mut sub_short_prefix = ShortPrefix::default();
                        let mut sub_short = ShortNames::default();
                        let result = parse_value_argument(
                            sub_args,
                            Some(&mut $sub_language),
                            None,
                            None,
                            None,
                            $program_settings,
                            Some(&mut sub_long_prefix),
                            Some(&mut sub_long),
                            Some(&mut sub_short_prefix),
                            Some(&mut sub_short),
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                        );
                        if let Err(create_error) = result {
                            return Err(Box::new(move |arg_name| match create_error(arg_name) {
                                Unsupported { arg_name, argument } => Unsupported {
                                    arg_name,
                                    argument: Argument {
                                        span,
                                        name,
                                        value: ArgumentValue::SubArgument(vec![argument])
                                    }
                                },
                                error => error,
                            }))
                        };
                        let (sub_lang_ts, erster) = match sub_long.namen.as_ref() {
                            Some((head, tail)) => (
                                quote!(
                                    #crate_ident::NonEmpty {
                                        head: #head,
                                        tail: vec![#(#tail),*]
                                    }
                                ),
                                head,
                            ),
                            None => (quote!(#name), &name),
                        };
                        let sub_kurz_ts = sub_short.into_vec_tokens(erster, sub_long.case);
                        let $prefix_and_names =
                            (sub_long_prefix, sub_lang_ts, sub_short_prefix, sub_kurz_ts);
                    }
                }
                match (
                    name.as_str(),
                    create_help.as_mut(),
                    create_version.as_mut(),
                    default_program_settings.as_mut(),
                ) {
                    ("hilfe" | "help", Some(erstelle_hilfe), _, _) => {
                        let mut sub_program_settings = ProgramSettings {
                            name: ProgramName(None),
                            version: ProgramVersion::Unspecified,
                            description: Some(ProgramDescription(None)),
                        };
                        recursive!(Some(&mut sub_program_settings), sub_language, prefix_and_name,);
                        let sub_program_description = ProgramSettings {
                            name: sub_program_settings.name,
                            version: sub_program_settings.version,
                            description: sub_program_settings.description.expect(
                                "Some-Wert für Programm-Description wird bei rekursiven Aufruf \
                                 nie None!",
                            ),
                        };
                        let default_language = if name == "hilfe" { Deutsch } else { English };
                        **erstelle_hilfe = CreateHelp(Some(Box::new(create_help_method(
                            &sub_language.unwrap_or(default_language),
                            Some(prefix_and_name),
                            sub_program_description,
                        ))));
                    },
                    ("version", _, Some(erstelle_version), _) => {
                        let mut sub_program_settings = ProgramSettings {
                            name: ProgramName(None),
                            version: ProgramVersion::Unspecified,
                            description: None,
                        };
                        recursive!(Some(&mut sub_program_settings), sub_language, prefix_and_names,);
                        let sub_program_description = ProgramSettings {
                            name: sub_program_settings.name,
                            version: sub_program_settings.version,
                            description: (),
                        };
                        **erstelle_version = CreateVersion(Some(Box::new(create_version_method(
                            sub_language,
                            Some(prefix_and_names),
                            sub_program_description,
                        ))));
                    },
                    ("programm" | "program", _, _, Some(default_program_settings)) => {
                        let mut sub_program_settings = ProgramSettings {
                            name: ProgramName(None),
                            version: ProgramVersion::Unspecified,
                            description: Some(ProgramDescription(None)),
                        };
                        let result = parse_value_argument(
                            sub_args,
                            None,
                            None,
                            None,
                            None,
                            Some(&mut sub_program_settings),
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
                        );
                        if let Err(create_error) = result {
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
                        **default_program_settings = ProgramSettings {
                            name: sub_program_settings.name,
                            version: sub_program_settings.version,
                            description: sub_program_settings.description.expect(
                                "Some-Wert für Programm-Description wird bei rekursivem Aufruf \
                                 nie None!",
                            ),
                        }
                    },
                    ("case", _, _, _) => {
                        for sub_arg in sub_args {
                            if let Argument {
                                name: sub_name,
                                value: ArgumentValue::Stream(ts),
                                ..
                            } = sub_arg
                            {
                                /// Value for an unsupported sub-argument.
                                macro_rules! error_argument {
                                    () => {
                                        Argument {
                                            span,
                                            name,
                                            value: ArgumentValue::SubArgument(vec![Argument {
                                                span,
                                                name: sub_name,
                                                value: ArgumentValue::Stream(ts),
                                            }]),
                                        }
                                    };
                                }
                                let Some(case) = Case::parse(&ts) else {
                                    return Err(Box::new(move |arg_name| Unsupported {
                                        arg_name,
                                        argument: error_argument!(),
                                    }));
                                };
                                match sub_name.as_str() {
                                    "lang_präfix" | "long_prefix" => {
                                        set_argument_case!(long_prefix, case, error_argument!());
                                    },
                                    "lang" | "long" => {
                                        set_argument_case!(long_names, case, error_argument!());
                                    },
                                    "kurz_präfix" | "short_prefix" => {
                                        set_argument_case!(short_prefix, case, error_argument!());
                                    },
                                    "kurz" | "short" => {
                                        set_argument_case!(short_names, case, error_argument!());
                                    },
                                    "invertiere_präfix" | "invert_prefix" => {
                                        set_argument_case!(invert_prefix, case, error_argument!());
                                    },
                                    "invertiere_infix" | "invert_infix" => {
                                        set_argument_case!(invert_infix, case, error_argument!());
                                    },
                                    "wert_infix" | "value_infix" => {
                                        set_argument_case!(value_infix, case, error_argument!());
                                    },
                                    _ => {
                                        return Err(Box::new(move |arg_name| Unsupported {
                                            arg_name,
                                            argument: error_argument!(),
                                        }));
                                    },
                                }
                            } else {
                                return Err(Box::new(move |arg_name| Unsupported {
                                    arg_name,
                                    argument: Argument {
                                        span,
                                        name,
                                        value: ArgumentValue::SubArgument(vec![sub_arg]),
                                    },
                                }));
                            }
                        }
                    },
                    _ => {
                        return Err(Box::new(move |arg_name| Unsupported {
                            arg_name,
                            argument: Argument {
                                span,
                                name,
                                value: ArgumentValue::SubArgument(sub_args),
                            },
                        }));
                    },
                }
            },
        }
    }
    Ok(())
}

/// Unsupported type for the derive macro: only structs are supported.
#[derive(Debug)]
pub(crate) enum UnsupportedType {
    /// enum
    Enum,
    /// union
    Union,
    /// Unknown
    Unknown,
}

impl Display for UnsupportedType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use UnsupportedType::{Enum, Union, Unknown};
        formatter.write_str(match self {
            Enum => "enum",
            Union => "union",
            Unknown => "Unknown",
        })
    }
}

/// Error while parsing a struct and its attributes.
#[derive(Debug)]
pub(crate) enum Error {
    /// Error returned when a [`venial`] parser cannot parse the input tokens.
    Venial(venial::Error),
    /// Error while splitting arguments.
    SplitArguments(SplitArgumentsError),
    /// Error while parsing a value.
    ParseValue(ParseValueError),
    /// The type is not a `struct`.
    NotStruct {
        /// Parsed kind of type.
        typ: UnsupportedType,
        /// Location of the unsupported type keyword.
        span: proc_macro2::Span,
        /// Macro input.
        input: TokenStream,
    },
    /// Type with generics as macro input.
    Generics {
        /// Number of generic parameters.
        count: usize,
        /// `where` clause of the type.
        where_clause: bool,
        /// Location of the generic parameters or `where` clause.
        span: proc_macro2::Span,
    },
    /// Unnamed field in the `struct`.
    UnnamedFields(proc_macro2::Span),
    /// Field with an empty name.
    EmptyFieldName(Ident),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use Error::{
            EmptyFieldName, Generics, NotStruct, ParseValue, SplitArguments, UnnamedFields, Venial,
        };
        match self {
            Venial(error) => write!(formatter, "{error}"),
            SplitArguments(error) => write!(formatter, "{error}"),
            ParseValue(error) => write!(formatter, "{error}"),
            NotStruct { typ, input, .. } => {
                write!(formatter, "Only structs are supported, but received {typ}: {input}")
            },
            Generics { count, where_clause, .. } => {
                write!(
                    formatter,
                    "Only structs without generics are supported, but received {count} \
                     parameter(s)"
                )?;
                if *where_clause {
                    write!(formatter, " and a where clause")?;
                }
                write!(formatter, ".")
            },
            UnnamedFields(_) => formatter.write_str("Only named fields are supported."),
            EmptyFieldName(ident) => write!(formatter, "Named field has an empty name: {ident}"),
        }
    }
}

impl From<venial::Error> for Error {
    fn from(input: venial::Error) -> Error {
        Error::Venial(input)
    }
}

impl From<SplitArgumentsError> for Error {
    fn from(input: SplitArgumentsError) -> Error {
        Error::SplitArguments(input)
    }
}

impl From<ParseValueError> for Error {
    fn from(input: ParseValueError) -> Error {
        Error::ParseValue(input)
    }
}

impl CompileError for Error {
    fn span(&self) -> proc_macro2::Span {
        match self {
            Self::ParseValue(ParseValueError::Unsupported { argument, .. }) => argument.span,
            Self::ParseValue(ParseValueError::NoLongName { span, .. })
            | Self::NotStruct { span, .. }
            | Self::Generics { span, .. }
            | Self::UnnamedFields(span) => *span,
            Self::Venial(error) => error.span(),
            Self::SplitArguments(error) => error.span(),
            Self::EmptyFieldName(ident) => ident.span(),
        }
    }
}

/// Obtain the [`Ok`] value or return the [`Err`] value immediately.
macro_rules! unwrap_or_return {
    ($result:expr $(, $($arg:expr)+)? $(,)?) => {
        match $result {
            Ok(wert) => wert,
            Err(function) => return Err(function($($($arg)+)?).into()),
        }
    };
}

/// Implementation of the derive macro for the [`Parse`] trait.
#[expect(
    clippy::too_many_lines,
    reason = "Deriving Parse requires collecting and translating all struct and field attributes."
)]
pub(crate) fn derive_parse(input: TokenStream) -> Result<TokenStream, Error> {
    use Error::{EmptyFieldName, Generics, NotStruct, UnnamedFields};
    let item = parse_item(input.clone())?;
    // Item als #[non_exhaustive] markiert
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "venial::Item in non_exhaustive. Only Types are supported."
    )]
    let Struct { fields, name, generic_params, where_clause, attributes, .. } = match item {
        Item::Struct(struct_) => struct_,
        Item::Enum(enum_) => {
            return Err(NotStruct {
                typ: UnsupportedType::Enum,
                span: enum_.tk_enum.span(),
                input,
            });
        },
        Item::Union(union) => {
            return Err(NotStruct {
                typ: UnsupportedType::Union,
                span: union.tk_union.span(),
                input,
            });
        },
        _ => {
            return Err(NotStruct {
                typ: UnsupportedType::Unknown,
                span: input
                    .clone()
                    .into_iter()
                    .next()
                    .map_or_else(proc_macro2::Span::call_site, |tt| tt.span()),
                input,
            });
        },
    };
    let param_count = generic_params.as_ref().map_or(0, |param_list| param_list.params.len());
    let has_where_clause = where_clause.is_some();
    if (param_count > 0) || has_where_clause {
        let span = generic_params.as_ref().map_or_else(
            || {
                where_clause
                    .expect("where clause exists when generic parameters do not")
                    .tk_where
                    .span()
            },
            |params| params.tk_l_bracket.span(),
        );
        return Err(Generics { count: param_count, where_clause: has_where_clause, span });
    }
    let mut args = Vec::new();
    for attr in attributes {
        if path_is_ident(&attr, "kommandozeilen_argumente") {
            split_parenthesized_arguments(Vec::new(), &mut args, attr.value)?;
        }
    }
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    // let version = env!("CARGO_PKG_VERSION");
    // CARGO_PKG_NAME — The name of your package.
    // CARGO_PKG_VERSION — The full version of your package.
    // CARGO_PKG_AUTHORS — Colon separated list of authors from the manifest of your package.
    // CARGO_PKG_DESCRIPTION — The description from the manifest of your package.
    // CARGO_BIN_NAME — The name of the binary that is currently being compiled (if it is a binary).
    // This name does not include any file extension, such as .exe
    let mut create_version = CreateVersion(None);
    let mut create_help = CreateHelp(None);
    let mut language = None;
    let mut default_program_setttings = ProgramSettings {
        name: ProgramName(None),
        version: ProgramVersion::Unspecified,
        description: ProgramDescription(None),
    };
    let mut long_prefix = LongPrefix::default();
    let mut short_prefix = ShortPrefix::default();
    let mut invert_prefix = InvertPrefix::default();
    let mut invert_infix = InvertInfix::default();
    let mut value_infix = ValueInfix::default();
    let mut meta_var = None;
    let crate_ident = crate_ident();
    unwrap_or_return!(
        parse_value_argument(
            args,
            Some(&mut language),
            Some(&mut default_program_setttings),
            Some(&mut create_help),
            Some(&mut create_version),
            None,
            Some(&mut long_prefix),
            None,
            Some(&mut short_prefix),
            None,
            Some(&mut invert_prefix),
            Some(&mut invert_infix),
            Some(&mut value_infix),
            Some(&mut meta_var),
            None,
            None,
        ),
        None
    );
    let language = language.unwrap_or(English);
    let meta_var = if let Some(meta_var) = meta_var {
        quote!(#meta_var)
    } else {
        let language_tokens = language.token_stream();
        quote!(#language_tokens.meta_var)
    };
    let mut tuples = Vec::new();
    let iter: Box<dyn Iterator<Item = NamedField>> = match fields {
        Fields::Unit => Box::new(iter::empty()),
        Fields::Named(named_fields) => {
            Box::new(named_fields.fields.inner.into_iter().map(|(field, _punct)| field))
        },
        Fields::Tuple(tuple_fields) => return Err(UnnamedFields(tuple_fields.tk_parens.span)),
    };
    for field in iter {
        let NamedField { attributes: field_attrs, name: field_ident, ty: field_type, .. } = field;
        let mut help_literals = Vec::new();
        let field_ident_str = field_ident.to_string();
        if field_ident_str.is_empty() {
            return Err(EmptyFieldName(field_ident));
        }
        let mut lang = quote!(#field_ident_str);
        let mut kurz = quote!(None::<&str>);
        let mut field_long_prefix = long_prefix.clone();
        let mut field_short_prefix = short_prefix.clone();
        let mut field_invert_prefix = invert_prefix.clone();
        let mut field_invert_infix = invert_infix.clone();
        let mut field_value_infix = value_infix.clone();
        let mut field_meta_var = None;
        let mut standard =
            DefaultValue(quote!(<#field_type as ::#crate_ident::ParseArgument>::default()));
        let mut field_argument = FieldArgument::EnumArgument;
        for attr in field_attrs {
            if path_is_ident(&attr, "doc") {
                let args_str = quote!(#(attr.meta)).to_string();
                if let Some(stripped) =
                    args_str.strip_prefix("= \"").and_then(|string| string.strip_suffix('"'))
                {
                    let trimmed = stripped.trim();
                    if !trimmed.is_empty() {
                        help_literals.push(trimmed.to_owned());
                    }
                }
            } else if path_is_ident(&attr, "kommandozeilen_argumente") {
                let mut field_args = Vec::new();
                split_parenthesized_arguments(
                    vec![field_ident.to_string()],
                    &mut field_args,
                    attr.value,
                )?;
                let mut long_names = LongNames::default();
                let mut short_names = ShortNames::default();
                unwrap_or_return!(
                    parse_value_argument(
                        field_args,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(&mut field_long_prefix),
                        Some(&mut long_names),
                        Some(&mut field_short_prefix),
                        Some(&mut short_names),
                        Some(&mut field_invert_prefix),
                        Some(&mut field_invert_infix),
                        Some(&mut field_value_infix),
                        Some(&mut field_meta_var),
                        Some(&mut standard),
                        Some(&mut field_argument),
                    ),
                    Some(field_ident_str)
                );
                let erster = if let Some((head, tail)) = long_names.namen.as_ref() {
                    lang = if let Some(case) = long_names.case {
                        quote!(
                            ::#crate_ident::NonEmpty {
                                head: ::#crate_ident::Compare {
                                    string: ::#crate_ident::Normalized::new(#head),
                                    case: #case,
                                },
                                tail: vec![#(::#crate_ident::Compare {
                                    string: ::#crate_ident::Normalized::new(#tail),
                                    case: #case,
                                }),*],
                            }
                        )
                    } else {
                        quote!(
                            ::#crate_ident::NonEmpty {
                                head: #head,
                                tail: vec![#(#tail),*]
                            }
                        )
                    };
                    head
                } else {
                    lang = quote!(#field_ident_str);
                    &field_ident_str
                };
                kurz = short_names.into_vec_tokens(erster, long_names.case);
            } else {
                // unused attribute
            }
        }
        let field_long_prefix = field_long_prefix.token_stream(&language);
        let field_short_prefix = field_short_prefix.token_stream(&language);
        let field_invert_prefix = field_invert_prefix.token_stream(&language);
        let field_invert_infix = field_invert_infix.token_stream(&language);
        let field_value_infix = field_value_infix.token_stream(&language);
        let field_meta_var = if let Some(MetaVar(string)) = field_meta_var {
            quote!(#string)
        } else {
            meta_var.clone()
        };
        let mut help_string = String::new();
        for part_string in help_literals {
            if !help_string.is_empty() {
                help_string.push(' ');
            }
            help_string.push_str(&part_string);
        }
        let hilfe =
            if help_string.is_empty() { quote!(None::<&str>) } else { quote!(Some(#help_string)) };
        let create_description = quote!(
            let description = ::#crate_ident::Description::new(
                #field_long_prefix,
                #lang,
                #field_short_prefix,
                #kurz,
                #hilfe,
                #standard,
            );
        );
        let create_arguments = field_argument.create_arguments(
            &create_description,
            &field_invert_prefix,
            &field_invert_infix,
            &field_value_infix,
            &field_meta_var,
            &field_type,
        );
        tuples.push((field_ident, create_arguments));
    }
    let (idents, create_arguments): (Vec<_>, Vec<_>) = tuples.into_iter().unzip();
    let combined = quote!(
        #(
            let #idents = ::#crate_ident::Arguments::from(#create_arguments);
        )*
        ::#crate_ident::combine!(|#(#idents),*| Self {#(#idents),*}, #(#idents),*)
    );
    let with_version = if let CreateVersion(Some(add_version)) = create_version {
        add_version(
            combined,
            language,
            &default_program_setttings.name,
            &default_program_setttings.version,
        )
    } else {
        combined
    };
    let with_help = if let CreateHelp(Some(add_help)) = create_help {
        add_help(
            with_version,
            &default_program_setttings.name,
            &default_program_setttings.version,
            &default_program_setttings.description,
        )
    } else {
        with_version
    };
    let ts = quote! {
        #[expect(
            clippy::shadow_unrelated,
            reason = "Generated bindings use user-provided field names."
        )]
        impl ::#crate_ident::Parse for #name {
            type Error = String;

            fn arguments<'t>() -> ::#crate_ident::Arguments<'t, Self, Self::Error> {
                #with_help
            }
        }
    };
    Ok(ts)
}
