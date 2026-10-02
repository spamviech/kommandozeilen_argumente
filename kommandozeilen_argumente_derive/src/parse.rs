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

use crate::utility::{
    Argument, ArgumentValue, Case, SplitArgumentsError, crate_ident, exactly_one, path_is_ident,
    split_parenthesized_arguments,
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
                    ::#crate_ident::ParseArgument::arguments(
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
                                    |fehler| ::#crate_ident::ParseError::ParseError(fehler.to_string())
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
        let ProgramSettings {
            name: program_name,
            version: program_version,
            beschreibung: (),
        } = program_settings;
        let program_name = program_name.or(default_name);
        let program_version_display = ProgramVersionDisplay {
            program_version: program_version.or_default(default_version),
            ist_option: false,
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
        beschreibung: program_description,
    } = program_settings;
    move |item, default_name, default_version, default_description| {
        let program_name = program_name.or(default_name);
        let program_version_display = ProgramVersionDisplay {
            program_version: program_version.or_default(default_version),
            ist_option: true,
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
    },
}

impl Display for ParseValueError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentValue::{NoValue, List, Stream, SubArgument};
        use ParseValueError::{NoLongName, Unsupported};
        match self {
            Unsupported { arg_name, argument: Argument { name, wert: NoValue } } => {
                write!(formatter, "Unsupported argument")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {name}")
            },
            Unsupported {
                arg_name,
                argument: Argument { name, wert: wert @ SubArgument(_) },
            } => {
                write!(formatter, "Unsupported sub-argument of {name}")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {wert}")
            },
            Unsupported { arg_name, argument: Argument { name, wert: wert @ List(_) } } => {
                write!(formatter, "Unsupported list argument {name}")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {wert}")
            },
            Unsupported { arg_name, argument: Argument { name, wert: wert @ Stream(_) } } => {
                write!(formatter, "Unsupported named argument {name}")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, " for {arg_name}")?;
                }
                write!(formatter, ": {wert}")
            },
            NoLongName { arg_name, name } => {
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
#[allow(clippy::missing_docs_in_private_items)]
struct ProgramSettings<Description> {
    name: ProgramName,
    version: ProgramVersion,
    beschreibung: Description,
}

// TODO erlaube env-Variable
/// Program name in help/version text.
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

// TODO erlaube env-Variable
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
    /// Returns self if it contains [`ProgramVersion::Specified`], or [`ProgramVersion::CrateMacro`],
    /// otherwise returns the `alternative`.
    fn or_default(self, default: &Self) -> Self {
        match self {
            ProgramVersion::Specified(_) | ProgramVersion::CrateMacro => self,
            ProgramVersion::HelpWithoutSubArgument | ProgramVersion::Unspecified => {
                default.clone()
            },
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
    ist_option: bool,
}

impl ToTokens for ProgramVersionDisplay {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_ident = crate_ident();
        let add_some_if_option =
            |tokens: TokenStream| if self.ist_option { quote!(Some(#tokens)) } else { tokens };
        match &self.program_version {
            ProgramVersion::Specified(string) => {
                tokens.extend(add_some_if_option(quote!(#string)));
            },
            ProgramVersion::CrateMacro | ProgramVersion::HelpWithoutSubArgument => {
                tokens.extend(add_some_if_option(quote!(::#crate_ident::crate_version!())));
            },
            ProgramVersion::Unspecified => {
                if self.ist_option {
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
    #[allow(clippy::type_complexity)]
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
    #[allow(clippy::type_complexity)]
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
    Standard: TokenStream,
}

/// Create newtypes for string-like values wrapped in a [`Compare`] in the generated [`TokenStream`].
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

/// Return a string literal directly, or otherwise convert the [`TokenStream`] with [`ToString::to_string`].
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
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn parse_value_argument(
    args: Vec<Argument>,
    mut language: Option<&mut Option<Language>>,
    mut standard_programm_einstellungen: Option<&mut ProgramSettings<ProgramDescription>>,
    mut erstelle_hilfe: Option<&mut CreateHelp>,
    mut erstelle_version: Option<&mut CreateVersion>,
    mut programm_einstellungen: Option<&mut ProgramSettings<Option<ProgramDescription>>>,
    mut long_prefix: Option<&mut LongPrefix>,
    mut long_names: Option<&mut LongNames>,
    mut short_prefix: Option<&mut ShortPrefix>,
    mut short_names: Option<&mut ShortNames>,
    mut invertiere_präfix: Option<&mut InvertPrefix>,
    mut invertiere_infix: Option<&mut InvertInfix>,
    mut wert_infix: Option<&mut ValueInfix>,
    mut meta_var: Option<&mut Option<MetaVar>>,
    mut standard: Option<&mut Standard>,
    mut field_argument: Option<&mut FieldArgument>,
) -> Result<(), CreateError> {
    use ParseValueError::{NoLongName, Unsupported};
    let crate_ident = crate_ident();
    /// Set the argument value or return [`ParseValueError::Unsupported`].
    macro_rules! set_argument {
        (< $([$mut_var: expr, $wert: expr $(,)?]),+ $(,)?>, $sub_arg: expr $(,)?) => {
            $(
                if let Some(var) = $mut_var.as_mut() {
                    **var = $wert;
                } else
            )+
            {
                return Err(Box::new(|arg_name| Unsupported {
                    arg_name,
                    argument: $sub_arg,
                }));
            }
        };
        ($mut_var: expr, $wert: expr, $sub_arg: expr $(,)?) => {
            set_argument!(<[$mut_var, $wert]>, $sub_arg)
        };
    }
    /// Helper macro for the argument-setting macros below.
    macro_rules! set_argument_field {
        ($mut_var: expr, $feld:ident, $wert: expr, $sub_arg: expr) => {
            if let Some(var) = $mut_var.as_mut() {
                var.$feld = $wert;
            } else {
                return Err(Box::new(|arg_name| Unsupported {
                    arg_name,
                    argument: $sub_arg,
                }));
            }
        };
    }
    /// Set [`LongNames`] or [`ShortNames`] for an argument.
    macro_rules! set_argument_names {
        ($mut_var: expr, $wert: expr, $sub_arg: expr) => {
            set_argument_field!($mut_var, namen, $wert, $sub_arg)
        };
    }
    /// Set a [`String`] value for an argument.
    macro_rules! set_argument_string {
        ($mut_var: expr, $wert: expr, $sub_arg: expr) => {
            set_argument_field!($mut_var, string, Some($wert), $sub_arg)
        };
    }
    /// Set the [`Case`] value for an argument.
    macro_rules! set_argument_case {
        ($mut_var: expr, $wert: expr, $sub_arg: expr) => {
            set_argument_field!($mut_var, case, Some($wert), $sub_arg)
        };
    }
    for Argument { name, wert } in args {
        match wert {
            ArgumentValue::NoValue => match name.as_str() {
                "name" => set_argument!(
                    programm_einstellungen.as_mut().map(
                        #[allow(clippy::shadow_unrelated)]
                        |program_description| { &mut program_description.name }
                    ),
                    ProgramName(None),
                    Argument { name, wert }
                ),
                "version" => set_argument!(
                    <
                        [
                            programm_einstellungen.as_mut().map(
                                #[allow(clippy::shadow_unrelated)]
                                |program_description| { &mut program_description.version }
                            ),
                            ProgramVersion::CrateMacro,
                        ],
                        [
                            erstelle_version,
                            CreateVersion(Some(Box::new(create_version_method(
                                None,
                                None,
                                ProgramSettings {
                                    name: ProgramName(None),
                                    version: ProgramVersion::Unspecified,
                                    beschreibung: ()
                                }
                            )))),
                        ],
                    >,
                    Argument { name, wert }
                ),
                "hilfe" => set_argument!(
                    erstelle_hilfe,
                    CreateHelp(Some(Box::new(create_help_method(
                        &Deutsch,
                        None,
                        ProgramSettings {
                            name: ProgramName(None),
                            version: ProgramVersion::HelpWithoutSubArgument,
                            beschreibung: ProgramDescription(None)
                        }
                    )))),
                    Argument { name, wert }
                ),
                "help" => set_argument!(
                    erstelle_hilfe,
                    CreateHelp(Some(Box::new(create_help_method(
                        &English,
                        None,
                        ProgramSettings {
                            name: ProgramName(None),
                            version: ProgramVersion::HelpWithoutSubArgument,
                            beschreibung: ProgramDescription(None)
                        }
                    )))),
                    Argument { name, wert }
                ),
                "kurz" | "short" => {
                    set_argument_names!(short_names, ShortNamesKind::Auto, Argument { name, wert });
                },
                "glätten" | "flatten" => {
                    set_argument!(field_argument, FieldArgument::Parse, Argument { name, wert });
                },
                "FromStr" => {
                    set_argument!(field_argument, FieldArgument::FromStr, Argument { name, wert });
                },
                "benötigt" | "required" => {
                    set_argument!(standard, Standard(quote!(None)), Argument { name, wert });
                },
                _ => {
                    return Err(Box::new(|arg_name| Unsupported {
                        arg_name,
                        argument: Argument { name, wert },
                    }));
                },
            },
            ArgumentValue::List(liste) => match name.as_str() {
                "lang" | "long" => {
                    let mut namen_iter = liste.iter().map(literal_or_to_string);
                    let (head, tail) = if let Some(head) = namen_iter.next() {
                        (head, namen_iter.collect())
                    } else {
                        return Err(Box::new(|arg_name| NoLongName { arg_name, name }));
                    };
                    set_argument_names!(
                        long_names,
                        Some((head, tail)),
                        Argument { name, wert: ArgumentValue::List(liste) }
                    );
                },
                "kurz" | "short" => {
                    let namen_iter = liste.iter().map(literal_or_to_string);
                    set_argument_names!(
                        short_names,
                        ShortNamesKind::Names(namen_iter.collect()),
                        Argument { name, wert: ArgumentValue::List(liste) }
                    );
                },
                _ => {
                    return Err(Box::new(|arg_name| Unsupported {
                        arg_name,
                        argument: Argument { name, wert: ArgumentValue::List(liste) },
                    }));
                },
            },
            ArgumentValue::Stream(ts) => match name.as_str() {
                "sprache" | "language" => set_argument!(
                    language,
                    Some(Language::parse(ts)),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "name" => set_argument!(
                    programm_einstellungen.as_mut().map(
                        #[allow(clippy::shadow_unrelated)]
                        |program_description| { &mut program_description.name }
                    ),
                    ProgramName(Some(literal_or_to_string(&ts))),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "version" => set_argument!(
                    programm_einstellungen.as_mut().map(
                        #[allow(clippy::shadow_unrelated)]
                        |program_description| { &mut program_description.version }
                    ),
                    ProgramVersion::Specified(literal_or_to_string(&ts)),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "beschreibung" | "description" => set_argument!(
                    programm_einstellungen.as_mut().and_then(
                        #[allow(clippy::shadow_unrelated)]
                        |program_description| { program_description.beschreibung.as_mut() }
                    ),
                    ProgramDescription(Some(literal_or_to_string(&ts))),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "lang_präfix" | "long_prefix" => set_argument_string!(
                    long_prefix,
                    literal_or_to_string(&ts),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "kurz_präfix" | "short_prefix" => set_argument_string!(
                    short_prefix,
                    literal_or_to_string(&ts),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "invertiere_präfix" | "invert_prefix" => set_argument_string!(
                    invertiere_präfix,
                    literal_or_to_string(&ts),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "invertiere_infix" | "invert_infix" => set_argument_string!(
                    invertiere_infix,
                    literal_or_to_string(&ts),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "wert_infix" | "value_infix" => set_argument_string!(
                    wert_infix,
                    literal_or_to_string(&ts),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "meta_var" => set_argument!(
                    meta_var,
                    Some(MetaVar(literal_or_to_string(&ts))),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "standard" | "default" => set_argument!(
                    standard,
                    Standard(quote!(Some(#ts))),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "lang" | "long" => set_argument_names!(
                    long_names,
                    Some((literal_or_to_string(&ts), Vec::new())),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "kurz" | "short" => set_argument_names!(
                    short_names,
                    ShortNamesKind::Names(vec![literal_or_to_string(&ts)]),
                    Argument { name, wert: ArgumentValue::Stream(ts) }
                ),
                "case" => {
                    let Some(case) = Case::parse(&ts) else {
                        return Err(Box::new(|arg_name| Unsupported {
                            arg_name,
                            argument: Argument { name, wert: ArgumentValue::Stream(ts) },
                        }));
                    };
                    let argument = Argument { name, wert: ArgumentValue::Stream(ts) };
                    set_argument_case!(long_prefix, case, argument);
                    set_argument_case!(long_names, case, argument);
                    set_argument_case!(short_prefix, case, argument);
                    set_argument_case!(short_names, case, argument);
                    set_argument_case!(invertiere_präfix, case, argument);
                    set_argument_case!(invertiere_infix, case, argument);
                    set_argument_case!(wert_infix, case, argument);
                },
                _ => {
                    return Err(Box::new(|arg_name| Unsupported {
                        arg_name,
                        argument: Argument { name, wert: ArgumentValue::Stream(ts) },
                    }));
                },
            },
            ArgumentValue::SubArgument(sub_args) => {
                /// Parse a sub-argument recursively.
                macro_rules! recursive {
                    ($programm_einstellungen: expr, $sub_sprache: ident, $präfix_und_namen: ident $(,)?) => {
                        let mut $sub_sprache = None;
                        let mut sub_long_prefix = LongPrefix::default();
                        let mut sub_lang = LongNames::default();
                        let mut sub_short_prefix = ShortPrefix::default();
                        let mut sub_kurz = ShortNames::default();
                        let result = parse_value_argument(
                            sub_args,
                            Some(&mut $sub_sprache),
                            None,
                            None,
                            None,
                            $programm_einstellungen,
                            Some(&mut sub_long_prefix),
                            Some(&mut sub_lang),
                            Some(&mut sub_short_prefix),
                            Some(&mut sub_kurz),
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                        );
                        if let Err(erstelle_fehler) = result {
                            return Err(Box::new(|arg_name| match erstelle_fehler(arg_name) {
                                Unsupported { arg_name, argument } => Unsupported {
                                    arg_name,
                                    argument: Argument {
                                        name,
                                        wert: ArgumentValue::SubArgument(vec![argument])
                                    }
                                },
                                fehler => fehler,
                            }))
                        };
                        let (sub_lang_ts, erster) = match sub_lang.namen.as_ref() {
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
                        let sub_kurz_ts = sub_kurz.into_vec_tokens(erster, sub_lang.case);
                        let $präfix_und_namen =
                            (sub_long_prefix, sub_lang_ts, sub_short_prefix, sub_kurz_ts);
                    }
                }
                match (
                    name.as_str(),
                    erstelle_hilfe.as_mut(),
                    erstelle_version.as_mut(),
                    standard_programm_einstellungen.as_mut(),
                ) {
                    ("hilfe" | "help", Some(erstelle_hilfe), _, _) => {
                        let mut sub_programm_einstellungen = ProgramSettings {
                            name: ProgramName(None),
                            version: ProgramVersion::Unspecified,
                            beschreibung: Some(ProgramDescription(None)),
                        };
                        recursive!(
                            Some(&mut sub_programm_einstellungen),
                            sub_language,
                            präfix_und_namen,
                        );
                        let sub_program_description = ProgramSettings {
                            name: sub_programm_einstellungen.name,
                            version: sub_programm_einstellungen.version,
                            beschreibung: sub_programm_einstellungen
                                .beschreibung
                                .expect("Some-Wert für Programm-Description wird bei rekursiven Aufruf nie None!"),
                        };
                        let default_language = if name == "hilfe" { Deutsch } else { English };
                        **erstelle_hilfe = CreateHelp(Some(Box::new(create_help_method(
                            &sub_language.unwrap_or(default_language),
                            Some(präfix_und_namen),
                            sub_program_description,
                        ))));
                    },
                    ("version", _, Some(erstelle_version), _) => {
                        let mut sub_programm_einstellungen = ProgramSettings {
                            name: ProgramName(None),
                            version: ProgramVersion::Unspecified,
                            beschreibung: None,
                        };
                        recursive!(
                            Some(&mut sub_programm_einstellungen),
                            sub_language,
                            präfix_und_namen,
                        );
                        let sub_program_description = ProgramSettings {
                            name: sub_programm_einstellungen.name,
                            version: sub_programm_einstellungen.version,
                            beschreibung: (),
                        };
                        **erstelle_version =
                            CreateVersion(Some(Box::new(create_version_method(
                                sub_language,
                                Some(präfix_und_namen),
                                sub_program_description,
                            ))));
                    },
                    ("programm" | "program", _, _, Some(standard_programm_einstellungen)) => {
                        let mut sub_programm_einstellungen = ProgramSettings {
                            name: ProgramName(None),
                            version: ProgramVersion::Unspecified,
                            beschreibung: Some(ProgramDescription(None)),
                        };
                        let result = parse_value_argument(
                            sub_args,
                            None,
                            None,
                            None,
                            None,
                            Some(&mut sub_programm_einstellungen),
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
                        if let Err(erstelle_fehler) = result {
                            return Err(Box::new(|arg_name| match erstelle_fehler(arg_name) {
                                #[allow(clippy::shadow_unrelated)]
                                Unsupported { arg_name, argument } => Unsupported {
                                    arg_name,
                                    argument: Argument {
                                        name,
                                        wert: ArgumentValue::SubArgument(vec![argument]),
                                    },
                                },
                                fehler @ NoLongName { .. } => fehler,
                            }));
                        };
                        **standard_programm_einstellungen = ProgramSettings {
                            name: sub_programm_einstellungen.name,
                            version: sub_programm_einstellungen.version,
                            beschreibung: sub_programm_einstellungen
                                .beschreibung
                                .expect("Some-Wert für Programm-Description wird bei rekursivem Aufruf nie None!"),
                        };
                    },
                    ("case", _, _, _) => {
                        for sub_arg in sub_args {
                            if let Argument { name: sub_name, wert: ArgumentValue::Stream(ts) } =
                                sub_arg
                            {
                                /// Value for an unsupported sub-argument.
                                macro_rules! error_argument {
                                    () => {
                                        Argument {
                                            name,
                                            wert: ArgumentValue::SubArgument(vec![Argument {
                                                name: sub_name,
                                                wert: ArgumentValue::Stream(ts),
                                            }]),
                                        }
                                    };
                                }
                                let Some(case) = Case::parse(&ts) else {
                                    return Err(Box::new(|arg_name| Unsupported {
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
                                        set_argument_case!(
                                            invertiere_präfix,
                                            case,
                                            error_argument!()
                                        );
                                    },
                                    "invertiere_infix" | "invert_infix" => {
                                        set_argument_case!(
                                            invertiere_infix,
                                            case,
                                            error_argument!()
                                        );
                                    },
                                    "wert_infix" | "value_infix" => {
                                        set_argument_case!(wert_infix, case, error_argument!());
                                    },
                                    _ => {
                                        return Err(Box::new(|arg_name| Unsupported {
                                            arg_name,
                                            argument: error_argument!(),
                                        }));
                                    },
                                }
                            } else {
                                return Err(Box::new(|arg_name| Unsupported {
                                    arg_name,
                                    argument: Argument {
                                        name,
                                        wert: ArgumentValue::SubArgument(vec![sub_arg]),
                                    },
                                }));
                            }
                        }
                    },
                    _ => {
                        return Err(Box::new(|arg_name| Unsupported {
                            arg_name,
                            argument: Argument {
                                name,
                                wert: ArgumentValue::SubArgument(sub_args),
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
        use UnsupportedType::{Enum, Unknown, Union};
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
        /// Macro input.
        input: TokenStream,
    },
    /// Type with generics as macro input.
    Generics {
        /// Number of generic parameters.
        anzahl: usize,
        /// `where` clause of the type.
        where_clause: bool,
    },
    /// Unnamed field in the `struct`.
    UnnamedFields,
    /// Field with an empty name.
    EmptyFieldName(Ident),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use Error::{
            UnnamedFields, Generics, NotStruct, EmptyFieldName, ParseValue, SplitArguments, Venial,
        };
        match self {
            Venial(error) => write!(formatter, "{error}"),
            SplitArguments(fehler) => write!(formatter, "{fehler}"),
            ParseValue(fehler) => write!(formatter, "{fehler}"),
            NotStruct { typ, input } => {
                write!(formatter, "Only structs are supported, but received {typ}: {input}")
            },
            Generics { anzahl, where_clause } => {
                write!(formatter, "Only structs without generics are supported, but received {anzahl} parameter(s)")?;
                if *where_clause {
                    write!(formatter, " and a where clause")?;
                }
                write!(formatter, ".")
            },
            UnnamedFields => formatter.write_str("Only named fields are supported."),
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

/// Obtain the [`Ok`] value or return the [`Err`] value immediately.
macro_rules! unwrap_or_return {
    ($result:expr $(, $($arg:expr)+)? $(,)?) => {
        match $result {
            Ok(wert) => wert,
            Err(funktion) => return Err(funktion($($($arg)+)?).into()),
        }
    };
}

/// Implementation of the derive macro for the [`Parse`] trait.
#[allow(clippy::too_many_lines)]
pub(crate) fn derive_parse(input: TokenStream) -> Result<TokenStream, Error> {
    use Error::{UnnamedFields, Generics, NotStruct, EmptyFieldName};
    let item = parse_item(input.clone())?;
    // Item als #[non_exhaustive] markiert
    #[allow(clippy::wildcard_enum_match_arm)]
    let Struct { fields, name, generic_params, where_clause, attributes, .. } = match item {
        Item::Struct(struct_) => struct_,
        Item::Enum(_) => return Err(NotStruct { typ: UnsupportedType::Enum, input }),
        Item::Union(_) => return Err(NotStruct { typ: UnsupportedType::Union, input }),
        _ => return Err(NotStruct { typ: UnsupportedType::Unknown, input }),
    };
    let param_count = generic_params.map_or(0, |param_list| param_list.params.len());
    let has_where_clause = where_clause.is_some();
    if (param_count > 0) || has_where_clause {
        return Err(Generics { anzahl: param_count, where_clause: has_where_clause });
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
    // CARGO_BIN_NAME — The name of the binary that is currently being compiled (if it is a binary). This name does not include any file extension, such as .exe
    let mut erstelle_version = CreateVersion(None);
    let mut erstelle_hilfe = CreateHelp(None);
    let mut sprache = None;
    let mut standard_programm_einstellungen = ProgramSettings {
        name: ProgramName(None),
        version: ProgramVersion::Unspecified,
        beschreibung: ProgramDescription(None),
    };
    let mut long_prefix = LongPrefix::default();
    let mut short_prefix = ShortPrefix::default();
    let mut invertiere_präfix = InvertPrefix::default();
    let mut invertiere_infix = InvertInfix::default();
    let mut wert_infix = ValueInfix::default();
    let mut meta_var = None;
    let crate_ident = crate_ident();
    unwrap_or_return!(
        parse_value_argument(
            args,
            Some(&mut sprache),
            Some(&mut standard_programm_einstellungen),
            Some(&mut erstelle_hilfe),
            Some(&mut erstelle_version),
            None,
            Some(&mut long_prefix),
            None,
            Some(&mut short_prefix),
            None,
            Some(&mut invertiere_präfix),
            Some(&mut invertiere_infix),
            Some(&mut wert_infix),
            Some(&mut meta_var),
            None,
            None,
        ),
        None
    );
    let language = sprache.unwrap_or(English);
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
        Fields::Tuple(_) => return Err(UnnamedFields),
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
        let mut feld_long_prefix = long_prefix.clone();
        let mut feld_short_prefix = short_prefix.clone();
        let mut field_invert_prefix = invertiere_präfix.clone();
        let mut field_invert_infix = invertiere_infix.clone();
        let mut field_value_infix = wert_infix.clone();
        let mut field_meta_var = None;
        let mut standard = Standard(quote!(#crate_ident::parse::ParseArgument::standard()));
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
                split_parenthesized_arguments(vec![field_ident.to_string()], &mut field_args, attr.value)?;
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
                        Some(&mut feld_long_prefix),
                        Some(&mut long_names),
                        Some(&mut feld_short_prefix),
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
                    lang = quote!(
                        ::#crate_ident::NonEmpty {
                            head: #head,
                            tail: vec![#(#tail),*]
                        }
                    );
                    head
                } else {
                    lang = quote!(#field_ident_str);
                    &field_ident_str
                };
                kurz = short_names.into_vec_tokens(erster, long_names.case);
            } else {
                // nicht verwendetes Attribut
            }
        }
        let feld_long_prefix = feld_long_prefix.token_stream(&language);
        let feld_short_prefix = feld_short_prefix.token_stream(&language);
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
        let hilfe = if help_string.is_empty() {
            quote!(None::<&str>)
        } else {
            quote!(Some(#help_string))
        };
        let create_description = quote!(
            let description = ::#crate_ident::Description::new(
                #feld_long_prefix,
                #lang,
                #feld_short_prefix,
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
    let with_version = if let CreateVersion(Some(version_hinzufügen)) = erstelle_version {
        version_hinzufügen(
            combined,
            language,
            &standard_programm_einstellungen.name,
            &standard_programm_einstellungen.version,
        )
    } else {
        combined
    };
    let with_help = if let CreateHelp(Some(hilfe_hinzufügen)) = erstelle_hilfe {
        hilfe_hinzufügen(
            with_version,
            &standard_programm_einstellungen.name,
            &standard_programm_einstellungen.version,
            &standard_programm_einstellungen.beschreibung,
        )
    } else {
        with_version
    };
    let ts = quote! {
        #[allow(clippy::shadow_unrelated, clippy::disallowed_script_idents)]
        impl ::#crate_ident::Parse for #name {
            type Error = String;

            fn arguments<'t>() -> ::#crate_ident::Arguments<'t, Self, Self::Error> {
                #with_help
            }
        }
    };
    Ok(ts)
}
