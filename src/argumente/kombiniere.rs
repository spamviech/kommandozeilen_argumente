//! Kombiniere mehrere [Argumente] zu einem neuen, basierend auf einer Funktion.

use std::fmt::{self, Debug, Formatter};

use nonempty::{nonempty, NonEmpty};
use paste::paste;

use crate::{
    argumente::{
        hilfe::{self, ErzeugeHilfeText},
        Argumente,
    },
    beschreibung::ArgumentInput,
    ergebnis::Ergebnis,
};

/// Kombiniere mehrere Argumente mit der übergebenen Funktion.
///
/// ## English synonym
/// [`combine`]
#[macro_export]
macro_rules! kombiniere {
    ($funktion: expr, $a: expr, $b: expr, $c: expr, $d: expr, $e: expr, $f: expr, $g: expr $(, $tail: ident)+ $(,)?) => {
        #[allow(clippy::shadow_unrelated, clippy::shadow_same, clippy::shadow_reuse)]
        {
            let kombiniere =$crate::argumente::Argumente::kombiniere((
                |a, b, c, d, e, f, g| (a, b, c, d, e, f, g),
                $crate::argumente::Argumente::from($a),
                $crate::argumente::Argumente::from($b),
                $crate::argumente::Argumente::from($c),
                $crate::argumente::Argumente::from($d),
                $crate::argumente::Argumente::from($e),
                $crate::argumente::Argumente::from($f),
                $crate::argumente::Argumente::from($g),
            ));
            let uncurry_f = move |(a, b, c, d, e, f, g) $(, $tail)+| $funktion(a, b, c, d, e, f, g $(, $tail)+);
            $crate::kombiniere!(uncurry_f, kombiniere $(, $tail)+)
        }
    };
    ($funktion: expr $(, $tail: expr)* $(,)?) => {
        $crate::argumente::Argumente::kombiniere(($funktion $(, $crate::argumente::Argumente::from($tail))*))
    };
}

/// Combine multiple arguments with the given function.
///
/// ## Deutsches Synonym
/// [`kombiniere`]
#[macro_export]
macro_rules! combine {
    ($function: expr, $a: expr, $b: expr, $c: expr, $d: expr, $e: expr, $f: expr, $g: expr $(, $tail: ident)+ $(,)?) => {
        $crate::kombiniere!($function, $a, $b, $c, $d, $e, $f, $g $(, $tail)+)
    };
    ($function: expr $(, $tail: expr)* $(,)?) => {
        $crate::kombiniere!($function $(, $tail)*)
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
        args: Box<dyn '_ + Iterator<Item = Option<ArgumentInput>>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<ArgumentInput>>);

    /// Erzeuge den Hilfetext für die enthaltenen [`Einzelargumente`](EinzelArgument).
    fn erzeuge_hilfe_text(
        &self,
        variante: &dyn ErzeugeHilfeText,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> NonEmpty<hilfe::Alternativen>;

    /// Provide a specialized [`Debug`]-implementation.
    /// If left unspecified, a placeholder-string is used instead.
    ///
    /// ## Errors
    /// Following the same rules as [`Debug::fmt`](std::fmt::Debug::fmt).
    #[inline]
    fn debug_fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "<konvertiere>")
    }
}

impl<'t, T, Fehler, F: FnOnce() -> T> Kombiniere<'t, T, Fehler> for F {
    #[inline]
    fn parse(
        self: Box<Self>,
        args: Box<dyn '_ + Iterator<Item = Option<ArgumentInput>>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<ArgumentInput>>) {
        (Ergebnis::Wert(self()), args.collect())
    }

    #[inline]
    fn erzeuge_hilfe_text(
        &self,
        _variante: &dyn ErzeugeHilfeText,
        _meta_standard: &str,
        _meta_erlaubte_werte: &str,
    ) -> NonEmpty<hilfe::Alternativen> {
        nonempty![hilfe::Alternativen::Leer]
    }

    #[inline]
    fn debug_fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "<closure>")
    }
}

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
                Fehler: Debug,
                $(
                    [<'t $suffix:snake:lower>]: 't,
                    [<T $suffix:camel>]: Debug,
                )+
            {
                #[inline]
                fn parse(
                    self: Box<Self>,
                    args: Box<dyn '_ + Iterator<Item = Option<ArgumentInput>>>,
                ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<ArgumentInput>>) {
                    let (funktion, $([<arg_ $suffix:snake:lower>]),+) = *self;
                    let nicht_verwendet: Vec<_> = args.collect();
                    let mut alle_fehler = Vec::new();
                    let mut alle_frühes_beenden = Vec::new();
                    $(
                        let (ergebnis, nicht_verwendet)
                            = [<arg_ $suffix:snake:lower>].parse_rekursiv(nicht_verwendet.into_iter());
                        let mut [<wert_ $suffix:snake:lower>] = None;
                        match ergebnis {
                            Ergebnis::Wert(wert) => [<wert_ $suffix:snake:lower>] = Some(wert),
                            Ergebnis::FrühesBeenden(nachrichten) => alle_frühes_beenden.extend(nachrichten),
                            Ergebnis::Fehler(fehler_liste) => {
                                alle_fehler.extend(fehler_liste)
                            },
                        }
                    )+
                    let ergebnis = match NonEmpty::from_vec(alle_frühes_beenden) {
                        Some(nachrichten) if alle_fehler.iter().all(|fehler| {
                            matches!(
                                fehler,
                                $crate::Fehler::FehlendeFlag { .. } | $crate::Fehler::FehlenderWert { .. }
                            )
                        }) => Ergebnis::FrühesBeenden(nachrichten),
                        _ => {
                            if let Some(fehler) = NonEmpty::from_vec(alle_fehler) {
                                Ergebnis::Fehler(fehler)
                            } else {
                                Ergebnis::Wert(funktion(
                                    $([<wert_ $suffix:snake:lower>].expect("Kein Fehler oder FrühesBeenden!")
                                ),+))
                            }
                        }
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
                    let (_f, $([<arg_ $suffix:snake:lower>]),+) = self;
                    let mut hilfe_texte = Vec::new();
                    $(
                        hilfe_texte.extend(
                            [<arg_ $suffix:snake:lower>]
                                .erzeuge_hilfe_text(variante, meta_standard, meta_erlaubte_werte)
                        );
                    )+
                    NonEmpty::from_vec(hilfe_texte).expect("Mindestens ein suffix als Macro-Argument!")
                }

                #[inline]
                fn debug_fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                    let (_funktion, $([<arg_ $suffix:snake:lower>]),+) = self;
                    write!(formatter, "(<closure>")?;
                    $(
                        write!(formatter, "{:?}", [<arg_ $suffix:snake:lower>])?;
                    )+
                    write!(formatter, ")")
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
