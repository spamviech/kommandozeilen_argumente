//! Trait für Typen, die aus Kommandozeilen-Argumenten geparst werden können.

use std::{
    borrow::Cow,
    collections::HashMap,
    convert::identity,
    ffi::{OsStr, OsString},
    fmt::Display,
    num::NonZeroI32,
    str::FromStr,
};

use nonempty::NonEmpty;
use void::Void;

use crate::{
    argumente::{
        einzelargument::EinzelArgument,
        flag::Flag,
        wert::{EnumArgument, Wert},
        Argumente, Arguments,
    },
    beschreibung::{Beschreibung, Description, Konfiguration},
    ergebnis::{Ergebnis, Error, Fehler, ParseFehler},
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
    ) -> EinzelArgument<'t, Self, String>;

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
    ) -> EinzelArgument<'t, Self, String> {
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
    ) -> EinzelArgument<'t, Self, String> {
        Self::argumente_mit_sprache(description, language)
    }

    /// Erstelle ein [Argumente] für die übergebene [`Beschreibung`].
    ///
    /// ## English version
    /// [`new`](ParseArgument::new)
    #[inline]
    #[allow(clippy::needless_lifetimes)]
    fn neu<'t>(beschreibung: Beschreibung<'t, Self>) -> EinzelArgument<'t, Self, String> {
        Self::argumente_mit_sprache(beschreibung, Sprache::DEUTSCH)
    }

    /// Create an [Argumente] for the [`Beschreibung`].
    ///
    /// ## Deutsche Version
    /// [`neu`](ParseArgument::neu)
    #[inline]
    #[allow(clippy::needless_lifetimes)]
    fn new<'t>(beschreibung: Beschreibung<'t, Self>) -> EinzelArgument<'t, Self, String> {
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
    ) -> EinzelArgument<'t, Self, String> {
        EinzelArgument::Flag(Flag {
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
    ) -> EinzelArgument<'t, Self, String> {
        EinzelArgument::wert(Wert {
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
            ) -> EinzelArgument<'t, Self, String> {
                EinzelArgument::wert(Wert {
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

impl<T: 'static + ParseArgument + Clone + Display> ParseArgument for Option<T> {
    #[inline]
    fn argumente<'t>(
        beschreibung: Beschreibung<'t, Self>,
        invertiere_präfix: impl Into<Vergleich<'t>>,
        invertiere_infix: impl Into<Vergleich<'t>>,
        wert_infix: impl Into<Vergleich<'t>>,
        meta_var: &'t str,
    ) -> EinzelArgument<'t, Self, String> {
        /// Typ-Synonym für Hilfs-Closure:
        /// Ersetzte [`Fehler::FehlenderWert`] durch [`Ergebnis::Wert`] mit dem Standard-Wert.
        type VerwendeStandard<'s, T> =
            Box<dyn 's + Fn(NonEmpty<Fehler<'_, String>>) -> Ergebnis<'_, Option<T>, String>>;
        let name_lang_präfix = beschreibung.name.lang_präfix.clone();
        let name_lang = beschreibung.name.lang.clone();
        let name_kurz_präfix = beschreibung.name.kurz_präfix.clone();
        let name_kurz = beschreibung.name.kurz.clone();
        let wert_infix_vergleich = wert_infix.into();
        let einzel_argument = T::argumente(
            Beschreibung::neu(
                name_lang_präfix,
                name_lang.clone(),
                name_kurz_präfix,
                name_kurz.clone(),
                None::<&str>,
                None,
            ),
            invertiere_präfix,
            invertiere_infix,
            wert_infix_vergleich.clone(),
            meta_var,
        );
        match einzel_argument {
            EinzelArgument::Flag(flag) => todo!(),
            EinzelArgument::FrühesBeenden { frühes_beenden, wert } => todo!(),
            EinzelArgument::Wert(wert) => todo!(),
        }
        let parse = todo!();
        let (beschreibung_string, option_standard) = beschreibung
            .als_string_beschreibung_allgemein(|opt| {
                #[allow(clippy::min_ident_chars)]
                if let Some(t) = opt {
                    t.to_string()
                } else {
                    "None".to_owned()
                }
            });
        let verwende_standard: VerwendeStandard<'t, T> = if let Some(standard) = option_standard {
            Box::new(move |fehler_sammlung| {
                let mut fehler_iter =
                    fehler_sammlung.into_iter().filter_map(|fehler| match fehler {
                        #[allow(clippy::shadow_unrelated)]
                        Fehler::FehlenderWert { name, wert_infix, meta_var } => {
                            let passender_lang_name = name.lang.iter().eq(name_lang.iter());
                            let passender_kurz_name = name.kurz.iter().eq(name_kurz.iter());
                            if passender_lang_name && passender_kurz_name {
                                None
                            } else {
                                Some(Fehler::FehlenderWert { name, wert_infix, meta_var })
                            }
                        },
                        Fehler::FehlendeFlag { .. } | Fehler::Fehler { .. } => Some(fehler),
                    });
                if let Some(head) = fehler_iter.next() {
                    let tail = fehler_iter.collect();
                    Ergebnis::Fehler(NonEmpty { head, tail })
                } else {
                    Ergebnis::Wert(standard.clone())
                }
            })
        } else {
            Box::new(|fehler| Ergebnis::Fehler(fehler))
        };
        // Argumente {
        //     konfigurationen: vec![Konfiguration::Wert {
        //         beschreibung: beschreibung_string,
        //         meta_var,
        //         wert_infix: wert_infix_vergleich,
        //         mögliche_werte: None,
        //     }],
        //     flag_kurzformen: HashMap::new(),
        //     parse: Box::new(move |args| {
        //         let (ergebnis, nicht_verwendet) = parse(args);
        //         let option_ergebnis = match ergebnis {
        //             Ergebnis::Wert(wert) => Ergebnis::Wert(Some(wert)),
        //             Ergebnis::FrühesBeenden(nachrichten) => Ergebnis::FrühesBeenden(nachrichten),
        //             Ergebnis::Fehler(fehler_sammlung) => verwende_standard(fehler_sammlung),
        //         };
        //         (option_ergebnis, nicht_verwendet)
        //     }),
        // }
        todo!()
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
    ) -> EinzelArgument<'t, Self, String> {
        todo!()
        // Argumente::wert_enum_display(beschreibung, wert_infix, meta_var)
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
    fn kommandozeilen_argumente<'t>() -> Argumente<'t, Self, Self::Fehler, Void>;

    /// Parse die übergebenen Kommandozeilen-Argumente und versuche den gewünschten Typ zu erzeugen.
    ///
    /// ## English
    /// Parse the given command line arguments to create the requested type.
    #[inline]
    fn parse<'t>(
        args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, Self, Self::Fehler>, Vec<Option<OsString>>)
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
    fn parse_aus_env<'t>() -> (Ergebnis<'t, Self, Self::Fehler>, Vec<OsString>)
    where
        Self: 't,
        Self::Fehler: 't,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_aus_env()
    }

    /// Parse [`args_os`](std::env::args_os) and try to create the requested type.
    ///
    /// ## Deutsches Synonym
    /// [`parse_aus_env`](Parse::parse_aus_env)
    #[inline]
    fn parse_from_env<'t>() -> (Ergebnis<'t, Self, Self::Fehler>, Vec<OsString>)
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
    fn parse_aus_env_mit_frühen_beenden<'t>(
    ) -> (Result<Self, NonEmpty<Fehler<'t, Self::Fehler>>>, Vec<OsString>)
    where
        Self: 't,
        Self::Fehler: 't,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_aus_env_mit_frühen_beenden()
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_aus_env_mit_frühen_beenden`](Argumente::parse_aus_env_mit_frühen_beenden)
    #[inline]
    fn parse_from_env_with_early_exit<'t>(
    ) -> (Result<Self, NonEmpty<Error<'t, Self::Fehler>>>, Vec<OsString>)
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
    fn parse_mit_frühen_beenden<'t>(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<Self, NonEmpty<Fehler<'t, Self::Fehler>>>, Vec<OsString>)
    where
        Self: 't,
        Self::Fehler: 't,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_mit_frühen_beenden(args)
    }

    /// Parse the given command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    ///
    /// ## Deutsches Synonym
    /// [`parse_mit_frühen_beenden`](Parse::parse_mit_frühen_beenden)
    #[inline]
    fn parse_with_early_exit<'t>(
        args: impl Iterator<Item = OsString>,
    ) -> (Result<Self, NonEmpty<Error<'t, Self::Fehler>>>, Vec<OsString>)
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
        todo!()
        // Self::kommandozeilen_argumente().parse_vollständig(
        //     args,
        //     fehler_code,
        //     fehlende_flag,
        //     fehlender_wert,
        //     parse_fehler,
        //     invalider_string,
        //     arg_nicht_verwendet,
        // )
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
        todo!()
        // Self::kommandozeilen_argumente().parse_vollständig_mit_sprache(args, fehler_code, sprache)
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
        todo!()
        // Self::kommandozeilen_argumente().parse_mit_fehlermeldung(args, fehler_code)
    }

    /// Parse command line arguments to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
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
        todo!()
        // Self::kommandozeilen_argumente().parse_with_error_message(args, error_code)
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
        todo!()
        // Self::kommandozeilen_argumente().parse_vollständig_aus_env(
        //     fehler_code,
        //     fehlende_flag,
        //     fehlender_wert,
        //     parse_fehler,
        //     invalider_string,
        //     arg_nicht_verwendet,
        // )
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
        todo!()
        // Self::kommandozeilen_argumente()
        //     .parse_vollständig_mit_sprache_aus_env(fehler_code, sprache)
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
    /// ## English version
    /// [`parse_with_error_message_from_env`](Parse::parse_with_error_message_from_env)
    #[inline]
    #[must_use]
    fn parse_mit_fehlermeldung_aus_env(fehler_code: NonZeroI32) -> Self
    where
        Self::Fehler: Display,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_mit_fehlermeldung_aus_env(fehler_code)
    }

    /// Parse [`args_os`](std::env::args_os) to create the requested type.
    /// If an early exit is desired (e.g. `--version`), the corresponding messages are written to
    /// `stdout` and the program stops via [`exit`](std::process::exit) with exit code `0`.
    /// In case of an error, or if there are leftover arguments, the error message is written to
    /// `stderr` and the program stops via [`exit`](std::process::exit) with exit code `error_code`.
    ///
    /// ## Deutsche Version
    /// [`parse_mit_fehlermeldung_aus_env`](Parse::parse_mit_fehlermeldung_aus_env)
    #[inline]
    #[must_use]
    fn parse_with_error_message_from_env(error_code: NonZeroI32) -> Self
    where
        Self::Fehler: Display,
    {
        todo!()
        // Self::kommandozeilen_argumente().parse_with_error_message_from_env(error_code)
    }
}
