//! Kombiniere mehrere [Argumente] zu einem neuen, basierend auf einer Funktion.

use std::ffi::OsString;

use nonempty::NonEmpty;
use paste::paste;

use crate::{
    argumente::{
        hilfe::{self, ErzeugeHilfeText},
        Argumente,
    },
    ergebnis::Ergebnis,
};

/// Kombiniere mehrere Argumente mit der übergebenen Funktion.
///
/// ## English synonym
/// [`combine`]
#[macro_export]
macro_rules! kombiniere {
    ($f: expr, $arg: expr) => {
        $crate::argumente::Argumente::kombiniere(($f, $arg))
    };
    ($f: expr, $a: expr, $b: expr $(, $tail: ident)* $(,)?) => {{
        #[allow(clippy::shadow_unrelated, clippy::shadow_same, clippy::shadow_reuse)]
        {
            let kombiniere = $crate::argumente::Argumente::kombiniere((|first, second| (first, second), $a, $b));
            let uncurry_f = move |(first, second) $(, $tail)*| $f(first, second $(, $tail)*);
            $crate::kombiniere!(uncurry_f, kombiniere $(, $tail)*)
        }
    }};
}

/// Combine multiple arguments with the given function.
///
/// ## Deutsches Synonym
/// [`kombiniere`]
#[macro_export]
macro_rules! combine {
    ($f: expr, $arg: expr) => {
        $crate::kombiniere!($f, $arg)
    };
    ($f: expr, $a: expr, $b: expr $(, $tail: ident)* $(,)?) => {
        $crate::kombiniere!($f, $a, $b $(, $tail)*)
    };
}

/// Erlaube kombinieren mehrerer Argumente.
///
/// ## English
/// Allow combining multiple arguments.
pub trait Kombiniere<'t, T, Fehler> {
    /// Parse die übergebenen Argumente und erzeuge den zugehörigen Wert.
    ///
    /// ## English
    /// Parse the given arguments and return the corresponding value.
    fn parse(
        self: Box<Self>,
        args: Box<dyn '_ + Iterator<Item = Option<OsString>>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>);

    /// Erzeuge den Hilfetext für die enthaltenen [`Einzelargumente`](EinzelArgument).
    fn erzeuge_hilfe_text(
        &self,
        variante: &dyn ErzeugeHilfeText,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> NonEmpty<hilfe::Alternativen>;
}

// TODO Kurz-Namen verschmelzen
// TODO erlaube impl Into<Argumente>
/// Implementiere das [`Kombiniere`]-trait für ein Tupel (f, a0, a1, ...)
macro_rules! impl_kombiniere_tuple {
    ($($suffix: ident),+ $(,)?) => {
        paste! {
            impl <
                't, $([<'t $suffix:snake:lower>],)+ F, T, Fehler,
                $([<T $suffix:camel>],)+
            >
                Kombiniere<'t, T, Fehler> for (
                    F,
                    $(Argumente<
                        [<'t $suffix:snake:lower>],
                        [<T $suffix:camel>],
                        Fehler,
                    >),+
                )
            where
                F: Fn($([<T $suffix:camel>]),+) -> T,
                $(
                    [<'t $suffix:snake:lower>]: 't,
                )+
            {
                #[inline]
                fn parse(
                    self: Box<Self>,
                    args: Box<dyn '_ + Iterator<Item = Option<OsString>>>,
                ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>) {
                    let (funktion, $([<arg_ $suffix:snake:lower>]),+) = *self;
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
                                alle_fehler.extend(fehler_liste)
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
                fn erzeuge_hilfe_text(
                    &self,
                    variante: &dyn ErzeugeHilfeText,
                    meta_standard: &str,
                    meta_erlaubte_werte: &str,
                ) -> NonEmpty<hilfe::Alternativen> {
                    let (_f, $([<a_ $suffix:snake:lower>]),+) = self;
                    let mut hilfe_texte = Vec::new();
                    $(
                        hilfe_texte.extend(
                            [<a_ $suffix:snake:lower>]
                                .erzeuge_hilfe_text(variante, meta_standard, meta_erlaubte_werte)
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
