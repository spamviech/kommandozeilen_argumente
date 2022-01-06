//! Kombiniere mehrere [Arg] zu einem neuen, basierend auf einer Funktion.

use nonempty::NonEmpty;

use crate::{arg::Arg, ergebnis::ParseErgebnis};

#[macro_export]
/// Parse mehrere Kommandozeilen-Argumente und kombiniere die Ergebnisse mit der übergebenen Funktion.
macro_rules! kombiniere {
    ($funktion: expr => ) => {
        Arg::konstant($funktion)
    };
    ($funktion: expr => $a: ident $(,)?) => {
        Arg::konvertiere($funktion, $a)
    };
    ($funktion: expr => $a: ident, $b:ident $(,)?) => {
        Arg::kombiniere2($funktion, $a, $b)
    };
    ($funktion: expr => $a: ident, $b:ident, $($args: ident),+ $(,)?) => {{
        let tuple_arg = Arg::kombiniere2(|a,b| (a,b), $a, $b);
        let uncurry_first_two = move |(a,b), $($args),+| $funktion(a, b, $($args),+);
        $crate::kombiniere!(uncurry_first_two => tuple_arg, $($args),+)
    }};
}
pub use crate::kombiniere;

macro_rules! impl_kombiniere_n {
    ($name: ident ($($var: ident: $ty_var: ident),*)) => {
        /// Parse mehrere Kommandozeilen-Argumente und kombiniere die Ergebnisse mit der übergebenen Funktion.
        pub fn $name<$($ty_var: 'static),*>(
            f: impl 'static + Fn($($ty_var),*) -> T,
            $($var: Arg<$ty_var, Error>),*
        ) -> Arg<T, Error> {
            kombiniere!(f=>$($var),*)
        }

    };
}

impl<T, Error: 'static> Arg<T, Error> {
    /// Parse keine Kommandozeilen-Argumente und erzeuge das Ergebnis mit der übergebenen Funktion.
    pub fn konstant(f: impl 'static + Fn() -> T) -> Arg<T, Error> {
        Arg {
            beschreibungen: Vec::new(),
            flag_kurzformen: Vec::new(),
            parse: Box::new(move |args| (ParseErgebnis::Wert(f()), args)),
        }
    }

    /// Parse Kommandozeilen-Argumente und konvertiere das Ergebnis mit der übergebenen Funktion.
    pub fn konvertiere<A: 'static>(
        f: impl 'static + Fn(A) -> T,
        Arg { beschreibungen, flag_kurzformen, parse }: Arg<A, Error>,
    ) -> Arg<T, Error> {
        Arg {
            beschreibungen,
            flag_kurzformen,
            parse: Box::new(move |args| {
                let (ergebnis, nicht_verwendet) = parse(args);
                let konvertiert = match ergebnis {
                    ParseErgebnis::Wert(wert) => ParseErgebnis::Wert(f(wert)),
                    ParseErgebnis::FrühesBeenden(nachrichten) => {
                        ParseErgebnis::FrühesBeenden(nachrichten)
                    }
                    ParseErgebnis::Fehler(fehler) => ParseErgebnis::Fehler(fehler),
                };
                (konvertiert, nicht_verwendet)
            }),
        }
    }

    /// Parse mehrere Kommandozeilen-Argumente und kombiniere die Ergebnisse mit der übergebenen Funktion.
    pub fn kombiniere2<A: 'static, B: 'static>(
        f: impl 'static + Fn(A, B) -> T,
        a: Arg<A, Error>,
        b: Arg<B, Error>,
    ) -> Arg<T, Error> {
        let mut beschreibungen = a.beschreibungen;
        beschreibungen.extend(b.beschreibungen);
        let mut flag_kurzformen = a.flag_kurzformen;
        flag_kurzformen.extend(b.flag_kurzformen);
        Arg {
            beschreibungen,
            flag_kurzformen,
            parse: Box::new(move |args| {
                let mut fehler = Vec::new();
                let mut frühes_beenden = Vec::new();
                let (a_ergebnis, a_nicht_verwendet) = (a.parse)(args);
                let a = match a_ergebnis {
                    ParseErgebnis::Wert(wert) => Some(wert),
                    ParseErgebnis::FrühesBeenden(nachrichten) => {
                        frühes_beenden.extend(nachrichten);
                        None
                    }
                    ParseErgebnis::Fehler(parse_fehler) => {
                        fehler.extend(parse_fehler);
                        None
                    }
                };
                let (b_ergebnis, b_nicht_verwendet) = (b.parse)(a_nicht_verwendet);
                let b = match b_ergebnis {
                    ParseErgebnis::Wert(wert) => Some(wert),
                    ParseErgebnis::FrühesBeenden(nachrichten) => {
                        frühes_beenden.extend(nachrichten);
                        None
                    }
                    ParseErgebnis::Fehler(parse_fehler) => {
                        fehler.extend(parse_fehler);
                        None
                    }
                };
                let ergebnis = if let Some(fehler) = NonEmpty::from_vec(fehler) {
                    ParseErgebnis::Fehler(fehler)
                } else if let Some(nachrichten) = NonEmpty::from_vec(frühes_beenden) {
                    ParseErgebnis::FrühesBeenden(nachrichten)
                } else {
                    ParseErgebnis::Wert(f(a.unwrap(), b.unwrap()))
                };
                (ergebnis, b_nicht_verwendet)
            }),
        }
    }

    impl_kombiniere_n! {kombiniere3(a: A, b: B, c: C)}
    impl_kombiniere_n! {kombiniere4(a: A, b: B, c: C, d: D)}
    impl_kombiniere_n! {kombiniere5(a: A, b: B, c: C, d: D, e: E)}
    impl_kombiniere_n! {kombiniere6(a: A, b: B, c: C, d: D, e: E, f: F)}
    impl_kombiniere_n! {kombiniere7(a: A, b: B, c: C, d: D, e: E, f: F, g: G)}
    impl_kombiniere_n! {kombiniere8(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H)}
    impl_kombiniere_n! {kombiniere9(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I)}
}
