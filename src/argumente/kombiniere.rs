//! Kombiniere mehrere [Argumente] zu einem neuen, basierend auf einer Funktion.

use nonempty::NonEmpty;

use crate::{argumente::Argumente, ergebnis::Ergebnis};

#[macro_export]
/// Parse mehrere Kommandozeilen-Argumente und kombiniere die Ergebnisse mit der übergebenen Funktion.
macro_rules! kombiniere {
    ($funktion: expr => ) => {
        $crate::Argumente::konstant($funktion)
    };
    ($funktion: expr => $a: ident $(,)?) => {
        $crate::Argumente::konvertiere($funktion, $a)
    };
    ($funktion: expr => $a: ident, $b:ident $(,)?) => {
        $crate::Argumente::kombiniere2($funktion, $a, $b)
    };
    ($funktion: expr => $a: ident, $b:ident, $($args: ident),+ $(,)?) => {{
        let tuple_arg = $crate::Argumente::kombiniere2(|a,b| (a,b), $a, $b);
        let uncurry_first_two = move |(a,b), $($args),+| $funktion(a, b, $($args),+);
        $crate::kombiniere!(uncurry_first_two => tuple_arg, $($args),+)
    }};
}

macro_rules! impl_kombiniere_n {
    ($name: ident ($($var: ident: $ty_var: ident),*)) => {
        /// Parse mehrere Kommandozeilen-Argumente und kombiniere die Ergebnisse mit der übergebenen Funktion.
        pub fn $name<$($ty_var: 'static),*>(
            f: impl 'static + Fn($($ty_var),*) -> T,
            $($var: Argumente<$ty_var, Error>),*
        ) -> Argumente<T, Error> {
            kombiniere!(f=>$($var),*)
        }

    };
}

impl<T, Error: 'static> Argumente<T, Error> {
    /// Parse keine Kommandozeilen-Argumente und erzeuge das Ergebnis mit der übergebenen Funktion.
    pub fn konstant(f: impl 'static + Fn() -> T) -> Argumente<T, Error> {
        Argumente {
            beschreibungen: Vec::new(),
            flag_kurzformen: Vec::new(),
            parse: Box::new(move |args| (Ergebnis::Wert(f()), args)),
        }
    }

    /// Parse Kommandozeilen-Argumente und konvertiere das Ergebnis mit der übergebenen Funktion.
    pub fn konvertiere<A: 'static>(
        f: impl 'static + Fn(A) -> T,
        Argumente { beschreibungen, flag_kurzformen, parse }: Argumente<A, Error>,
    ) -> Argumente<T, Error> {
        Argumente {
            beschreibungen,
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

    /// Parse mehrere Kommandozeilen-Argumente und kombiniere die Ergebnisse mit der übergebenen Funktion.
    pub fn kombiniere2<A: 'static, B: 'static>(
        f: impl 'static + Fn(A, B) -> T,
        a: Argumente<A, Error>,
        b: Argumente<B, Error>,
    ) -> Argumente<T, Error> {
        let mut beschreibungen = a.beschreibungen;
        beschreibungen.extend(b.beschreibungen);
        let mut flag_kurzformen = a.flag_kurzformen;
        flag_kurzformen.extend(b.flag_kurzformen);
        Argumente {
            beschreibungen,
            flag_kurzformen,
            parse: Box::new(move |args| {
                let mut fehler = Vec::new();
                let mut frühes_beenden = Vec::new();
                let (a_ergebnis, a_nicht_verwendet) = (a.parse)(args);
                let a = match a_ergebnis {
                    Ergebnis::Wert(wert) => Some(wert),
                    Ergebnis::FrühesBeenden(nachrichten) => {
                        frühes_beenden.extend(nachrichten);
                        None
                    }
                    Ergebnis::Fehler(parse_fehler) => {
                        fehler.extend(parse_fehler);
                        None
                    }
                };
                let (b_ergebnis, b_nicht_verwendet) = (b.parse)(a_nicht_verwendet);
                let b = match b_ergebnis {
                    Ergebnis::Wert(wert) => Some(wert),
                    Ergebnis::FrühesBeenden(nachrichten) => {
                        frühes_beenden.extend(nachrichten);
                        None
                    }
                    Ergebnis::Fehler(parse_fehler) => {
                        fehler.extend(parse_fehler);
                        None
                    }
                };
                let ergebnis = if let Some(fehler) = NonEmpty::from_vec(fehler) {
                    Ergebnis::Fehler(fehler)
                } else if let Some(nachrichten) = NonEmpty::from_vec(frühes_beenden) {
                    Ergebnis::FrühesBeenden(nachrichten)
                } else {
                    Ergebnis::Wert(f(a.unwrap(), b.unwrap()))
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
