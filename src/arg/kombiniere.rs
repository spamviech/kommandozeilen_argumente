//! Kombiniere mehrere [Arg] zu einem neuen, basierend auf einer Funktion.

use nonempty::NonEmpty;

use crate::{arg::Arg, ergebnis::ParseErgebnis};

#[macro_export]
macro_rules! kombiniere {
    ($funktion: expr => $($args: ident),*) => {{
        #[allow(unused_mut)]
        let mut beschreibungen = Vec::new();
        $(beschreibungen.extend($args.beschreibungen);)*
        #[allow(unused_mut)]
        let mut flag_kurzformen = Vec::new();
        $(flag_kurzformen.extend($args.flag_kurzformen);)*
        Arg {
            beschreibungen,
            flag_kurzformen,
            parse: Box::new(move |args| {
                #[allow(unused_mut)]
                let mut fehler = Vec::new();
                #[allow(unused_mut)]
                let mut frühes_beenden = Vec::new();
                $(
                    let (ergebnis, args) = ($args.parse)(args);
                    let $args = match ergebnis {
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
                )*
                if let Some(fehler) = NonEmpty::from_vec(fehler) {
                    (ParseErgebnis::Fehler(fehler), args)
                } else if let Some(nachrichten) = NonEmpty::from_vec(frühes_beenden) {
                    (ParseErgebnis::FrühesBeenden(nachrichten), args)
                } else {
                    (ParseErgebnis::Wert($funktion($($args.unwrap()),*)), args)
                }
            }),
        }
    }};
}
pub use crate::kombiniere;

macro_rules! impl_kombiniere_n {
    ($name: ident ($($var: ident: $ty_var: ident),*)) => {
        pub fn $name<$($ty_var: 'static),*>(
            f: impl 'static + Fn($($ty_var),*) -> T,
            $($var: Arg<$ty_var, Error>),*
        ) -> Arg<T, Error> {
            kombiniere!(f=>$($var),*)
        }

    };
}

impl<T, Error: 'static> Arg<T, Error> {
    impl_kombiniere_n! {konstant()}
    impl_kombiniere_n! {konvertiere(a: A)}
    impl_kombiniere_n! {kombiniere2(a: A, b: B)}
    impl_kombiniere_n! {kombiniere3(a: A, b: B, c: C)}
    impl_kombiniere_n! {kombiniere4(a: A, b: B, c: C, d: D)}
    impl_kombiniere_n! {kombiniere5(a: A, b: B, c: C, d: D, e: E)}
    impl_kombiniere_n! {kombiniere6(a: A, b: B, c: C, d: D, e: E, f: F)}
    impl_kombiniere_n! {kombiniere7(a: A, b: B, c: C, d: D, e: E, f: F, g: G)}
    impl_kombiniere_n! {kombiniere8(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H)}
    impl_kombiniere_n! {kombiniere9(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H, i: I)}
}