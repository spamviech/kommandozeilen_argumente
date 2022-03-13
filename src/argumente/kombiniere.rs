//! Kombiniere mehrere [Argumente] zu einem neuen, basierend auf einer Funktion.

use nonempty::NonEmpty;

use crate::{argumente::Argumente, ergebnis::Ergebnis};

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
        pub fn $deutsch<$($ty_var: 'static),+>(
            f: impl 'static + Fn($($ty_var),+) -> T,
            $($var: Argumente<$ty_var, Error>),+
        ) -> Argumente<T, Error> {
            let mut konfigurationen = Vec :: new();
            $(konfigurationen.extend($var.konfigurationen);)+
            let mut flag_kurzformen = Vec::new();
            $(flag_kurzformen.extend($var.flag_kurzformen);)+
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
                        Ergebnis::Wert(f($($var.unwrap()),+))
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
        pub fn $english<$($ty_var: 'static),+>(
            f: impl 'static + Fn($($ty_var),+) -> T,
            $($var: Argumente<$ty_var, Error>),+
        ) -> Argumente<T, Error> {
            Argumente::$deutsch(f, $($var),+)
        }
    };
}

impl<T, Error: 'static> Argumente<T, Error> {
    /// Parse keine Kommandozeilen-Argumente und erzeuge das Ergebnis mit der übergebenen Funktion.
    ///
    /// ## English synonym
    /// [constant](Argumente::constant)
    pub fn konstant(f: impl 'static + Fn() -> T) -> Argumente<T, Error> {
        Argumente {
            konfigurationen: Vec::new(),
            flag_kurzformen: Vec::new(),
            parse: Box::new(move |args| (Ergebnis::Wert(f()), args)),
        }
    }

    /// Parse no command line arguments and create the result with the given function.
    ///
    /// ## Deutsches Synonym
    /// [konstant](Argumente::konstant)
    #[inline(always)]
    pub fn constant(f: impl 'static + Fn() -> T) -> Argumente<T, Error> {
        Argumente::konstant(f)
    }

    /// Parse ein Kommandozeilen-Argument und konvertiere das Ergebnis mit der übergebenen Funktion.
    ///
    /// ## English synonym
    /// [convert](Argumente::convert)
    pub fn konvertiere<A: 'static>(
        f: impl 'static + Fn(A) -> T,
        Argumente { konfigurationen, flag_kurzformen, parse }: Argumente<A, Error>,
    ) -> Argumente<T, Error> {
        Argumente {
            konfigurationen,
            flag_kurzformen,
            parse: Box::new(move |args| {
                let (ergebnis, nicht_verwendet) = parse(args);
                let konvertiert = match ergebnis {
                    Ergebnis::Wert(wert) => Ergebnis::Wert(f(wert)),
                    Ergebnis::FrühesBeenden(nachrichten) => Ergebnis::FrühesBeenden(nachrichten),
                    Ergebnis::Fehler(fehler) => Ergebnis::Fehler(fehler),
                };
                (konvertiert, nicht_verwendet)
            }),
        }
    }

    /// Parse one command line argument and convert the result with the given function.
    ///
    /// ## Deutsches Synonym
    /// [konvertiere](Argumente::konvertiere)
    #[inline(always)]
    pub fn convert<A: 'static>(
        f: impl 'static + Fn(A) -> T,
        arg: Argumente<A, Error>,
    ) -> Argumente<T, Error> {
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
