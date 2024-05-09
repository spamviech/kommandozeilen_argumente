//! Kombiniere mehrere [Argumente] zu einem neuen, basierend auf einer Funktion.

use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
};

use nonempty::NonEmpty;
use paste::paste;
use void::Void;

use crate::{
    argumente::{
        hilfe::{self, ErzeugeHilfeText},
        new, Argumente,
    },
    ergebnis::{Ergebnis, ParseFehler},
};

#[macro_export]
/// Parse mehrere Kommandozeilen-Argumente und kombiniere die Ergebnisse mit der übergebenen Funktion.
///
/// ## English synonym
/// [combine][`crate::argumente::combine`]
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
/// [kombiniere][`macro@crate::argumente::kombiniere`]
macro_rules! combine {
    ($funktion: expr $(, $($args:ident),*)?) => {
        $crate::kombiniere!($funktion $(, $($args),*)?)
    };
    ($funktion: expr => $($args:ident),*) => {
        $crate::kombiniere!($funktion, $($args),*)
    };
}

/// Erzeuge die `kombiniere_n`- und `combine_n`-Methoden, die mehrere Argumente kombiniert.
macro_rules! impl_kombiniere_n {
    ($deutsch: ident - $english: ident ($($var: ident: $ty_var: ident),+)) => {
        /// Parse mehrere Kommandozeilen-Argumente und kombiniere die Ergebnisse mit der übergebenen Funktion.
        ///
        /// ## English synonym
        #[doc = concat!("[", stringify!($english), "](Argumente::", stringify!($english), ")")]
        #[allow(clippy::too_many_arguments, clippy::min_ident_chars)]
        #[inline]
        pub fn $deutsch<$($ty_var: 't),+>(
            funktion: impl 't + Fn($($ty_var),+) -> T,
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
                        Ergebnis::Wert(funktion($($var.expect("Kein Wert ohne Fehler.")),+))
                    };
                    (ergebnis, nicht_verwendet)
                }),
            }
        }


        /// Parse multiple command line arguments and combine the results with the given function.
        ///
        /// ## Deutsches Synonym
        #[doc = concat!("[", stringify!($deutsch), "](Argumente::", stringify!($deutsch), ")")]
        #[allow(clippy::too_many_arguments, clippy::min_ident_chars)]
        #[inline]
        pub fn $english<$($ty_var: 't),+>(
            function: impl 't + Fn($($ty_var),+) -> T,
            $($var: Argumente<'t, $ty_var, Error>),+
        ) -> Argumente<'t, T, Error> {
            Argumente::$deutsch(function, $($var),+)
        }
    };
}

impl<'t, T, Error: 't> Argumente<'t, T, Error> {
    /// Parse keine Kommandozeilen-Argumente und erzeuge das Ergebnis mit der übergebenen Funktion.
    ///
    /// ## English synonym
    /// [`constant`](Argumente::constant)
    #[inline]
    pub fn konstant(funktion: impl 't + Fn() -> T) -> Argumente<'t, T, Error> {
        Argumente {
            konfigurationen: Vec::new(),
            flag_kurzformen: HashMap::new(),
            parse: Box::new(move |args| (Ergebnis::Wert(funktion()), args)),
        }
    }

    /// Parse no command line arguments and create the result with the given function.
    ///
    /// ## Deutsches Synonym
    /// [`konstant`](Argumente::konstant)
    #[inline]
    pub fn constant(funktion: impl 't + Fn() -> T) -> Argumente<'t, T, Error> {
        Argumente::konstant(funktion)
    }

    /// Parse ein Kommandozeilen-Argument und konvertiere das Ergebnis mit der übergebenen Funktion.
    ///
    /// ## English synonym
    /// [`convert`](Argumente::convert)
    #[inline]
    pub fn konvertiere<A: 't>(
        mapper: impl 't + Fn(A) -> T,
        Argumente { konfigurationen, flag_kurzformen, parse }: Argumente<'t, A, Error>,
    ) -> Argumente<'t, T, Error> {
        Argumente {
            konfigurationen,
            flag_kurzformen,
            parse: Box::new(move |args| {
                let (ergebnis, nicht_verwendet) = parse(args);
                (ergebnis.konvertiere(&mapper), nicht_verwendet)
            }),
        }
    }

    /// Parse one command line argument and convert the result with the given function.
    ///
    /// ## Deutsches Synonym
    /// [`konvertiere`](Argumente::konvertiere)
    #[inline]
    pub fn convert<A: 't>(
        mapper: impl 't + Fn(A) -> T,
        arg: Argumente<'t, A, Error>,
    ) -> Argumente<'t, T, Error> {
        Argumente::konvertiere(mapper, arg)
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

/// Erlaube kombinieren mehrerer Argumente.
///
/// ## English
/// Allow combining multiple arguments.
pub trait Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige> {
    /// Parse die übergebenen Argumente und erzeuge den zugehörigen Wert.
    ///
    /// ## English
    /// Parse the given arguments and return the corresponding value.
    fn parse(
        self,
        args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>);

    /// Erzeuge den Hilfetext für die enthaltenen [`Einzelargumente`](EinzelArgument).
    fn erzeuge_hilfe_text<H: ErzeugeHilfeText>(
        &self,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> NonEmpty<hilfe::Alternativen<'_>>;
}

impl<'t, T, Bool, Parse, Fehler, Anzeige> Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige> for Void {
    #[inline]
    fn parse(
        self,
        _args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>) {
        void::unreachable(self)
    }

    #[inline]
    fn erzeuge_hilfe_text<H: ErzeugeHilfeText>(
        &self,
        _meta_standard: &str,
        _meta_erlaubte_werte: &str,
    ) -> NonEmpty<hilfe::Alternativen<'_>> {
        void::unreachable(*self)
    }
}

/// Implementiere das [`Kombiniere`]-trait für ein Tupel (f, a0, a1, ...)
macro_rules! impl_kombiniere_tuple {
    ($($suffix: ident),+ $(,)?) => {
        paste! {
            impl <
                't, $([<'t $suffix:snake:lower>],)+ F, T, Bool, Parse, Fehler, Anzeige,
                $(
                    [<T $suffix:camel>],
                    [<Bool $suffix:camel>],
                    [<Parse $suffix:camel>],
                    [<Anzeige $suffix:camel>],
                    [<Fehler $suffix:camel>],
                    [<Kombiniere $suffix:camel>],
                )+
            >
                Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige> for (
                    F,
                    $(new::Argumente<
                        [<'t $suffix:snake:lower>],
                        [<T $suffix:camel>],
                        [<Bool $suffix:camel>],
                        [<Parse $suffix:camel>],
                        [<Anzeige $suffix:camel>],
                        [<Kombiniere $suffix:camel>],
                    >),+
                )
            where
                F: Fn($([<T $suffix:camel>]),+) -> T,
                $(
                    [<'t $suffix:snake:lower>]: 't,
                    [<Bool $suffix:camel>]: Fn(bool) -> [<T $suffix:camel>],
                    [<Parse $suffix:camel>]:
                        Fn(&OsStr) -> Result<[<T $suffix:camel>], ParseFehler<[<Fehler $suffix:camel>]>>,
                    [<Anzeige $suffix:camel>]: Fn(&[<T $suffix:camel>]) -> String,
                    Fehler: From<[<Fehler $suffix:camel>]>,
                    [<Kombiniere $suffix:camel>]:
                        Kombiniere<
                            [<'t $suffix:snake:lower>],
                            [<T $suffix:camel>],
                            [<Bool $suffix:camel>],
                            [<Parse $suffix:camel>],
                            [<Fehler $suffix:camel>],
                            [<Anzeige $suffix:camel>],
                        >
                ),+
            {
                #[inline]
                fn parse(
                    self,
                    args: impl Iterator<Item = Option<OsString>>,
                ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>) {
                    let (funktion, $([<arg_ $suffix:snake:lower>]),+) = self;
                    let nicht_verwendet: Vec<_> = args.collect();
                    let mut alle_fehler = Vec::new();
                    let mut alle_frühes_beenden = Vec::new();
                    $(
                        let (ergebnis, nicht_verwendet)
                            = [<arg_ $suffix:snake:lower>].parse(nicht_verwendet.into_iter());
                        let mut [<wert_ $suffix:snake:lower>] = None;
                        match ergebnis {
                            Ergebnis::Wert(wert) => [<wert_ $suffix:snake:lower>] = Some(wert),
                            Ergebnis::FrühesBeenden(nachrichten) => alle_frühes_beenden.extend(nachrichten),
                            Ergebnis::Fehler(fehler_liste) => {
                                alle_fehler.extend(
                                    fehler_liste
                                        .into_iter()
                                        .map(|fehler| fehler.konvertiere(Fehler::from))
                                )
                            },
                        }
                    )+
                    let ergebnis = if let Some(fehler) = NonEmpty::from_vec(alle_fehler) {
                        Ergebnis::Fehler(fehler)
                    } else if let Some(nachrichten) = NonEmpty::from_vec(alle_frühes_beenden) {
                        Ergebnis::FrühesBeenden(nachrichten)
                    } else {
                        Ergebnis::Wert(funktion(
                            $([<wert_ $suffix:snake:lower>].expect("Kein Fehler oder FrühesBeenden!")
                        ),+))
                    };
                    (ergebnis, nicht_verwendet)
                }

                #[inline]
                fn erzeuge_hilfe_text<H: ErzeugeHilfeText>(
                    &self,
                    meta_standard: &str,
                    meta_erlaubte_werte: &str,
                ) -> NonEmpty<hilfe::Alternativen<'_>> {
                    let (_f, $([<a_ $suffix:snake:lower>]),+) = self;
                    let mut hilfe_texte = Vec::new();
                    $(
                        hilfe_texte.extend(
                            [<a_ $suffix:snake:lower>]
                                .erzeuge_hilfe_text::<H, [<Fehler $suffix:camel>]>(meta_standard, meta_erlaubte_werte)
                        );
                    )+
                    NonEmpty::from_vec(hilfe_texte).expect("Mindestens ein suffix als Macro-Argument!")
                }
            }
        }
    };
}

impl_kombiniere_tuple!(A);
impl_kombiniere_tuple!(A, B);
impl_kombiniere_tuple!(A, B, C);
impl_kombiniere_tuple!(A, B, C, D);
impl_kombiniere_tuple!(A, B, C, D, E);
impl_kombiniere_tuple!(A, B, C, D, E, F);
impl_kombiniere_tuple!(A, B, C, D, E, F, G);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
#[rustfmt::skip]
impl_kombiniere_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
