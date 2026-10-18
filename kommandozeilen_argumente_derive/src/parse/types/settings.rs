//! Language and program help/version configuration.

use proc_macro2::{TokenStream, TokenTree};
use quote::{ToTokens, quote};
use venial::TypeExpr;

use super::names::{LongPrefix, ShortPrefix};
use crate::utility::{crate_ident, exactly_one};

/// Language for an argument structure. Influences default values for other settings.
#[derive(Debug, Clone)]
pub(in crate::parse) enum Language {
    /// German
    Deutsch,
    /// English
    English,
    /// Unknown language
    TokenStream(TokenStream),
}
pub(in crate::parse) use Language::{Deutsch, English};

impl Language {
    /// Parse a language from a [`TokenStream`].
    pub(in crate::parse) fn parse(ts: TokenStream) -> Language {
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
    pub(in crate::parse) fn token_stream(&self) -> TokenStream {
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
pub(in crate::parse) enum FieldArgument {
    /// `EnumArgument`
    EnumArgument,
    /// `FromStr`
    FromStr,
    /// Parse
    Parse,
}

impl FieldArgument {
    /// Create the [`TokenStream`] that constructs the arguments for this [`FieldArgument`].
    pub(in crate::parse) fn create_arguments(
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
pub(in crate::parse) fn create_version_method(
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
pub(in crate::parse) fn create_help_method<'t>(
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

/// Description of the program.
#[derive(Debug, Clone)]
pub(in crate::parse) struct ProgramSettings<Description> {
    /// Program name specified through the derive macro.
    pub(in crate::parse) name: ProgramName,
    /// Program version specified through the derive macro.
    pub(in crate::parse) version: ProgramVersion,
    /// Program description specified through the derive macro.
    pub(in crate::parse) description: Description,
}

/// Program name in help/version text.
/// Falls back to the `CARGO_PKG_NAME` environment variable when specifying [`None`].
#[derive(Debug, Clone)]
pub(in crate::parse) struct ProgramName(pub(in crate::parse) Option<String>);

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
pub(in crate::parse) enum ProgramVersion {
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
pub(in crate::parse) struct ProgramVersionDisplay {
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
pub(in crate::parse) struct ProgramDescription(pub(in crate::parse) Option<String>);

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
pub(in crate::parse) struct CreateHelp(
    #[expect(
        clippy::type_complexity,
        reason = "The callback signature captures the generated help configuration."
    )]
    pub(in crate::parse)  Option<
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
pub(in crate::parse) struct CreateVersion(
    #[expect(
        clippy::type_complexity,
        reason = "The callback signature captures the generated version configuration."
    )]
    pub(in crate::parse)  Option<
        Box<dyn FnOnce(TokenStream, Language, &ProgramName, &ProgramVersion) -> TokenStream>,
    >,
);
