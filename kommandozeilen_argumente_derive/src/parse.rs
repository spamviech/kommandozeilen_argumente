//! Implementierung für das derive-Macro des Parse-Traits.

use std::{
    fmt::{self, Display, Formatter},
    iter,
};

use litrs::StringLit;
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use unicode_segmentation::UnicodeSegmentation;
use venial::{parse_item, Fields, Item, NamedField, Struct, TypeExpr};

use crate::utility::{
    crate_name, genau_eines, path_is_ident, split_klammer_argumente, Argument, ArgumentWert, Case,
    SplitArgumenteFehler,
};

/// Sprache für ein Argument die Gesamt-Struktur. Beeinflusst Standard-Werte für weitere Einstellungen.
#[derive(Debug, Clone)]
enum Sprache {
    /// Deutsch
    Deutsch,
    /// Englisch
    English,
    /// Unbekannte Sprache
    TokenStream(TokenStream),
}
use Sprache::{Deutsch, English};

impl Sprache {
    /// Parse eine Sprache aus einem [`TokenStream`].
    fn parse(ts: TokenStream) -> Sprache {
        match genau_eines(ts.into_iter()) {
            Ok(TokenTree::Ident(ident)) => match ident.to_string().as_str() {
                "deutsch" | "german" => Deutsch,
                "englisch" | "english" => English,
                _ => Sprache::TokenStream(TokenTree::Ident(ident).into()),
            },
            Ok(tt) => Sprache::TokenStream(tt.into()),
            Err(fehler) => Sprache::TokenStream(fehler.collect()),
        }
    }

    /// Erzeuge einen [`TokenStream`] für die aktuelle Sprache.
    fn token_stream(&self) -> TokenStream {
        use Sprache::{Deutsch, English, TokenStream};
        let crate_name = crate_name();
        match self {
            Deutsch => quote!(::#crate_name::Sprache::DEUTSCH),
            English => quote!(::#crate_name::Sprache::ENGLISH),
            TokenStream(ts) => ts.clone(),
        }
    }
}

/// Trait um ein Argument-Wert zu parsen.
enum FeldArgument {
    /// `EnumArgument`
    EnumArgument,
    /// `FromStr`
    FromStr,
    /// Parse
    Parse,
}

impl FeldArgument {
    /// Erstelle den [`TokenStream`] zum erstellen der [`Argumente`] für das [`FeldArgument`].
    fn erstelle_args(
        self,
        erstelle_beschreibung: &TokenStream,
        feld_invertiere_präfix: &TokenStream,
        feld_invertiere_infix: &TokenStream,
        feld_wert_infix: &TokenStream,
        feld_meta_var: &TokenStream,
        feld_typ: &TypeExpr,
    ) -> TokenStream {
        let crate_name = crate_name();
        match self {
            FeldArgument::EnumArgument => {
                quote!({
                    #erstelle_beschreibung
                    ::#crate_name::ParseArgument::argumente(
                        beschreibung,
                        #feld_invertiere_präfix,
                        #feld_invertiere_infix,
                        #feld_wert_infix,
                        #feld_meta_var
                    )
                })
            },
            FeldArgument::FromStr => {
                quote!({
                    #erstelle_beschreibung
                    ::#crate_name::Wert {
                        beschreibung,
                        wert_infix: ::#crate_name::Vergleich::from(#feld_wert_infix),
                        meta_var: #feld_meta_var,
                        mögliche_werte: None,
                        parse: ::std::borrow::Cow::Borrowed(&|os_str: &::std::ffi::OsStr| {
                            if let Some(string) = os_str.to_str() {
                                string.parse::<#feld_typ>().map_err(
                                    |fehler| ::#crate_name::ParseFehler::ParseFehler(fehler.to_string())
                                )
                            } else {
                                Err(::#crate_name::ParseFehler::InvaliderString(::std::ffi::OsString::from(os_str)))
                            }
                        }),
                        anzeige: ::std::borrow::Cow::Borrowed(&ToString::to_string),
                        anzeige_fehler: ::std::borrow::Cow::Borrowed(&ToString::to_string),
                    }
                })
            },
            FeldArgument::Parse => {
                quote!(::#crate_name::Parse::kommandozeilen_argumente())
            },
        }
    }
}

/// Erstelle eine Funktion um eine `--version`-Flag zu einem `item` hinzuzufügen.
fn erstelle_version_methode(
    feste_sprache: Option<Sprache>,
    namen: Option<(LangPräfix, TokenStream, KurzPräfix, TokenStream)>,
    programm_einstellungen: ProgrammEinstellungen,
) -> impl FnOnce(TokenStream, Sprache) -> TokenStream {
    // TODO Standard-Wert für ProgrammEinstellungen (analog Sprache)
    let crate_name = crate_name();
    move |item, standard_sprache| {
        let sprache = feste_sprache.unwrap_or(standard_sprache);
        let sprache_ts = sprache.token_stream();
        let lang_standard = quote!(#sprache_ts.version_lang);
        let kurz_standard = quote!(#sprache_ts.version_kurz);
        let (lang_präfix, lang_namen, kurz_präfix, kurz_namen) = namen.unwrap_or_else(|| {
            (LangPräfix::default(), lang_standard, KurzPräfix::default(), kurz_standard)
        });
        let lang_präfix = lang_präfix.token_stream(&sprache);
        let kurz_präfix = kurz_präfix.token_stream(&sprache);
        let beschreibung = quote!(
            ::#crate_name::Beschreibung::neu(
                #lang_präfix,
                #lang_namen,
                #kurz_präfix,
                #kurz_namen,
                Some(#sprache_ts.version_beschreibung),
                None,
            )
        );
        let ProgrammEinstellungen {
            name: programm_name,
            version: programm_version,
            beschreibung: _,
        } = programm_einstellungen;
        let programm_version = ProgrammVersionDarstellung { programm_version, ist_option: false };
        quote!(
            #item.mit_version_frühes_beenden(
                #beschreibung,
                #programm_name,
                #programm_version,
            )
        )
    }
}

/// Erstelle eine Funktion um eine `--hilfe`-Flag zu einem `item` hinzuzufügen.
fn erstelle_hilfe_methode(
    sprache: &Sprache,
    namen: Option<(LangPräfix, TokenStream, KurzPräfix, TokenStream)>,
    programm_einstellungen: ProgrammEinstellungen,
) -> impl Fn(TokenStream) -> TokenStream {
    // TODO Standard-Wert für ProgrammEinstellungen (analog Sprache)
    let crate_name = crate_name();
    let sprache_ts = sprache.token_stream();
    let lang_standard = quote!(#sprache_ts.hilfe_lang);
    let kurz_standard = quote!(#sprache_ts.hilfe_kurz);
    let (lang_präfix, lang_namen, kurz_präfix, kurz_namen) = namen.unwrap_or_else(|| {
        (LangPräfix::default(), lang_standard, KurzPräfix::default(), kurz_standard)
    });
    let lang_präfix = lang_präfix.token_stream(sprache);
    let kurz_präfix = kurz_präfix.token_stream(sprache);
    let beschreibung = quote!(
        ::#crate_name::Beschreibung::neu(
            #lang_präfix,
            #lang_namen,
            #kurz_präfix,
            #kurz_namen,
            Some(#sprache_ts.hilfe_beschreibung),
            None,
        )
    );
    let ProgrammEinstellungen {
        name: programm_name,
        version: programm_version,
        beschreibung: programm_beschreibung,
    } = programm_einstellungen;
    let programm_version = ProgrammVersionDarstellung { programm_version, ist_option: true };
    move |item| {
        quote!(
            #item.mit_hilfe_frühes_beenden(
                &::#crate_name::argumente::hilfe::Standard,
                #beschreibung,
                #programm_name,
                #programm_beschreibung,
                #programm_version,
                #sprache_ts.standard,
                #sprache_ts.erlaubte_werte,
                #sprache_ts.optionen,
                #sprache_ts.syntax_präfix,
                #sprache_ts.syntax_padding,
                #sprache_ts.alternative_präfix,
                #sprache_ts.alternative_trennzeichen,
            )
        )
    }
}

/// Fehler beim parsen des Attributs eines Werts.
#[derive(Debug)]
pub(crate) enum ParseWertFehler {
    /// Das Attribut wird nicht unterstützt.
    NichtUnterstützt {
        /// Feld-Name, bei dem das Argument angegeben wurde.
        arg_name: Option<String>,
        /// Das unbekannte Argument.
        argument: Argument,
    },
    /// Kein Lang-Name angegeben.
    KeinLangName {
        /// Feld-Name, bei dem das Argument angegeben wurde.
        arg_name: Option<String>,
        /// String, der als Lang-Name geparst wurde.
        name: String,
    },
}

impl Display for ParseWertFehler {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use ArgumentWert::{KeinWert, Liste, Stream, Unterargument};
        use ParseWertFehler::{KeinLangName, NichtUnterstützt};
        match self {
            NichtUnterstützt { arg_name, argument: Argument { name, wert: KeinWert } } => {
                write!(formatter, "Argument ")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, "für {arg_name} ")?;
                }
                write!(formatter, "nicht unterstützt: {name}")
            },
            NichtUnterstützt {
                arg_name,
                argument: Argument { name, wert: wert @ Unterargument(_) },
            } => {
                write!(formatter, "Unterargument von {name} ")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, "für {arg_name} ")?;
                }
                write!(formatter, "nicht unterstützt: {wert}")
            },
            NichtUnterstützt { arg_name, argument: Argument { name, wert: wert @ Liste(_) } } => {
                write!(formatter, "Listen-Argument {name} ")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, "für {arg_name} ")?;
                }
                write!(formatter, "nicht unterstützt: {wert}")
            },
            NichtUnterstützt { arg_name, argument: Argument { name, wert: wert @ Stream(_) } } => {
                write!(formatter, "Benanntes Argument {name} ")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, "für {arg_name} ")?;
                }
                write!(formatter, "nicht unterstützt: {wert}")
            },
            KeinLangName { arg_name, name } => {
                write!(formatter, "Kein Langname ")?;
                if let Some(arg_name) = arg_name {
                    write!(formatter, "für {arg_name} ")?;
                }
                write!(formatter, "in expliziter Liste mit {name} angegeben!")
            },
        }
    }
}

/// Funktion um einen [`ParseWertFehler`] aus dem `arg_namen` zu erstellen.
type ErstelleFehler = Box<dyn FnOnce(Option<String>) -> ParseWertFehler>;

/// Beschreibung für das Programm.
#[derive(Debug, Clone)]
#[allow(clippy::missing_docs_in_private_items)]
struct ProgrammEinstellungen {
    name: ProgrammName,
    version: ProgrammVersion,
    beschreibung: Option<ProgrammBeschreibung>,
}

/// Programm-Name im Hilfe/Version-Text
#[derive(Debug, Clone)]
struct ProgrammName(Option<String>);

impl ToTokens for ProgrammName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(string) = &self.0 {
            tokens.extend(quote!(#string));
        } else {
            let crate_name = crate_name();
            tokens.extend(quote!(::#crate_name::crate_name!()));
        }
    }
}

/// Programm-Version im Hilfe/Version-Text
#[derive(Debug, Clone)]
enum ProgrammVersion {
    /// Der explizite Text wurde angegeben.
    Spezifiziert(String),
    /// Es wurde der Wert aus der `CARGO_PKG_VERSION`-Variable gewünscht.
    CrateMacro,
    /// Version beim `hilfe`/`help` ohne Sub-Argument.
    /// Verwende den Wert aus der `CARGO_PKG_VERSION`-Variable
    HilfeOhneSubArgument,
    /// Kein Wert wurde spezifiziert.
    Unspezifiziert,
}

/// Helper für [`ProgrammVersion`] um alternative [`ToTokens`]-Implementierungen anzubieten.
struct ProgrammVersionDarstellung {
    /// Die spezifizierte Version.
    programm_version: ProgrammVersion,
    /// Wir der Wert in einer Option angegeben.
    /// Bei einer Option wird für [`ProgrammVersion::Unspezifiziert`] [`None`] erzeugt,
    /// ansonsten der Wert aus der `CARGO_PKG_VERSION`-Variable.
    ist_option: bool,
}

impl ToTokens for ProgrammVersionDarstellung {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let crate_name = crate_name();
        let add_some_wenn_option =
            |ts: TokenStream| if self.ist_option { quote!(Some(#ts)) } else { ts };
        match &self.programm_version {
            ProgrammVersion::Spezifiziert(string) => {
                tokens.extend(add_some_wenn_option(quote!(#string)));
            },
            ProgrammVersion::CrateMacro | ProgrammVersion::HilfeOhneSubArgument => {
                tokens.extend(add_some_wenn_option(quote!(::#crate_name::crate_version!())));
            },
            ProgrammVersion::Unspezifiziert => {
                if self.ist_option {
                    tokens.extend(quote!(None));
                } else {
                    tokens.extend(add_some_wenn_option(quote!(::#crate_name::crate_version!())));
                }
            },
        }
    }
}

/// Programm-Beschreibung im Hilfe-Text
#[derive(Debug, Clone)]
struct ProgrammBeschreibung(Option<String>);

impl ToTokens for ProgrammBeschreibung {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(string) = &self.0 {
            tokens.extend(quote!(Some(#string)));
        } else {
            tokens.extend(quote!(None));
        }
    }
}

/// Funktion um eine `--hilfe`-Flag zu erstellen.
struct ErstelleHilfe(Option<Box<dyn FnOnce(TokenStream) -> TokenStream>>);

/// Funktion um eine `--version`-Flag zu erstellen.
struct ErstelleVersion(Option<Box<dyn FnOnce(TokenStream, Sprache) -> TokenStream>>);

/// Erstelle einen newtype-Typ mit identischer [`ToTokens`]-Implementierung.
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

/// Erzeuge newtypes für String-artige Typen, die im [`TokenStream`] in einen [`Vergleich`] verpackt werden.
macro_rules! vergleich_typen {
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
                fn token_stream(&self, sprache: &Sprache) -> TokenStream {
                    let crate_name = crate_name();
                    let string = if let Some(string) = &self.string {
                        quote!(#string)
                    } else {
                        let sprache_ts = sprache.token_stream();
                        quote!(#sprache_ts.$sprache_ident)
                    };
                    let case = self.case.unwrap_or_default();
                    quote!(::#crate_name::unicode::Vergleich {
                        string: ::#crate_name::unicode::Normalisiert::neu(#string),
                        case: #case,
                    })
                }
            }
        )*
    };
}

vergleich_typen! {
    LangPräfix(lang_präfix),
    KurzPräfix(kurz_präfix),
    InvertierePräfix(invertiere_präfix),
    InvertiereInfix(invertiere_infix),
    WertInfix(wert_infix),
}

/// Lang-Namen für ein Argument.
#[derive(Debug, Default)]
struct LangNamen {
    /// Die angegebenen Lang-Namen.
    namen: Option<(String, Vec<String>)>,
    /// Wird Groß-/Kleinschreibung beim Vergleich berücksichtigt.
    case: Option<Case>,
}

/// Angegebene Kurz-Namen, ohne [`Case`].
#[derive(Debug)]
enum KurzNamenEnum {
    /// Keine Kurz-Namen.
    Keiner,
    /// Automatisch abgeleiteter Kurz-Name.
    Auto,
    /// Explizit angegebene Kurz-Namen.
    Namen(Vec<String>),
}

/// Kurz-Namen für ein Argument.
#[derive(Debug)]
struct KurzNamen {
    /// Angegebene Kurz-Namen, ohne [`Case`].
    namen: KurzNamenEnum,
    /// Wird Groß-/Kleinschreibung beim Vergleich berücksichtigt.
    case: Option<Case>,
}

impl Default for KurzNamen {
    fn default() -> Self {
        KurzNamen { namen: KurzNamenEnum::Keiner, case: None }
    }
}

impl KurzNamen {
    /// Konvertiere in einen potenziell leeren [`Vec`] mit festen [`Strings`](String).
    fn into_vec(
        self,
        lang_name: &str,
        lang_namen_case: Option<Case>,
    ) -> (Vec<String>, Option<Case>) {
        match self.namen {
            KurzNamenEnum::Keiner => (Vec::new(), self.case),
            KurzNamenEnum::Auto => (
                vec![lang_name
                    .graphemes(true)
                    .next()
                    .expect("Langname ohne Graphemes!")
                    .to_owned()],
                self.case.or(lang_namen_case),
            ),
            KurzNamenEnum::Namen(namen) => (namen, self.case),
        }
    }

    /// Erzeuge einen [`TokenStream`] mit einem [`vec!`]-Macro für alle Kurz-Namen.
    fn into_vec_ts(self, lang_name: &str, lang_namen_case: Option<Case>) -> TokenStream {
        let (vec, case) = self.into_vec(lang_name, lang_namen_case);
        if vec.is_empty() {
            quote!(None::<&str>)
        } else {
            let crate_name = crate_name();
            if let Some(case) = case {
                quote!(vec![#(#crate_name::unicode::Vergleich {
                    string: #vec,
                    case: #case,
                }),*])
            } else {
                quote!(vec![#(#vec),*])
            }
        }
    }
}

/// Gebe den Wert eines String-Literal direkt zurück, ansonsten den [`TokenStream`] konvertiert mit [`ToString::to_string`].
fn literal_oder_to_string(token_stream: &TokenStream) -> String {
    if let Some(string_lit) = genau_eines(token_stream.clone().into_iter())
        .ok()
        .and_then(|literal| StringLit::try_from(literal).ok())
    {
        string_lit.into_value().into_owned()
    } else {
        token_stream.to_string()
    }
}

/// Parse die Attribute für ein Wert-Argument.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn parse_wert_arg(
    args: Vec<Argument>,
    mut sprache: Option<&mut Option<Sprache>>,
    mut erstelle_hilfe: Option<&mut ErstelleHilfe>,
    mut programm_einstellungen: Option<&mut ProgrammEinstellungen>,
    mut erstelle_version: Option<&mut ErstelleVersion>,
    mut lang_präfix: Option<&mut LangPräfix>,
    mut lang_namen: Option<&mut LangNamen>,
    mut kurz_präfix: Option<&mut KurzPräfix>,
    mut kurz_namen: Option<&mut KurzNamen>,
    mut invertiere_präfix: Option<&mut InvertierePräfix>,
    mut invertiere_infix: Option<&mut InvertiereInfix>,
    mut wert_infix: Option<&mut WertInfix>,
    mut meta_var: Option<&mut Option<MetaVar>>,
    mut standard: Option<&mut Standard>,
    mut feld_argument: Option<&mut FeldArgument>,
) -> Result<(), ErstelleFehler> {
    use ParseWertFehler::{KeinLangName, NichtUnterstützt};
    let crate_name = crate_name();
    /// Setzte den Wert für das Argument, oder gebe [`ParseWertFehler::NichtUnterstützt`] zurück.
    macro_rules! setze_argument {
        (< $([$mut_var: expr, $wert: expr $(,)?]),+ $(,)?>, $sub_arg: expr $(,)?) => {
            $(
                if let Some(var) = $mut_var.as_mut() {
                    **var = $wert;
                } else
            )+
            {
                return Err(Box::new(|arg_name| NichtUnterstützt {
                    arg_name,
                    argument: $sub_arg,
                }));
            }
        };
        ($mut_var: expr, $wert: expr, $sub_arg: expr $(,)?) => {
            setze_argument!(<[$mut_var, $wert]>, $sub_arg)
        };
    }
    /// Hilfs-Makro für [`setze_argument_namen!`], [`setze_argument_string!`] und [`setze_argument_case!`].
    macro_rules! setze_argument_feld {
        ($mut_var: expr, $feld:ident, $wert: expr, $sub_arg: expr) => {
            if let Some(var) = $mut_var.as_mut() {
                var.$feld = $wert;
            } else {
                return Err(Box::new(|arg_name| NichtUnterstützt {
                    arg_name,
                    argument: $sub_arg,
                }));
            }
        };
    }
    /// Setzte einen [`LangNamen`]/[`KurzNamen`] für ein Argument.
    macro_rules! setze_argument_namen {
        ($mut_var: expr, $wert: expr, $sub_arg: expr) => {
            setze_argument_feld!($mut_var, namen, $wert, $sub_arg)
        };
    }
    /// Setze einen [`String`]-Wert für ein Argument.
    macro_rules! setze_argument_string {
        ($mut_var: expr, $wert: expr, $sub_arg: expr) => {
            setze_argument_feld!($mut_var, string, Some($wert), $sub_arg)
        };
    }
    /// Setzte den [`Case`]-Wert für ein Argument.
    macro_rules! setze_argument_case {
        ($mut_var: expr, $wert: expr, $sub_arg: expr) => {
            setze_argument_feld!($mut_var, case, Some($wert), $sub_arg)
        };
    }
    for Argument { name, wert } in args {
        match wert {
            ArgumentWert::KeinWert => match name.as_str() {
                "name" => setze_argument!(
                    programm_einstellungen.as_mut().map(
                        #[allow(clippy::shadow_unrelated)]
                        |programm_beschreibung| { &mut programm_beschreibung.name }
                    ),
                    ProgrammName(None),
                    Argument { name, wert }
                ),
                "version" => setze_argument!(
                    <
                        [
                            programm_einstellungen.as_mut().map(
                                #[allow(clippy::shadow_unrelated)]
                                |programm_beschreibung| { &mut programm_beschreibung.version }
                            ),
                            ProgrammVersion::CrateMacro,
                        ],
                        [
                            erstelle_version,
                            ErstelleVersion(Some(Box::new(erstelle_version_methode(
                                None,
                                None,
                                ProgrammEinstellungen {
                                    name: ProgrammName(None),
                                    version: ProgrammVersion::Unspezifiziert,
                                    beschreibung: None
                                }
                            )))),
                        ],
                    >,
                    Argument { name, wert }
                ),
                "hilfe" => setze_argument!(
                    erstelle_hilfe,
                    ErstelleHilfe(Some(Box::new(erstelle_hilfe_methode(
                        &Deutsch,
                        None,
                        ProgrammEinstellungen {
                            name: ProgrammName(None),
                            version: ProgrammVersion::HilfeOhneSubArgument,
                            beschreibung: Some(ProgrammBeschreibung(None))
                        }
                    )))),
                    Argument { name, wert }
                ),
                "help" => setze_argument!(
                    erstelle_hilfe,
                    ErstelleHilfe(Some(Box::new(erstelle_hilfe_methode(
                        &English,
                        None,
                        ProgrammEinstellungen {
                            name: ProgrammName(None),
                            version: ProgrammVersion::HilfeOhneSubArgument,
                            beschreibung: Some(ProgrammBeschreibung(None))
                        }
                    )))),
                    Argument { name, wert }
                ),
                "kurz" | "short" => {
                    setze_argument_namen!(kurz_namen, KurzNamenEnum::Auto, Argument { name, wert });
                },
                "glätten" | "flatten" => {
                    setze_argument!(feld_argument, FeldArgument::Parse, Argument { name, wert });
                },
                "FromStr" => {
                    setze_argument!(feld_argument, FeldArgument::FromStr, Argument { name, wert });
                },
                "benötigt" | "required" => {
                    setze_argument!(standard, Standard(quote!(None)), Argument { name, wert });
                },
                _ => {
                    return Err(Box::new(|arg_name| NichtUnterstützt {
                        arg_name,
                        argument: Argument { name, wert },
                    }))
                },
            },
            ArgumentWert::Liste(liste) => match name.as_str() {
                "lang" | "long" => {
                    let mut namen_iter = liste.iter().map(literal_oder_to_string);
                    let (head, tail) = if let Some(head) = namen_iter.next() {
                        (head, namen_iter.collect())
                    } else {
                        return Err(Box::new(|arg_name| KeinLangName { arg_name, name }));
                    };
                    setze_argument_namen!(
                        lang_namen,
                        Some((head, tail)),
                        Argument { name, wert: ArgumentWert::Liste(liste) }
                    );
                },
                "kurz" | "short" => {
                    let namen_iter = liste.iter().map(literal_oder_to_string);
                    setze_argument_namen!(
                        kurz_namen,
                        KurzNamenEnum::Namen(namen_iter.collect()),
                        Argument { name, wert: ArgumentWert::Liste(liste) }
                    );
                },
                _ => {
                    return Err(Box::new(|arg_name| NichtUnterstützt {
                        arg_name,
                        argument: Argument { name, wert: ArgumentWert::Liste(liste) },
                    }))
                },
            },
            ArgumentWert::Stream(ts) => match name.as_str() {
                "sprache" | "language" => setze_argument!(
                    sprache,
                    Some(Sprache::parse(ts)),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "name" => setze_argument!(
                    programm_einstellungen.as_mut().map(
                        #[allow(clippy::shadow_unrelated)]
                        |programm_beschreibung| { &mut programm_beschreibung.name }
                    ),
                    ProgrammName(Some(literal_oder_to_string(&ts))),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "version" => setze_argument!(
                    programm_einstellungen.as_mut().map(
                        #[allow(clippy::shadow_unrelated)]
                        |programm_beschreibung| { &mut programm_beschreibung.version }
                    ),
                    ProgrammVersion::Spezifiziert(literal_oder_to_string(&ts)),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "beschreibung" | "description" => setze_argument!(
                    programm_einstellungen.as_mut().and_then(
                        #[allow(clippy::shadow_unrelated)]
                        |programm_beschreibung| { programm_beschreibung.beschreibung.as_mut() }
                    ),
                    ProgrammBeschreibung(Some(literal_oder_to_string(&ts))),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "lang_präfix" | "long_prefix" => setze_argument_string!(
                    lang_präfix,
                    literal_oder_to_string(&ts),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "kurz_präfix" | "short_prefix" => setze_argument_string!(
                    kurz_präfix,
                    literal_oder_to_string(&ts),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "invertiere_präfix" | "invert_prefix" => setze_argument_string!(
                    invertiere_präfix,
                    literal_oder_to_string(&ts),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "invertiere_infix" | "invert_infix" => setze_argument_string!(
                    invertiere_infix,
                    literal_oder_to_string(&ts),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "wert_infix" | "value_infix" => setze_argument_string!(
                    wert_infix,
                    literal_oder_to_string(&ts),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "meta_var" => setze_argument!(
                    meta_var,
                    Some(MetaVar(literal_oder_to_string(&ts))),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "standard" | "default" => setze_argument!(
                    standard,
                    Standard(quote!(Some(#ts))),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "lang" | "long" => setze_argument_namen!(
                    lang_namen,
                    Some((literal_oder_to_string(&ts), Vec::new())),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "kurz" | "short" => setze_argument_namen!(
                    kurz_namen,
                    KurzNamenEnum::Namen(vec![literal_oder_to_string(&ts)]),
                    Argument { name, wert: ArgumentWert::Stream(ts) }
                ),
                "case" => {
                    let Some(case) = Case::parse(&ts) else {
                        return Err(Box::new(|arg_name| NichtUnterstützt {
                            arg_name,
                            argument: Argument { name, wert: ArgumentWert::Stream(ts) },
                        }));
                    };
                    let argument = Argument { name, wert: ArgumentWert::Stream(ts) };
                    setze_argument_case!(lang_präfix, case, argument);
                    setze_argument_case!(lang_namen, case, argument);
                    setze_argument_case!(kurz_präfix, case, argument);
                    setze_argument_case!(kurz_namen, case, argument);
                    setze_argument_case!(invertiere_präfix, case, argument);
                    setze_argument_case!(invertiere_infix, case, argument);
                    setze_argument_case!(wert_infix, case, argument);
                },
                _ => {
                    return Err(Box::new(|arg_name| NichtUnterstützt {
                        arg_name,
                        argument: Argument { name, wert: ArgumentWert::Stream(ts) },
                    }))
                },
            },
            ArgumentWert::Unterargument(sub_args) => {
                /// Parse ein Unterargument rekursiv.
                macro_rules! rekursiv {
                    ($programm_beschreibung:expr, $sub_sprache:ident, $präfix_und_namen: ident $(,)?) => {
                        let mut $sub_sprache = None;
                        let mut sub_lang_präfix = LangPräfix::default();
                        let mut sub_lang = LangNamen::default();
                        let mut sub_kurz_präfix = KurzPräfix::default();
                        let mut sub_kurz = KurzNamen::default();
                        let result = parse_wert_arg(
                            sub_args,
                            Some(&mut $sub_sprache),
                            None,
                            $programm_beschreibung,
                            None,
                            Some(&mut sub_lang_präfix),
                            Some(&mut sub_lang),
                            Some(&mut sub_kurz_präfix),
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
                                NichtUnterstützt { arg_name, argument } => NichtUnterstützt {
                                    arg_name,
                                    argument: Argument {
                                        name,
                                        wert: ArgumentWert::Unterargument(vec![argument])
                                    }
                                },
                                fehler => fehler,
                            }))
                        };
                        let (sub_lang_ts, erster) = match sub_lang.namen.as_ref() {
                            Some((head, tail)) => (
                                quote!(
                                    #crate_name::NonEmpty {
                                        head: #head,
                                        tail: vec![#(#tail),*]
                                    }
                                ),
                                head,
                            ),
                            None => (quote!(#name), &name),
                        };
                        let sub_kurz_ts = sub_kurz.into_vec_ts(erster, sub_lang.case);
                        let $präfix_und_namen =
                            (sub_lang_präfix, sub_lang_ts, sub_kurz_präfix, sub_kurz_ts);
                    }
                }
                match (name.as_str(), erstelle_hilfe.as_mut(), erstelle_version.as_mut()) {
                    ("hilfe" | "help", Some(erstelle_hilfe), _) => {
                        let mut sub_programm_beschreibung = ProgrammEinstellungen {
                            name: ProgrammName(None),
                            version: ProgrammVersion::Unspezifiziert,
                            beschreibung: Some(ProgrammBeschreibung(None)),
                        };
                        rekursiv!(
                            Some(&mut sub_programm_beschreibung),
                            sub_sprache,
                            präfix_und_namen,
                        );
                        let standard_sprache = if name == "hilfe" { Deutsch } else { English };
                        **erstelle_hilfe = ErstelleHilfe(Some(Box::new(erstelle_hilfe_methode(
                            &sub_sprache.unwrap_or(standard_sprache),
                            Some(präfix_und_namen),
                            sub_programm_beschreibung,
                        ))));
                    },
                    ("version", _, Some(erstelle_version)) => {
                        let mut sub_programm_beschreibung = ProgrammEinstellungen {
                            name: ProgrammName(None),
                            version: ProgrammVersion::Unspezifiziert,
                            beschreibung: None,
                        };
                        rekursiv!(
                            Some(&mut sub_programm_beschreibung),
                            sub_sprache,
                            präfix_und_namen,
                        );
                        **erstelle_version =
                            ErstelleVersion(Some(Box::new(erstelle_version_methode(
                                sub_sprache,
                                Some(präfix_und_namen),
                                sub_programm_beschreibung,
                            ))));
                    },
                    // TODO programm/program(<opts>)
                    ("case", _, _) => {
                        for sub_arg in sub_args {
                            if let Argument { name: sub_name, wert: ArgumentWert::Stream(ts) } =
                                sub_arg
                            {
                                /// Wert für nicht-unterstütztes Unterargument.
                                macro_rules! error_argument {
                                    () => {
                                        Argument {
                                            name,
                                            wert: ArgumentWert::Unterargument(vec![Argument {
                                                name: sub_name,
                                                wert: ArgumentWert::Stream(ts),
                                            }]),
                                        }
                                    };
                                }
                                let Some(case) = Case::parse(&ts) else {
                                    return Err(Box::new(|arg_name| NichtUnterstützt {
                                        arg_name,
                                        argument: error_argument!(),
                                    }));
                                };
                                match sub_name.as_str() {
                                    "lang_präfix" | "long_prefix" => {
                                        setze_argument_case!(lang_präfix, case, error_argument!());
                                    },
                                    "lang" | "long" => {
                                        setze_argument_case!(lang_namen, case, error_argument!());
                                    },
                                    "kurz_präfix" | "short_prefix" => {
                                        setze_argument_case!(kurz_präfix, case, error_argument!());
                                    },
                                    "kurz" | "short" => {
                                        setze_argument_case!(kurz_namen, case, error_argument!());
                                    },
                                    "invertiere_präfix" | "invert_prefix" => {
                                        setze_argument_case!(
                                            invertiere_präfix,
                                            case,
                                            error_argument!()
                                        );
                                    },
                                    "invertiere_infix" | "invert_infix" => {
                                        setze_argument_case!(
                                            invertiere_infix,
                                            case,
                                            error_argument!()
                                        );
                                    },
                                    "wert_infix" | "value_infix" => {
                                        setze_argument_case!(wert_infix, case, error_argument!());
                                    },
                                    _ => {
                                        return Err(Box::new(|arg_name| NichtUnterstützt {
                                            arg_name,
                                            argument: error_argument!(),
                                        }))
                                    },
                                }
                            } else {
                                return Err(Box::new(|arg_name| NichtUnterstützt {
                                    arg_name,
                                    argument: Argument {
                                        name,
                                        wert: ArgumentWert::Unterargument(vec![sub_arg]),
                                    },
                                }));
                            }
                        }
                    },
                    _ => {
                        return Err(Box::new(|arg_name| NichtUnterstützt {
                            arg_name,
                            argument: Argument {
                                name,
                                wert: ArgumentWert::Unterargument(sub_args),
                            },
                        }))
                    },
                }
            },
        }
    }
    Ok(())
}

/// Nicht unterstützter Typ für das derive-Macro: Nur structs sind unterstützt.
#[derive(Debug)]
pub(crate) enum TypNichtUnterstützt {
    /// enum
    Enum,
    /// union
    Union,
    /// Unbekannt
    Unbekannt,
}

impl Display for TypNichtUnterstützt {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use TypNichtUnterstützt::{Enum, Unbekannt, Union};
        formatter.write_str(match self {
            Enum => "enum",
            Union => "union",
            Unbekannt => "Unbekannt",
        })
    }
}

/// Fehler beim Parsen des structs inklusive Attribute.
#[derive(Debug)]
pub(crate) enum Fehler {
    /// Error returned when a [`venial`] parser cannot parse the input tokens.
    Venial(venial::Error),
    /// Fehler beim teilen der Argumente.
    SplitArgumente(SplitArgumenteFehler),
    /// Fehler beim parsen eines Wertes.
    ParseWert(ParseWertFehler),
    /// Der Typ ist kein `struct`.
    KeinStruct {
        /// Der geparste Typ-Art.
        typ: TypNichtUnterstützt,
        /// Der Macro-Input.
        input: TokenStream,
    },
    /// Typ mit Generics als Macro-Argument.
    Generics {
        /// Anzahl der Generic-Parameter.
        anzahl: usize,
        /// `where`-Klausel des Typs.
        where_clause: bool,
    },
    /// Unbenanntes Feld im `struct`.
    FelderOhneName,
    /// Feld mit leerem Namen.
    LeererFeldName(Ident),
}

impl Display for Fehler {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        use Fehler::{
            FelderOhneName, Generics, KeinStruct, LeererFeldName, ParseWert, SplitArgumente, Venial,
        };
        match self {
            Venial(error) => write!(formatter, "{error}"),
            SplitArgumente(fehler) => write!(formatter, "{fehler}"),
            ParseWert(fehler) => write!(formatter, "{fehler}"),
            KeinStruct { typ, input } => {
                write!(formatter, "Nur structs unterstützt, aber {typ} bekommen: {input}")
            },
            Generics { anzahl, where_clause } => {
                write!(
                    formatter,
                    "Nur Structs ohne Generics unterstützt, aber {anzahl} Parameter "
                )?;
                if *where_clause {
                    write!(formatter, "und eine where-Klausel ")?;
                }
                write!(formatter, "bekommen.")
            },
            FelderOhneName => formatter.write_str("Nur benannte Felder unterstützt."),
            LeererFeldName(ident) => write!(formatter, "Benanntes Feld mit leerem Namen: {ident}"),
        }
    }
}

impl From<venial::Error> for Fehler {
    fn from(input: venial::Error) -> Fehler {
        Fehler::Venial(input)
    }
}

impl From<SplitArgumenteFehler> for Fehler {
    fn from(input: SplitArgumenteFehler) -> Fehler {
        Fehler::SplitArgumente(input)
    }
}

impl From<ParseWertFehler> for Fehler {
    fn from(input: ParseWertFehler) -> Fehler {
        Fehler::ParseWert(input)
    }
}

/// Erhalte den [`Ok`]-Wert, oder gebe direkt einen [`Err`]-Wert zurück.
macro_rules! unwrap_or_call_return {
    ($result:expr $(, $($arg:expr)+)? $(,)?) => {
        match $result {
            Ok(wert) => wert,
            Err(funktion) => return Err(funktion($($($arg)+)?).into()),
        }
    };
}

/// Implementierung für das derive-Macro des [`Parse`]-traits.
#[allow(clippy::too_many_lines)]
pub(crate) fn derive_parse(input: TokenStream) -> Result<TokenStream, Fehler> {
    use Fehler::{FelderOhneName, Generics, KeinStruct, LeererFeldName};
    let item = parse_item(input.clone())?;
    // Item als #[non_exhaustive] markiert
    #[allow(clippy::wildcard_enum_match_arm)]
    let Struct { fields, name, generic_params, where_clause, attributes, .. } = match item {
        Item::Struct(struct_) => struct_,
        Item::Enum(_) => return Err(KeinStruct { typ: TypNichtUnterstützt::Enum, input }),
        Item::Union(_) => return Err(KeinStruct { typ: TypNichtUnterstützt::Union, input }),
        _ => return Err(KeinStruct { typ: TypNichtUnterstützt::Unbekannt, input }),
    };
    let param_count = generic_params.map_or(0, |param_list| param_list.params.len());
    let has_where_clause = where_clause.is_some();
    if (param_count > 0) || has_where_clause {
        return Err(Generics { anzahl: param_count, where_clause: has_where_clause });
    }
    let mut args = Vec::new();
    for attr in attributes {
        if path_is_ident(&attr, "kommandozeilen_argumente") {
            split_klammer_argumente(Vec::new(), &mut args, attr.value)?;
        }
    }
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    // let version = env!("CARGO_PKG_VERSION");
    // CARGO_PKG_NAME — The name of your package.
    // CARGO_PKG_VERSION — The full version of your package.
    // CARGO_PKG_AUTHORS — Colon separated list of authors from the manifest of your package.
    // CARGO_PKG_DESCRIPTION — The description from the manifest of your package.
    // CARGO_BIN_NAME — The name of the binary that is currently being compiled (if it is a binary). This name does not include any file extension, such as .exe
    let mut erstelle_version = ErstelleVersion(None);
    let mut erstelle_hilfe = ErstelleHilfe(None);
    let mut sprache = None;
    let mut lang_präfix = LangPräfix::default();
    let mut kurz_präfix = KurzPräfix::default();
    let mut invertiere_präfix = InvertierePräfix::default();
    let mut invertiere_infix = InvertiereInfix::default();
    let mut wert_infix = WertInfix::default();
    let mut meta_var = None;
    let crate_name = crate_name();
    unwrap_or_call_return!(
        parse_wert_arg(
            args,
            Some(&mut sprache),
            Some(&mut erstelle_hilfe),
            None,
            Some(&mut erstelle_version),
            Some(&mut lang_präfix),
            None,
            Some(&mut kurz_präfix),
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
    let sprache = sprache.unwrap_or(English);
    let meta_var = if let Some(meta_var) = meta_var {
        quote!(#meta_var)
    } else {
        let sprache_ts = sprache.token_stream();
        quote!(#sprache_ts.meta_var)
    };
    let mut tuples = Vec::new();
    let iter: Box<dyn Iterator<Item = NamedField>> = match fields {
        Fields::Unit => Box::new(iter::empty()),
        Fields::Named(named_fields) => {
            Box::new(named_fields.fields.inner.into_iter().map(|(field, _punct)| field))
        },
        Fields::Tuple(_) => return Err(FelderOhneName),
    };
    for field in iter {
        let NamedField { attributes: field_attrs, name: field_ident, ty: feld_typ, .. } = field;
        let mut hilfe_lits = Vec::new();
        let field_ident_str = field_ident.to_string();
        if field_ident_str.is_empty() {
            return Err(LeererFeldName(field_ident));
        }
        let mut lang = quote!(#field_ident_str);
        let mut kurz = quote!(None::<&str>);
        let mut feld_lang_präfix = lang_präfix.clone();
        let mut feld_kurz_präfix = kurz_präfix.clone();
        let mut feld_invertiere_präfix = invertiere_präfix.clone();
        let mut feld_invertiere_infix = invertiere_infix.clone();
        let mut feld_wert_infix = wert_infix.clone();
        let mut feld_meta_var = None;
        let mut standard = Standard(quote!(#crate_name::parse::ParseArgument::standard()));
        let mut feld_argument = FeldArgument::EnumArgument;
        for attr in field_attrs {
            if path_is_ident(&attr, "doc") {
                let args_str = quote!(#(attr.meta)).to_string();
                if let Some(stripped) =
                    args_str.strip_prefix("= \"").and_then(|string| string.strip_suffix('"'))
                {
                    let trimmed = stripped.trim();
                    if !trimmed.is_empty() {
                        hilfe_lits.push(trimmed.to_owned());
                    }
                }
            } else if path_is_ident(&attr, "kommandozeilen_argumente") {
                let mut feld_args = Vec::new();
                split_klammer_argumente(vec![field_ident.to_string()], &mut feld_args, attr.value)?;
                let mut lang_namen = LangNamen::default();
                let mut kurz_namen = KurzNamen::default();
                unwrap_or_call_return!(
                    parse_wert_arg(
                        feld_args,
                        None,
                        None,
                        None,
                        None,
                        Some(&mut feld_lang_präfix),
                        Some(&mut lang_namen),
                        Some(&mut feld_kurz_präfix),
                        Some(&mut kurz_namen),
                        Some(&mut feld_invertiere_präfix),
                        Some(&mut feld_invertiere_infix),
                        Some(&mut feld_wert_infix),
                        Some(&mut feld_meta_var),
                        Some(&mut standard),
                        Some(&mut feld_argument),
                    ),
                    Some(field_ident_str)
                );
                let erster = if let Some((head, tail)) = lang_namen.namen.as_ref() {
                    lang = quote!(
                        ::#crate_name::NonEmpty {
                            head: #head,
                            tail: vec![#(#tail),*]
                        }
                    );
                    head
                } else {
                    lang = quote!(#field_ident_str);
                    &field_ident_str
                };
                kurz = kurz_namen.into_vec_ts(erster, lang_namen.case);
            } else {
                // nicht verwendetes Attribut
            }
        }
        let feld_lang_präfix = feld_lang_präfix.token_stream(&sprache);
        let feld_kurz_präfix = feld_kurz_präfix.token_stream(&sprache);
        let feld_invertiere_präfix = feld_invertiere_präfix.token_stream(&sprache);
        let feld_invertiere_infix = feld_invertiere_infix.token_stream(&sprache);
        let feld_wert_infix = feld_wert_infix.token_stream(&sprache);
        let feld_meta_var = if let Some(MetaVar(string)) = feld_meta_var {
            quote!(#string)
        } else {
            meta_var.clone()
        };
        let mut hilfe_string = String::new();
        for teil_string in hilfe_lits {
            if !hilfe_string.is_empty() {
                hilfe_string.push(' ');
            }
            hilfe_string.push_str(&teil_string);
        }
        let hilfe = if hilfe_string.is_empty() {
            quote!(None::<&str>)
        } else {
            quote!(Some(#hilfe_string))
        };
        let erstelle_beschreibung = quote!(
            let beschreibung = ::#crate_name::Beschreibung::neu(
                #feld_lang_präfix,
                #lang,
                #feld_kurz_präfix,
                #kurz,
                #hilfe,
                #standard,
            );
        );
        let erstelle_args = feld_argument.erstelle_args(
            &erstelle_beschreibung,
            &feld_invertiere_präfix,
            &feld_invertiere_infix,
            &feld_wert_infix,
            &feld_meta_var,
            &feld_typ,
        );
        tuples.push((field_ident, erstelle_args));
    }
    let (idents, erstelle_args): (Vec<_>, Vec<_>) = tuples.into_iter().unzip();
    let kombiniere = quote!(
        #(
            let #idents = ::#crate_name::Argumente::from(#erstelle_args);
        )*
        ::#crate_name::kombiniere!(|#(#idents),*| Self {#(#idents),*}, #(#idents),*)
    );
    let nach_version = if let ErstelleVersion(Some(version_hinzufügen)) = erstelle_version {
        version_hinzufügen(kombiniere, sprache)
    } else {
        kombiniere
    };
    let nach_hilfe = if let ErstelleHilfe(Some(hilfe_hinzufügen)) = erstelle_hilfe {
        hilfe_hinzufügen(nach_version)
    } else {
        nach_version
    };
    let ts = quote! {
        #[allow(clippy::shadow_unrelated, clippy::disallowed_script_idents)]
        impl ::#crate_name::Parse for #name {
            type Fehler = String;

            fn kommandozeilen_argumente<'t>() -> ::#crate_name::Argumente<'t, Self, Self::Fehler> {
                #nach_hilfe
            }
        }
    };
    Ok(ts)
}
