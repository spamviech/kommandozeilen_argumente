//! Helper types and code-generation utilities for `#[derive(Parse)]`.

use std::fmt::{self, Display, Formatter};

use litrs::StringLit;
use proc_macro2::{TokenStream, TokenTree};
use quote::{ToTokens, quote};
use unicode_segmentation::UnicodeSegmentation;
use venial::TypeExpr;

use crate::utility::{Argument, ArgumentValue, Case, crate_ident, exactly_one};

/// Language for an argument structure. Influences default values for other settings.
#[derive(Debug, Clone)]
pub(super) enum Language {
    /// German
    Deutsch,
    /// English
    English,
    /// Unknown language
    TokenStream(TokenStream),
}
pub(super) use Language::{Deutsch, English};

impl Language {
    /// Parse a language from a [`TokenStream`].
    pub(super) fn parse(ts: TokenStream) -> Language {
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
    pub(super) fn token_stream(&self) -> TokenStream {
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
pub(super) enum FieldArgument {
    /// `EnumArgument`
    EnumArgument,
    /// `FromStr`
    FromStr,
    /// Parse
    Parse,
}

impl FieldArgument {
    /// Create the [`TokenStream`] that constructs the arguments for this [`FieldArgument`].
    pub(super) fn create_arguments(
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
pub(super) fn create_version_method(
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
pub(super) fn create_help_method<'t>(
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
pub(super) type CreateError = Box<dyn FnOnce(Option<String>) -> ParseValueError>;

/// Description of the program.
#[derive(Debug, Clone)]
pub(super) struct ProgramSettings<Description> {
    /// Program name specified through the derive macro.
    pub(super) name: ProgramName,
    /// Program version specified through the derive macro.
    pub(super) version: ProgramVersion,
    /// Program description specified through the derive macro.
    pub(super) description: Description,
}

/// Program name in help/version text.
/// Falls back to the `CARGO_PKG_NAME` environment variable when specifying [`None`].
#[derive(Debug, Clone)]
pub(super) struct ProgramName(pub(super) Option<String>);

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
pub(super) enum ProgramVersion {
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
pub(super) struct ProgramVersionDisplay {
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
pub(super) struct ProgramDescription(pub(super) Option<String>);

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
pub(super) struct CreateHelp(
    #[expect(
        clippy::type_complexity,
        reason = "The callback signature captures the generated help configuration."
    )]
    pub(super)  Option<
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
pub(super) struct CreateVersion(
    #[expect(
        clippy::type_complexity,
        reason = "The callback signature captures the generated version configuration."
    )]
    pub(super)  Option<
        Box<dyn FnOnce(TokenStream, Language, &ProgramName, &ProgramVersion) -> TokenStream>,
    >,
);

/// Create a newtype with an identical [`ToTokens`] implementation.
macro_rules! create_newtype {
    ($($name: ident : $type: ty),* $(,)?) => {
        $(
            #[derive(Debug, Clone)]
            pub(super) struct $name(pub(super) $type);

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
            pub(super) struct $name { pub(super) string: Option<String>, pub(super) case: Option<Case> }

            impl Default for $name {
                fn default() -> Self {
                    $name { string: None, case: None }
                }
            }

            impl $name {
                pub(super) fn token_stream(&self, language: &Language) -> TokenStream {
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
pub(super) struct LongNames {
    /// The specified long names.
    pub(super) namen: Option<(String, Vec<String>)>,
    /// Whether matching is case-sensitive.
    pub(super) case: Option<Case>,
}

/// Specified short names without [`Case`].
#[derive(Debug)]
pub(super) enum ShortNamesKind {
    /// No short names.
    None,
    /// Automatically derived short name.
    Auto,
    /// Explicitly specified short names.
    Names(Vec<String>),
}

/// Short names for an argument.
#[derive(Debug)]
pub(super) struct ShortNames {
    /// Specified short names without [`Case`].
    pub(super) namen: ShortNamesKind,
    /// Whether matching is case-sensitive.
    pub(super) case: Option<Case>,
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
    pub(super) fn into_vec_tokens(
        self,
        lang_name: &str,
        long_names_case: Option<Case>,
    ) -> TokenStream {
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
pub(super) fn literal_or_to_string(token_stream: &TokenStream) -> String {
    if let Some(string_lit) = exactly_one(token_stream.clone().into_iter())
        .ok()
        .and_then(|literal| StringLit::try_from(literal).ok())
    {
        string_lit.into_value()
    } else {
        token_stream.to_string()
    }
}
