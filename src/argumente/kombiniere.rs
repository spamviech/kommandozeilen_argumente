//! Kombiniere mehrere [Argumente] zu einem neuen, basierend auf einer Funktion.

use std::{
    borrow::Cow,
    collections::HashMap,
    ffi::{OsStr, OsString},
};

use nonempty::NonEmpty;
use void::Void;

use crate::{
    argumente::{einzelargument::EinzelArgument, test::ArgTest, Argumente},
    ergebnis::{Ergebnis, ParseFehler},
};

#[macro_export]
/// Parse mehrere Kommandozeilen-Argumente und kombiniere die Ergebnisse mit der übergebenen Funktion.
///
/// ## English synonym
/// [combine][crate::argumente::combine]
macro_rules! kombiniere {
    ($funktion:expr $(,)?) => {
        $crate::Argumente::konstant($funktion)
    };
    ($funktion:expr, $a:ident $(,)?) => {
        $crate::Argumente::konvertiere($funktion, $a)
    };
    ($funktion:expr, $a:ident, $b:ident $(,)?) => {
        $crate::Argumente::kombiniere2($funktion, $a, $b)
    };
    ($funktion:expr, $a:ident, $b:ident, $c:ident $(,)?) => {
        $crate::Argumente::kombiniere3($funktion, $a, $b, $c)
    };
    ($funktion:expr, $a:ident, $b:ident, $c:ident, $d:ident $(,)?) => {
        $crate::Argumente::kombiniere4($funktion, $a, $b, $c, $d)
    };
    ($funktion:expr, $a:ident, $b:ident, $c:ident, $d:ident, $e:ident $(,)?) => {
        $crate::Argumente::kombiniere5($funktion, $a, $b, $c, $d, $e)
    };
    ($funktion:expr, $a:ident, $b:ident, $c:ident, $d:ident, $e:ident, $f:ident $(,)?) => {
        $crate::Argumente::kombiniere6($funktion, $a, $b, $c, $d, $e, $f)
    };
    ($funktion:expr, $a:ident, $b:ident, $c:ident, $d:ident, $e:ident, $f:ident, $g:ident $(,)?) => {
        $crate::Argumente::kombiniere7($funktion, $a, $b, $c, $d, $e, $f, $g)
    };
    (
        $funktion:expr,
        $a:ident,
        $b:ident,
        $c:ident,
        $d:ident,
        $e:ident,
        $f:ident,
        $g:ident,
        $h:ident $(,)?
    ) => {
        $crate::Argumente::kombiniere8($funktion, $a, $b, $c, $d, $e, $f, $g, $h)
    };
    (
        $funktion:expr,
        $a:ident,
        $b:ident,
        $c:ident,
        $d:ident,
        $e:ident,
        $f:ident,
        $g:ident,
        $h:ident,
        $i:ident $(,)?
    ) => {
        $crate::Argumente::kombiniere9($funktion, $a, $b, $c, $d, $e, $f, $g, $h, $i)
    };
    (
        $funktion:expr,
        $a:ident,
        $b:ident,
        $c:ident,
        $d:ident,
        $e:ident,
        $f:ident,
        $g:ident,
        $h:ident,
        $i:ident,
        $($args: ident),+
    ) => {{
        let tuple_arg = $crate::Argumente::kombiniere9(
            |a,b,c,d,e,f,g,h,i| (a,b,c,d,e,f,g,h,i),
            $a,
            $b,
            $c,
            $d,
            $e,
            $f,
            $g,
            $h,
            $i
        );
        let uncurry_first_nine =
            move |(a,b,c,d,e,f,g,h,i), $($args),+| $funktion(a,b,c,d,e,f,g,h,i, $($args),+);
        $crate::kombiniere!(uncurry_first_nine, tuple_arg, $($args),+)
    }};
    ($funktion:expr => $($args:ident),*) => {
        $crate::kombiniere!($funktion, $($args),*)
    };
}

#[macro_export]
/// Parse multiple command line arguments and combine the results with the given function.
///
/// ## Deutsches Synonym
/// [kombiniere][macro@crate::argumente::kombiniere]
macro_rules! combine {
    ($funktion: expr $(, $($args:ident),*)?) => {
        $crate::kombiniere!($funktion $(, $($args),*)?)
    };
    ($funktion: expr => $($args:ident),*) => {
        $crate::kombiniere!($funktion, $($args),*)
    };
}

macro_rules! impl_kombiniere_n {
    ($deutsch: ident - $english: ident ($($var: ident: $ty_var: ident),+)) => {
        /// Parse mehrere Kommandozeilen-Argumente und kombiniere die Ergebnisse mit der übergebenen Funktion.
        ///
        /// ## English synonym
        #[doc = concat!("[", stringify!($english), "](Argumente::", stringify!($english), ")")]
        pub fn $deutsch<$($ty_var: 't),+>(
            f: impl 't + Fn($($ty_var),+) -> T,
            $($var: Argumente<'t, $ty_var, Error>),+
        ) -> Argumente<'t, T, Error> {
            let mut konfigurationen = Vec :: new();
            $(konfigurationen.extend($var.konfigurationen);)+
            let mut flag_kurzformen = HashMap::new();
            $(
                for (präfix, kurz_namen) in $var.flag_kurzformen {
                    flag_kurzformen.entry(präfix).or_insert(Vec::new()).extend(kurz_namen);
                }
            )+
            Argumente {
                konfigurationen,
                flag_kurzformen,
                parse: Box::new(move |args| {
                    let mut fehler = Vec::new();
                    let mut frühes_beenden = Vec::new();
                    let nicht_verwendet = args;
                    $(
                        let (ergebnis, nicht_verwendet) = ($var.parse)(nicht_verwendet);
                        let $var = match ergebnis {
                            Ergebnis::Wert(wert) => Some(wert),
                            Ergebnis::FrühesBeenden(nachrichten) => {
                                frühes_beenden.extend(nachrichten);
                                None
                            },
                            Ergebnis::Fehler(parse_fehler) => {
                                fehler.extend(parse_fehler);
                                None
                            },
                        };
                    )+
                    let ergebnis = if let Some(fehler) = NonEmpty::from_vec(fehler) {
                        Ergebnis::Fehler(fehler)
                    } else if let Some(nachrichten) = NonEmpty::from_vec(frühes_beenden) {
                        Ergebnis::FrühesBeenden(nachrichten)
                    } else {
                        // Werte werden nur auf None gesetzt, wenn ein Element zu
                        // `fehler` oder `frühes_beenden` hinzugefügt wird,
                        // diese demnach nicht-leer sind.
                        // In dieser Verzweigung sind beide leer, es sind also alle Werte Some
                        Ergebnis::Wert(f($($var.expect("Kein Wert ohne Fehler.")),+))
                    };
                    (ergebnis, nicht_verwendet)
                }),
            }
        }


        /// Parse multiple command line arguments and combine the results with the given function.
        ///
        /// ## Deutsches Synonym
        #[doc = concat!("[", stringify!($deutsch), "](Argumente::", stringify!($deutsch), ")")]
        #[inline(always)]
        pub fn $english<$($ty_var: 't),+>(
            f: impl 't + Fn($($ty_var),+) -> T,
            $($var: Argumente<'t, $ty_var, Error>),+
        ) -> Argumente<'t, T, Error> {
            Argumente::$deutsch(f, $($var),+)
        }
    };
}

impl<'t, T, Error: 't> Argumente<'t, T, Error> {
    /// Parse keine Kommandozeilen-Argumente und erzeuge das Ergebnis mit der übergebenen Funktion.
    ///
    /// ## English synonym
    /// [constant](Argumente::constant)
    pub fn konstant(f: impl 't + Fn() -> T) -> Argumente<'t, T, Error> {
        Argumente {
            konfigurationen: Vec::new(),
            flag_kurzformen: HashMap::new(),
            parse: Box::new(move |args| (Ergebnis::Wert(f()), args)),
        }
    }

    /// Parse no command line arguments and create the result with the given function.
    ///
    /// ## Deutsches Synonym
    /// [konstant](Argumente::konstant)
    #[inline(always)]
    pub fn constant(f: impl 't + Fn() -> T) -> Argumente<'t, T, Error> {
        Argumente::konstant(f)
    }

    /// Parse ein Kommandozeilen-Argument und konvertiere das Ergebnis mit der übergebenen Funktion.
    ///
    /// ## English synonym
    /// [convert](Argumente::convert)
    pub fn konvertiere<A: 't>(
        f: impl 't + Fn(A) -> T,
        Argumente { konfigurationen, flag_kurzformen, parse }: Argumente<'t, A, Error>,
    ) -> Argumente<'t, T, Error> {
        Argumente {
            konfigurationen,
            flag_kurzformen,
            parse: Box::new(move |args| {
                let (ergebnis, nicht_verwendet) = parse(args);
                (ergebnis.konvertiere(&f), nicht_verwendet)
            }),
        }
    }

    /// Parse one command line argument and convert the result with the given function.
    ///
    /// ## Deutsches Synonym
    /// [konvertiere](Argumente::konvertiere)
    #[inline(always)]
    pub fn convert<A: 't>(
        f: impl 't + Fn(A) -> T,
        arg: Argumente<'t, A, Error>,
    ) -> Argumente<'t, T, Error> {
        Argumente::konvertiere(f, arg)
    }

    impl_kombiniere_n! {kombiniere2-combine2(a: A, b: B)}
    impl_kombiniere_n! {kombiniere3-combine3(a: A, b: B, c: C)}
    impl_kombiniere_n! {kombiniere4-combine4(a: A, b: B, c: C, d: D)}
    impl_kombiniere_n! {kombiniere5-combine5(a: A, b: B, c: C, d: D, e: E)}
    impl_kombiniere_n! {kombiniere6-combine6(a: A, b: B, c: C, d: D, e: E, f: F)}
    impl_kombiniere_n! {kombiniere7-combine7(a: A, b: B, c: C, d: D, e: E, f: F, g: G)}
    impl_kombiniere_n! {kombiniere8-combine8(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H)}
    impl_kombiniere_n! {kombiniere9-combine9(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I)}
}

// TODO standard-implementierung, basierend auf Sprache?
/// Trait zum simulieren einer Rank-2 Funktion.
///
/// # English
/// Trait to simulate a rank-2 function.
pub trait HilfeText {
    fn erzeuge_hilfe_text<'t, S, Bool, Parse, Anzeige>(
        arg: &'t EinzelArgument<'t, S, Bool, Parse, Anzeige>,
        meta_standard: &'t str,
        meta_erlaubte_werte: &'t str,
    ) -> (String, Option<Cow<'t, str>>)
    where
        Anzeige: Fn(&S) -> String;
}

#[derive(Debug, Clone, Copy)]
pub struct Standard;

impl HilfeText for Standard {
    #[inline(always)]
    fn erzeuge_hilfe_text<'t, S, Bool, Parse, Anzeige>(
        arg: &'t EinzelArgument<'t, S, Bool, Parse, Anzeige>,
        meta_standard: &'t str,
        meta_erlaubte_werte: &'t str,
    ) -> (String, Option<Cow<'t, str>>)
    where
        Anzeige: Fn(&S) -> String,
    {
        arg.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
    }
}

/// Erlaube kombinieren mehrerer Argumente.
///
/// ## English
/// Allow combining multiple arguments.
pub trait Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige> {
    fn parse(
        self,
        args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>);

    /// Erzeuge den Hilfetext für die enthaltenen [Einzelargumente](EinzelArgument).
    fn erzeuge_hilfe_text<H: HilfeText>(
        &self,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> Vec<(String, Option<Cow<'_, str>>)>;
}

impl<'t, T, Bool, Parse, Fehler, Anzeige> Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige> for Void {
    fn parse(
        self,
        _args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>) {
        void::unreachable(self)
    }

    fn erzeuge_hilfe_text<H: HilfeText>(
        &self,
        _meta_standard: &str,
        _meta_erlaubte_werte: &str,
    ) -> Vec<(String, Option<Cow<'_, str>>)> {
        void::unreachable(*self)
    }
}

impl<'t, F, T0, T1, B0, B1, P0, P1, Fehler, A0, A1, K0> Kombiniere<'t, T1, B1, P1, Fehler, A1>
    for (F, ArgTest<'t, T0, B0, P0, A0, K0>)
where
    F: Fn(T0) -> T1,
    B0: Fn(bool) -> T0,
    A0: Fn(&T0) -> String,
    P0: Fn(&OsStr) -> Result<T0, ParseFehler<Fehler>>,
    K0: Kombiniere<'t, T0, B0, P0, Fehler, A0>,
{
    fn parse(
        self,
        args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, T1, Fehler>, Vec<Option<OsString>>) {
        let (f, argument) = self;
        let (ergebnis, nicht_verwendet) = argument.parse(args);
        (ergebnis.konvertiere(f), nicht_verwendet)
    }

    fn erzeuge_hilfe_text<H: HilfeText>(
        &self,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> Vec<(String, Option<Cow<'_, str>>)> {
        self.1.erzeuge_hilfe_text::<H, Fehler>(meta_standard, meta_erlaubte_werte)
    }
}

impl<'t, 't0, 't1, K, T, B, P, F, A, T0, B0, P0, F0, A0, K0, T1, B1, P1, F1, A1, K1>
    Kombiniere<'t, T, B, P, F, A>
    for (K, ArgTest<'t0, T0, B0, P0, A0, K0>, ArgTest<'t1, T1, B1, P1, A1, K1>)
where
    't0: 't,
    't1: 't,
    K: Fn(T0, T1) -> T,
    B0: Fn(bool) -> T0,
    P0: Fn(&OsStr) -> Result<T0, ParseFehler<F0>>,
    F0: Into<F>,
    A0: Fn(&T0) -> String,
    K0: Kombiniere<'t0, T0, B0, P0, F0, A0>,
    B1: Fn(bool) -> T1,
    P1: Fn(&OsStr) -> Result<T1, ParseFehler<F1>>,
    F1: Into<F>,
    A1: Fn(&T1) -> String,
    K1: Kombiniere<'t1, T1, B1, P1, F1, A1>,
{
    fn parse(
        self,
        args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, T, F>, Vec<Option<OsString>>) {
        use Ergebnis::*;

        let (f, a0, a1) = self;
        let (e0, nicht_verwendet0) = a0.parse(args);
        let (e1, nicht_verwendet1) = a1.parse(nicht_verwendet0.into_iter());
        let ergebnis = match (e0, e1) {
            (Wert(w0), Wert(w1)) => Wert(f(w0, w1)),
            (Wert(_w0), FrühesBeenden(n1)) => FrühesBeenden(n1),
            (Wert(_w0), Fehler(f1)) => Fehler(f1.map(|fehler| fehler.konvertiere(F1::into))),
            (FrühesBeenden(n0), Wert(_w1)) => FrühesBeenden(n0),
            (FrühesBeenden(n0), FrühesBeenden(_n1)) => FrühesBeenden(n0),
            (FrühesBeenden(_n0), Fehler(f1)) => {
                Fehler(f1.map(|fehler| fehler.konvertiere(F1::into)))
            },
            (Fehler(f0), Wert(_w1)) => Fehler(f0.map(|fehler| fehler.konvertiere(F0::into))),
            (Fehler(f0), FrühesBeenden(_n1)) => {
                Fehler(f0.map(|fehler| fehler.konvertiere(F0::into)))
            },
            (Fehler(f0), Fehler(f1)) => {
                let mut f = f0.map(|fehler| fehler.konvertiere(F0::into));
                f.extend(f1.into_iter().map(|fehler| fehler.konvertiere(F1::into)));
                Fehler(f)
            },
        };
        (ergebnis, nicht_verwendet1)
    }

    fn erzeuge_hilfe_text<H: HilfeText>(
        &self,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> Vec<(String, Option<Cow<'_, str>>)> {
        let (_f, a0, a1) = self;
        let mut hilfe_texte = a0.erzeuge_hilfe_text::<H, F0>(meta_standard, meta_erlaubte_werte);
        hilfe_texte.extend(a1.erzeuge_hilfe_text::<H, F1>(meta_standard, meta_erlaubte_werte));
        hilfe_texte
    }
}
