//! Trait für Typen, die aus Kommandozeilen-Argumenten geparst werden können.

use std::{
    borrow::Cow,
    convert::identity,
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    num::NonZeroI32,
    str::FromStr,
};

use nonempty::NonEmpty;

use crate::{
    argumente::{
        einzelargument::EinzelArgument,
        flag::Flag,
        hilfe::{self, ErzeugeHilfeText},
        kombiniere::Kombiniere,
        wert::{EnumArgument, Wert},
        Argumente,
    },
    beschreibung::{ArgumentInput, Beschreibung, Description},
    dyn_to_owned::{self, Anzeige, Bool},
    ergebnis::{Ergebnis, Error, Fehler, ParseFehler, ZwischenErgebnis},
    sprache::{Language, Sprache},
    unicode::Vergleich,
};

#[cfg(any(feature = "derive", all(doc, not(doctest))))]
#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use kommandozeilen_argumente_derive::Parse;

/// Trait für Typen, die direkt mit dem (derive-Macro)[`derive@Parse`]
/// für das [`Parse`]-Trait verwendet werden können.
///
/// ## English
/// Trait for types directly usable with the [derive macro](derive@Parse] for the [`Parse`] trait.
#[allow(clippy::module_name_repetitions)]
pub trait ParseArgument: Sized {
    /// Erstelle ein [`Argumente`] mit den konfigurierten Eigenschaften.
    ///
    /// `invertiere_präfix` ist für Flag-Argumente gedacht,
    /// `meta_var` für Wert-Argumente.
    ///
    /// ## English
    /// Create and [`Arguments`] with the configured properties.
    ///
    /// `invertiere_präfix` is intended as the prefix to invert flag arguments,
    /// `meta_var` is intended as the meta-variable used in the help text for value arguments.
    fn argumente<'t>(
        beschreibung: Beschreibung<'t, Self>,
        invertiere_präfix: impl Into<Vergleich<'t>>,
        invertiere_infix: impl Into<Vergleich<'t>>,
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
    ) -> Argumente<'t, Self, String>;

    /// Sollen Argumente dieses Typs normalerweise einen Standard-Wert haben?
    ///
    /// ## English
    /// Should arguments of this type have a default value if left unspecified?
    fn standard() -> Option<Self>;

    /// Erstelle ein [Argumente] für die übergebene [`Beschreibung`].
    ///
    /// ## English synonym
    /// [`arguments_with_language`](ParseArgument::arguments_with_language)
    #[inline]
    #[allow(clippy::needless_lifetimes)]
    fn argumente_mit_sprache<'t>(
        beschreibung: Beschreibung<'t, Self>,
        sprache: Sprache,
    ) -> Argumente<'t, Self, String> {
        Self::argumente(
            beschreibung,
            sprache.invertiere_präfix,
            sprache.invertiere_infix,
            sprache.wert_infix,
            sprache.meta_var,
        )
    }

    /// Create an [Arguments] for the given [`Description`].
    ///
    /// ## Deutsches Synonym
    /// [`argumente_mit_sprache`](ParseArgument::argumente_mit_sprache)
    #[inline]
    #[allow(clippy::needless_lifetimes)]
    fn arguments_with_language<'t>(
        description: Description<'t, Self>,
        language: Language,
    ) -> Argumente<'t, Self, String> {
        Self::argumente_mit_sprache(description, language)
    }

    /// Erstelle ein [Argumente] für die übergebene [`Beschreibung`].
    ///
    /// ## English version
    /// [`new`](ParseArgument::new)
    #[inline]
    #[allow(clippy::needless_lifetimes)]
    fn neu<'t>(beschreibung: Beschreibung<'t, Self>) -> Argumente<'t, Self, String> {
        Self::argumente_mit_sprache(beschreibung, Sprache::DEUTSCH)
    }

    /// Create an [Argumente] for the [`Beschreibung`].
    ///
    /// ## Deutsche Version
    /// [`neu`](ParseArgument::neu)
    #[inline]
    #[allow(clippy::needless_lifetimes)]
    fn new<'t>(beschreibung: Beschreibung<'t, Self>) -> Argumente<'t, Self, String> {
        Self::argumente_mit_sprache(beschreibung, Sprache::ENGLISH)
    }
}

impl ParseArgument for bool {
    #[inline]
    fn argumente<'t>(
        beschreibung: Beschreibung<'t, Self>,
        invertiere_präfix: impl Into<Vergleich<'t>>,
        invertiere_infix: impl Into<Vergleich<'t>>,
        _wert_infix: impl Into<Vergleich<'t>>,
        _meta_var: &'t str,
    ) -> Argumente<'t, Self, String> {
        Argumente::from(Flag {
            beschreibung,
            invertiere_präfix: invertiere_präfix.into(),
            invertiere_infix: invertiere_infix.into(),
            anzeige: Cow::Borrowed(&<bool as ToString>::to_string),
            konvertiere: Cow::Borrowed(&identity),
        })
    }

    #[inline]
    fn standard() -> Option<Self> {
        Some(false)
    }
}

impl ParseArgument for String {
    #[inline]
    fn argumente<'t>(
        beschreibung: Beschreibung<'t, Self>,
        _invertiere_präfix: impl Into<Vergleich<'t>>,
        _invertiere_infix: impl Into<Vergleich<'t>>,
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
    ) -> Argumente<'t, Self, String> {
        Argumente::from(Wert {
            beschreibung,
            wert_infix: wert_infix.into(),
            meta_var,
            mögliche_werte: None,
            parse: Cow::Borrowed(&|os_str: &OsStr| {
                if let Some(string) = os_str.to_str() {
                    Ok(String::from(string))
                } else {
                    Err(ParseFehler::InvaliderString(OsString::from(os_str)))
                }
            }),
            anzeige: Cow::Borrowed(&<String as Clone>::clone),
            anzeige_fehler: Cow::Borrowed(&<String as Clone>::clone),
        })
    }

    #[inline]
    fn standard() -> Option<Self> {
        None
    }
}

/// Implementiere [`ParseArgument`] für primitive Zahlentypen.
macro_rules! impl_parse_argument {
    ($($type:ty),*$(,)?) => {$(
        impl ParseArgument for $type {
            #[inline]
            fn argumente<'t>(
                beschreibung: Beschreibung<'t,Self>,
                _invertiere_präfix: impl Into<Vergleich<'t>>,
                _invertiere_infix: impl Into<Vergleich<'t>>,
                wert_infix: impl Into<Vergleich<'t>>,
                meta_var: &'t str,
            ) -> Argumente<'t, Self, String> {
                Argumente::from(Wert {
                    beschreibung,
                    wert_infix: wert_infix.into(),
                    meta_var,
                    mögliche_werte: None,
                    parse: Cow::Borrowed(&|os_str: &OsStr| {
                        if let Some(string) = os_str.to_str() {
                            string.parse().map_err(
                                |err: <$type as FromStr>::Err| ParseFehler::ParseFehler(err.to_string())
                            )
                        } else {
                            Err(ParseFehler::InvaliderString(os_str.to_owned()))
                        }
                    }),
                    anzeige: Cow::Borrowed(&<$type as ToString>::to_string),
                    anzeige_fehler: Cow::Borrowed(&<String as Clone>::clone),
                })
            }

            #[inline]
            fn standard() -> Option<Self> {
                None
            }
        }
    )*};
}
impl_parse_argument! {i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize, f32, f64}

/// Hilfs-Typ für die [`ParseArgument`]-Implementierung von [`Option<T>`].
struct OptionHelper<'t, F, T, Fehler> {
    /// Finales anpassen des Ergebnis beim parser einer [`Option<T>`].
    ergebnis_anpassen: F,
    /// Argument-Definition für parsen einer [`Option<T>`].
    argumente: Argumente<'t, T, Fehler>,
}

impl<'t, T, Fehler, F: FnOnce(Ergebnis<'t, T, Fehler>) -> Ergebnis<'t, T, Fehler>>
    Kombiniere<'t, T, Fehler> for OptionHelper<'t, F, T, Fehler>
{
    #[inline]
    fn parse<'a>(
        self: Box<Self>,
        args: Box<dyn '_ + Iterator<Item = Option<&'a OsStr>>>,
    ) -> (ZwischenErgebnis<'t, T, Fehler, Argumente<'t, T, Fehler>>, Vec<Option<&'a OsStr>>) {
        let OptionHelper { ergebnis_anpassen: _, argumente } = *self;
        argumente.parse_rekursiv(args)
    }

    #[inline]
    fn parse_merged_short_forms(
        self: Box<Self>,
        args: Box<dyn '_ + Iterator<Item = Option<ArgumentInput>>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<ArgumentInput>>) {
        let OptionHelper { ergebnis_anpassen, argumente } = *self;
        let (ergebnis, nicht_verwendet) = argumente.parse_rekursiv_merged_short_forms(args);
        (ergebnis_anpassen(ergebnis), nicht_verwendet)
    }

    #[inline]
    fn erzeuge_hilfe_text(
        &self,
        variante: &dyn ErzeugeHilfeText,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> NonEmpty<hilfe::Alternativen> {
        self.argumente.erzeuge_hilfe_text(variante, meta_standard, meta_erlaubte_werte)
    }
}

/// Erstelle die [`Anzeige`]-closure für [`Option<T>`] als trait-Objekt.
fn erstelle_boxed_option_anzeige<'t, T>(
    anzeige: Cow<'t, dyn Anzeige<'t, T>>,
) -> Box<dyn 't + Anzeige<'t, Option<T>>> {
    Box::new(move |opt: &Option<T>| {
        #[allow(clippy::min_ident_chars)]
        if let Some(t) = opt {
            anzeige(t)
        } else {
            String::from("None")
        }
    })
}

/// Erstelle die closure für das finale anpassen des [`Ergebnis`] bei einer [`Option<T>`].
fn erstelle_ergebnis_anpassen<'t, T: Clone>(
    standard: Option<T>,
) -> impl FnOnce(Ergebnis<'t, T, String>) -> Ergebnis<'t, T, String> {
    |ergebnis| match (ergebnis, standard) {
        (Ergebnis::Fehler(fehler_liste), Some(standard)) => {
            let mut finales_ergebnis = Some(Ergebnis::Wert(standard.clone()));
            for fehler in &fehler_liste {
                if let Fehler::ParseFehler(_parse_fehler) = fehler {
                    finales_ergebnis = None;
                    break;
                }
            }
            if let Some(finales_ergebnis) = finales_ergebnis {
                finales_ergebnis
            } else {
                Ergebnis::Fehler(fehler_liste)
            }
        },
        (ergebnis, _) => ergebnis,
    }
}

/// Erstelle die [`Beschreibung`] für den Aufruf von [`ParseArgument::argumente`] bei einer [`Option<T>`].
fn erstelle_beschreibung<'t, T>(beschreibung: &Beschreibung<'t, Option<T>>) -> Beschreibung<'t, T> {
    let name_lang_präfix = beschreibung.name.lang_präfix.clone();
    let name_lang = beschreibung.name.lang.clone();
    let name_kurz_präfix = beschreibung.name.kurz_präfix.clone();
    let name_kurz = beschreibung.name.kurz.clone();
    Beschreibung::neu(
        name_lang_präfix,
        name_lang.clone(),
        name_kurz_präfix,
        name_kurz.clone(),
        None::<&str>,
        None,
    )
}

impl<T: 'static + ParseArgument + Clone + Debug + Display> ParseArgument for Option<T> {
    #[inline]
    fn argumente<'t>(
        beschreibung: Beschreibung<'t, Self>,
        invertiere_präfix: impl Into<Vergleich<'t>>,
        invertiere_infix: impl Into<Vergleich<'t>>,
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
    ) -> Argumente<'t, Self, String> {
        let wert_infix_vergleich = wert_infix.into();
        let argumente = <T as ParseArgument>::argumente(
            erstelle_beschreibung(&beschreibung),
            invertiere_präfix,
            invertiere_infix,
            wert_infix_vergleich,
            meta_var,
        );
        let ergebnis_anpassen = erstelle_ergebnis_anpassen(beschreibung.standard.clone());
        #[allow(clippy::shadow_unrelated)]
        match argumente {
            Argumente::EinzelArgument(EinzelArgument::Flag(Flag {
                beschreibung: _,
                invertiere_präfix,
                invertiere_infix,
                konvertiere,
                anzeige,
            })) => {
                let boxed_konvertiere: Box<dyn Bool<'t, Option<T>>> =
                    Box::new(move |bool| Some(konvertiere(bool)));
                Argumente::EinzelArgument(EinzelArgument::Flag(Flag {
                    beschreibung,
                    invertiere_präfix,
                    invertiere_infix,
                    konvertiere: Cow::Owned(boxed_konvertiere),
                    anzeige: Cow::Owned(erstelle_boxed_option_anzeige(anzeige)),
                }))
            },
            Argumente::EinzelArgument(EinzelArgument::FrühesBeenden {
                frühes_beenden,
                wert,
                anzeige,
            }) => {
                // standard ist garantiert [`None`], daher kann [`Beschreibung`] übernommen werden.
                Argumente::EinzelArgument(EinzelArgument::FrühesBeenden {
                    frühes_beenden,
                    wert: Some(wert),
                    anzeige: Cow::Owned(erstelle_boxed_option_anzeige(anzeige)),
                })
            },
            Argumente::EinzelArgument(EinzelArgument::Wert(Wert {
                beschreibung: _,
                wert_infix,
                meta_var,
                mögliche_werte,
                parse,
                anzeige,
                anzeige_fehler,
            })) => {
                let boxed_parse: Box<dyn dyn_to_owned::Parse<'t, Option<T>, String>> =
                    Box::new(move |os_str: &OsStr| match parse(os_str) {
                        Ok(wert) => Ok(Some(wert)),
                        Err(_fehler) if os_str == "None" => Ok(None),
                        Err(fehler) => Err(fehler),
                    });
                let mögliche_werte = mögliche_werte.map(|nonempty| {
                    let mut nonempty = nonempty.map(Some);
                    nonempty.push(None);
                    nonempty
                });
                let wert = Argumente::from(Wert {
                    beschreibung,
                    wert_infix,
                    meta_var,
                    mögliche_werte,
                    parse: Cow::Owned(boxed_parse),
                    anzeige: Cow::Owned(erstelle_boxed_option_anzeige(anzeige)),
                    anzeige_fehler,
                });
                Argumente::kombiniere(OptionHelper { ergebnis_anpassen, argumente: wert })
            },
            Argumente::Kombiniere(kombiniere) => Argumente::kombiniere(OptionHelper {
                ergebnis_anpassen,
                argumente: Argumente::kombiniere((Some, Argumente::Kombiniere(kombiniere))),
            }),
            Argumente::Alternativen(alternativen) => Argumente::kombiniere(OptionHelper {
                ergebnis_anpassen,
                argumente: Argumente::kombiniere((Some, Argumente::Alternativen(alternativen))),
            }),
        }
    }

    #[inline]
    fn standard() -> Option<Self> {
        Some(None)
    }
}

impl<T: 'static + EnumArgument + Display + Clone> ParseArgument for T {
    #[inline]
    fn argumente<'t>(
        beschreibung: Beschreibung<'t, Self>,
        _invertiere_präfix: impl Into<Vergleich<'t>>,
        _invertiere_infix: impl Into<Vergleich<'t>>,
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
    ) -> Argumente<'t, Self, String> {
        let boxed_parse: Box<dyn dyn_to_owned::Parse<'t, T, String>> =
            Box::new(move |os_str: &OsStr| {
                let Some(string) = os_str.to_str() else {
                    return Err(ParseFehler::InvaliderString(OsString::from(os_str)));
                };
                <T as EnumArgument>::varianten()
                    .into_iter()
                    .flatten()
                    .find(
                        #[allow(clippy::min_ident_chars)]
                        |t| t.to_string() == string,
                    )
                    .ok_or_else(|| ParseFehler::ParseFehler(String::from(string)))
            });
        Argumente::from(Wert {
            beschreibung,
            wert_infix: wert_infix.into(),
            meta_var,
            mögliche_werte: <T as EnumArgument>::varianten(),
            parse: Cow::Owned(boxed_parse),
            anzeige: Cow::Borrowed(&<T as ToString>::to_string),
            anzeige_fehler: Cow::Borrowed(&<String as Clone>::clone),
        })
    }

    #[inline]
    fn standard() -> Option<Self> {
        None
    }
}

/// Erlaube parsen aus Kommandozeilen-Argumenten ausgehend einer Standard-Konfiguration.
///
/// Mit aktiviertem `derive`-Feature kann diese [`automatisch erzeugt werden`](derive@Parse).
///
/// ## English
/// Allow parsing from command line arguments, based on a default configuration.
///
/// With active `derive`-feature, the implementation can be [`automatically created`](derive@Parse).
pub trait Parse: Sized {
    /// Möglicher Parse-Fehler, die automatisch erzeugte Implementierung verwendet [`String`].
    ///
    /// ## English
    /// Possible parse error, the automatically created implementation uses [`String`].
    type Fehler;

    /// Erzeuge eine Beschreibung, wie Kommandozeilen-Argumente geparst werden sollen.
    ///
    /// ## English
    /// Create a description, how command line arguments should be parsed.
    fn kommandozeilen_argumente<'t>() -> Argumente<'t, Self, Self::Fehler>;

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    ///
    /// ## English
    /// Parse the given command line arguments to create the requested type.
    #[inline]
    fn parse<'t>(
        args: impl Iterator<Item = OsString>,
    ) -> (Ergebnis<'t, Self, Self::Fehler>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Fehler: 't,
    {
        Self::kommandozeilen_argumente().parse(args)
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    ///
    /// ## English synonym
    /// [`parse_from_env`](Parse::parse_from_env)
    #[inline]
    fn parse_aus_env<'t>() -> (Ergebnis<'t, Self, Self::Fehler>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Fehler: 't,
    {
        Self::kommandozeilen_argumente().parse_aus_env()
    }

    /// Parse [`args_os`](std::env::args_os) and try to create the requested type.
    ///
    /// ## Deutsches Synonym
    /// [`parse_aus_env`](Parse::parse_aus_env)
    #[inline]
    fn parse_from_env<'t>() -> (Ergebnis<'t, Self, Self::Fehler>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Fehler: 't,
    {
        Self::parse_aus_env()
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [`parse_from_env_with_early_exit`](Parse::parse_from_env_with_early_exit)
    #[inline]
    #[allow(clippy::type_complexity)]
    fn parse_aus_env_mit_frühen_beenden<'t>(
    ) -> (Result<Self, NonEmpty<Fehler<'t, Self::Fehler>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Fehler: 't,
    {
        Self::kommandozeilen_argumente().parse_aus_env_mit_frühen_beenden()
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_aus_env_mit_frühen_beenden`](Argumente::parse_aus_env_mit_frühen_beenden)
    #[inline]
    #[allow(clippy::type_complexity)]
    fn parse_from_env_with_early_exit<'t>(
    ) -> (Result<Self, NonEmpty<Error<'t, Self::Fehler>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Fehler: 't,
    {
        Self::parse_aus_env_mit_frühen_beenden()
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    ///
    /// ## English synonym
    /// [`parse_with_early_exit`](Parse::parse_with_early_exit)
    #[inline]
    #[allow(clippy::type_complexity)]
    fn parse_mit_frühen_beenden<'t>(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<Self, NonEmpty<Fehler<'t, Self::Fehler>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Fehler: 't,
    {
        Self::kommandozeilen_argumente().parse_mit_frühen_beenden(args)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_mit_frühen_beenden`](Parse::parse_mit_frühen_beenden)
    #[inline]
    #[allow(clippy::type_complexity)]
    fn parse_with_early_exit<'t>(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<Self, NonEmpty<Error<'t, Self::Fehler>>>, Vec<ArgumentInput>)
    where
        Self: 't,
        Self::Fehler: 't,
    {
        Self::parse_mit_frühen_beenden(args)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [`parse_complete`](Parse::parse_complete)
    #[inline]
    #[must_use]
    fn parse_vollständig(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        arg_nicht_verwendet: &str,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_vollständig(
            args,
            fehler_code,
            fehlende_flag,
            fehlender_wert,
            parse_fehler,
            invalider_string,
            arg_nicht_verwendet,
        )
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig`](Parse::parse_vollständig)
    #[inline]
    #[must_use]
    fn parse_complete(
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::parse_vollständig(
            args,
            error_code,
            missing_flag,
            missing_value,
            parse_error,
            invalid_string,
            unused_arg,
        )
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [`parse_complete_with_language`](Parse::parse_complete_with_language)
    #[inline]
    #[must_use]
    fn parse_vollständig_mit_sprache(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
        sprache: Sprache,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_vollständig_mit_sprache(args, fehler_code, sprache)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig_mit_sprache`](Parse::parse_vollständig_mit_sprache)
    #[inline]
    #[must_use]
    fn parse_complete_with_language(
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
        language: Language,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::parse_vollständig_mit_sprache(args, error_code, language)
    }

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// [`parse_vollständig_mit_sprache`](Parse::parse_vollständig_mit_sprache) mit [`Sprache::DEUTSCH`].
    ///
    /// ## English version
    /// [`parse_with_error_message`](Parse::parse_with_error_message)
    #[inline]
    #[must_use]
    fn parse_mit_fehlermeldung(
        args: impl Iterator<Item = OsString>,
        fehler_code: NonZeroI32,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_mit_fehlermeldung(args, fehler_code)
    }

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// [`parse_complete_with_language`](Parse::parse_complete_with_language) with [`Language::ENGLISH`].
    ///
    /// ## Deutsche version
    /// [`parse_mit_fehlermeldung`](Parse::parse_mit_fehlermeldung)
    #[inline]
    #[must_use]
    fn parse_with_error_message(
        args: impl Iterator<Item = OsString>,
        error_code: NonZeroI32,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_with_error_message(args, error_code)
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [`parse_complete_from_env`](Parse::parse_complete_from_env)
    #[inline]
    #[must_use]
    fn parse_vollständig_aus_env(
        fehler_code: NonZeroI32,
        fehlende_flag: &str,
        fehlender_wert: &str,
        parse_fehler: &str,
        invalider_string: &str,
        arg_nicht_verwendet: &str,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_vollständig_aus_env(
            fehler_code,
            fehlende_flag,
            fehlender_wert,
            parse_fehler,
            invalider_string,
            arg_nicht_verwendet,
        )
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig_aus_env`](Parse::parse_vollständig_aus_env)
    #[inline]
    #[must_use]
    fn parse_complete_from_env(
        error_code: NonZeroI32,
        missing_flag: &str,
        missing_value: &str,
        parse_error: &str,
        invalid_string: &str,
        unused_arg: &str,
    ) -> Self
    where
        Self::Fehler: Display,
    {
        Self::parse_vollständig_aus_env(
            error_code,
            missing_flag,
            missing_value,
            parse_error,
            invalid_string,
            unused_arg,
        )
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// ## English synonym
    /// [`parse_complete_with_language_from_env`](Parse::parse_complete_with_language_from_env)
    #[inline]
    #[must_use]
    fn parse_vollständig_mit_sprache_aus_env(fehler_code: NonZeroI32, sprache: Sprache) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente()
            .parse_vollständig_mit_sprache_aus_env(fehler_code, sprache)
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_vollständig_mit_sprache_aus_env`](Parse::parse_vollständig_mit_sprache_aus_env)
    #[inline]
    #[must_use]
    fn parse_complete_with_language_from_env(error_code: NonZeroI32, language: Language) -> Self
    where
        Self::Fehler: Display,
    {
        Self::parse_vollständig_mit_sprache_aus_env(error_code, language)
    }

    /// Parse [`args_os`](std::env::args_os) und versuche den gewünschten Typ zu erzeugen.
    /// Sofern ein frühes beenden gewünscht wird (z.B. `--version`) werden die
    /// entsprechenden Nachrichten in `stdout` geschrieben und das Program über
    /// [`exit`](std::process::exit) mit exit code `0` beendet.
    /// Tritt ein Fehler auf, oder gibt es nicht-geparste Argumente werden die Fehler in `stderr`
    /// geschrieben und das Programm über [`exit`](std::process::exit) mit exit code `fehler_code` beendet.
    ///
    /// [`parse_vollständig_mit_sprache_aus_env`](Parse::parse_vollständig_mit_sprache_aus_env)
    /// mit [`Sprache::DEUTSCH`].
    ///
    /// ## English version
    /// [`parse_with_error_message_from_env`](Parse::parse_with_error_message_from_env)
    #[inline]
    #[must_use]
    fn parse_mit_fehlermeldung_aus_env(fehler_code: NonZeroI32) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_mit_fehlermeldung_aus_env(fehler_code)
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// [`parse_complete_with_language_from_env`](Parse::parse_complete_with_language_from_env)
    /// with [`Language::ENGLISH`].
    ///
    /// ## Deutsche Version
    /// [`parse_mit_fehlermeldung_aus_env`](Parse::parse_mit_fehlermeldung_aus_env)
    #[inline]
    #[must_use]
    fn parse_with_error_message_from_env(error_code: NonZeroI32) -> Self
    where
        Self::Fehler: Display,
    {
        Self::kommandozeilen_argumente().parse_with_error_message_from_env(error_code)
    }
}
