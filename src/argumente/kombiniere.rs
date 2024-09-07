//! Kombiniere mehrere [Argumente] zu einem neuen, basierend auf einer Funktion.

use std::{
    ffi::OsStr,
    fmt::{self, Debug, Formatter},
};

use nonempty::{nonempty, NonEmpty};
use paste::paste;

use crate::{
    argumente::{
        hilfe::{self, ErzeugeHilfeText},
        Argumente,
    },
    beschreibung::ArgumentInput,
    ergebnis::{Ergebnis, SingeArgResult, ZwischenErgebnis},
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
    /// Parse das übergebene Argument und erzeuge den zugehörigen Wert.
    ///
    /// ## English
    /// Parse the given argument and return the corresponding value.
    fn parse_single_arg<'s>(&self, arg: &'s OsStr) -> Vec<SingeArgResult<'s, T, Fehler>> {
        todo!("{arg:?}")
    }

    /// Parse die übergebenen Argumente und erzeuge den zugehörigen Wert.
    ///
    /// ## English
    /// Parse the given arguments and return the corresponding value.
    #[allow(clippy::type_complexity)]
    fn parse<'a>(
        self: Box<Self>,
        args: Box<dyn '_ + Iterator<Item = Option<&'a OsStr>>>,
    ) -> (ZwischenErgebnis<'t, T, Fehler, Argumente<'t, T, Fehler>>, Vec<Option<&'a OsStr>>);

    /// Parse die übergebenen Argumente und erzeuge den zugehörigen Wert.
    ///
    /// ## English
    /// Parse the given arguments and return the corresponding value.
    fn parse_merged_short_forms(
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

impl<'t, T, Fehler, F: Fn() -> T> Kombiniere<'t, T, Fehler> for F {
    #[inline]
    fn parse_single_arg<'s>(&self, arg: &'s OsStr) -> Vec<SingeArgResult<'s, T, Fehler>> {
        vec![SingeArgResult::FullParse { adjusted_arg: None, result: Ok(self()) }]
    }

    #[inline]
    fn parse<'a>(
        self: Box<Self>,
        args: Box<dyn '_ + Iterator<Item = Option<&'a OsStr>>>,
    ) -> (ZwischenErgebnis<'t, T, Fehler, Argumente<'t, T, Fehler>>, Vec<Option<&'a OsStr>>) {
        (ZwischenErgebnis::Wert(self()), args.collect())
    }

    #[inline]
    fn parse_merged_short_forms(
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

impl<'t, 'ta, 'tb, F, T, Fehler, TA, TB> Kombiniere<'t, T, Fehler>
    for (F, &Argumente<'ta, TA, Fehler>, &Argumente<'tb, TB, Fehler>)
where
    F: 't + Fn(TA, TB) -> T,
    Fehler: Debug,
    'ta: 't,
    'tb: 't,
    TA: Debug + Clone,
    TB: Debug + Clone,
{
    fn parse<'a>(
        self: Box<Self>,
        args: Box<dyn '_ + Iterator<Item = Option<&'a OsStr>>>,
    ) -> (ZwischenErgebnis<'t, T, Fehler, Argumente<'t, T, Fehler>>, Vec<Option<&'a OsStr>>) {
        todo!()
    }

    fn parse_merged_short_forms(
        self: Box<Self>,
        args: Box<dyn '_ + Iterator<Item = Option<ArgumentInput>>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<ArgumentInput>>) {
        todo!()
    }

    fn erzeuge_hilfe_text(
        &self,
        variante: &dyn ErzeugeHilfeText,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> NonEmpty<hilfe::Alternativen> {
        todo!()
    }
}

impl<'t, 'ta, 'tb, 'tc, F, T, Fehler, TA, TB, TC> Kombiniere<'t, T, Fehler>
    for (F, Argumente<'ta, TA, Fehler>, Argumente<'tb, TB, Fehler>, Argumente<'tc, TC, Fehler>)
where
    F: 't + Fn(TA, TB, TC) -> T,
    Fehler: Debug,
    'ta: 't,
    'tb: 't,
    'tc: 't,
    TA: Debug + Clone,
    TB: Debug + Clone,
    TC: Debug + Clone,
{
    #[inline]
    fn parse_single_arg<'s>(&self, arg: &'s OsStr) -> Vec<SingeArgResult<'s, T, Fehler>> {
        let (funktion, arg_a, arg_b, arg_c) = self;
        let mut results = Vec::new();
        let res_a = arg_a.parse_single_arg(arg);
        results.extend(res_a.into_iter().map(|single_arg_result| match single_arg_result {
            SingeArgResult::FullParse { adjusted_arg, result: Ok(value_a) } => {
                SingeArgResult::IncompleteParse {
                    adjusted_arg,
                    parse_following_arg: Box::new(move |new_arg: &OsStr| {
                        let adjusted_function =
                            |value_b, value_c| funktion(value_a.clone(), value_b, value_c);
                        (adjusted_function, arg_b.clone(), arg_c.clone()).parse_single_arg(new_arg)
                    }),
                }
            },
            SingeArgResult::FullParse { adjusted_arg, result: Err(err) } => todo!(),
            SingeArgResult::NameOnly { adjusted_arg, parse_next_arg } => todo!(),
            SingeArgResult::IncompleteParse { adjusted_arg, parse_following_arg } => todo!(),
        }));
        todo!();
        results
    }

    #[inline]
    fn parse<'a>(
        self: Box<Self>,
        args: Box<dyn '_ + Iterator<Item = Option<&'a OsStr>>>,
    ) -> (ZwischenErgebnis<'t, T, Fehler, Argumente<'t, T, Fehler>>, Vec<Option<&'a OsStr>>) {
        todo!()
    }

    #[inline]
    fn parse_merged_short_forms(
        self: Box<Self>,
        args: Box<dyn '_ + Iterator<Item = Option<ArgumentInput>>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<ArgumentInput>>) {
        todo!()
    }

    #[inline]
    fn erzeuge_hilfe_text(
        &self,
        variante: &dyn ErzeugeHilfeText,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> NonEmpty<hilfe::Alternativen> {
        todo!()
    }

    #[inline]
    fn debug_fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        todo!()
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
                F: 't + Fn($([<T $suffix:camel>]),+) -> T,
                Fehler: Debug,
                $(
                    [<'t $suffix:snake:lower>]: 't,
                    [<T $suffix:camel>]: Debug,
                )+
            {
                #[inline]
                fn parse_single_arg<'s>(&self, arg: &'s OsStr) -> Vec<SingeArgResult<'s, T, Fehler>> {
                    let (funktion, $([<arg_ $suffix:snake:lower>]),+) = self;
                    todo!()
                }

                #[inline]
                fn parse<'a>(
                    self: Box<Self>,
                    args: Box<dyn '_ + Iterator<Item = Option<&'a OsStr>>>,
                ) -> (ZwischenErgebnis<'t, T, Fehler,  Argumente<'t, T, Fehler>>, Vec<Option<&'a OsStr>>) {
                    let (funktion, $([<arg_ $suffix:snake:lower>]),+) = *self;
                    let nicht_verwendet: Vec<_> = args.collect();
                    let mut any_fehler = false;
                    let mut any_frühes_beenden = false;
                    let mut any_incomplete = false;
                    $(
                        let ([<zwischen_ergebnis_ $suffix:snake:lower>], nicht_verwendet)
                            = [<arg_ $suffix:snake:lower>].parse_rekursiv(nicht_verwendet.into_iter());
                        match &[<zwischen_ergebnis_ $suffix:snake:lower>] {
                            ZwischenErgebnis::Wert(_wert) => {}
                            ZwischenErgebnis::FrühesBeenden(_nachrichten) => any_frühes_beenden = true,
                            ZwischenErgebnis::Fehler(_fehler_liste) => any_fehler = true,
                            ZwischenErgebnis::Incomplete(_incomplete) => any_incomplete = true,
                        }
                    )+
                    let zwischen_ergebnis = if any_incomplete {
                        let tuple = (funktion, $([<zwischen_ergebnis_ $suffix:snake:lower>]),+);
                        ZwischenErgebnis::Incomplete(Argumente::Kombiniere(Box::new(tuple)))
                    } else if any_frühes_beenden {
                        let mut alle_nachrichten = Vec::new();
                        $(
                            if let ZwischenErgebnis::FrühesBeenden(nachrichten)
                                = [<zwischen_ergebnis_ $suffix:snake:lower>]
                            {
                                alle_nachrichten.extend(nachrichten);
                            }
                        )+
                        let alle_nachrichten = NonEmpty::from_vec(alle_nachrichten)
                            .expect("Mindestens ein ZwischenErgebnis::FrühesBeenden");
                        ZwischenErgebnis::FrühesBeenden(alle_nachrichten)
                    } else if any_fehler {
                        let mut alle_fehler = Vec::new();
                        $(
                            if let ZwischenErgebnis::Fehler(fehler_liste)
                                = [<zwischen_ergebnis_ $suffix:snake:lower>]
                            {
                                alle_fehler.extend(fehler_liste);
                            }
                        )+
                        let alle_fehler = NonEmpty::from_vec(alle_fehler)
                            .expect("Mindestens ein ZwischenErgebnis::Fehler");
                        ZwischenErgebnis::Fehler(alle_fehler)
                    } else {
                        $(
                            let ZwischenErgebnis::Wert([<wert_ $suffix:snake:lower>])
                             = [<zwischen_ergebnis_ $suffix:snake:lower>] else {
                                unreachable!("Weder Incomplete, FrühesBeenden, noch Fehler!")
                             };
                        )+
                        ZwischenErgebnis::Wert(funktion($([<wert_ $suffix:snake:lower>]),+))
                    };
                    (zwischen_ergebnis, nicht_verwendet)
                }

                #[inline]
                fn parse_merged_short_forms(
                    self: Box<Self>,
                    args: Box<dyn '_ + Iterator<Item = Option<ArgumentInput>>>,
                ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<ArgumentInput>>) {
                    let (funktion, $([<arg_ $suffix:snake:lower>]),+) = *self;
                    let nicht_verwendet: Vec<_> = args.collect();
                    let mut alle_fehler = Vec::new();
                    let mut alle_frühes_beenden = Vec::new();
                    $(
                        let (ergebnis, nicht_verwendet)
                            = [<arg_ $suffix:snake:lower>].parse_rekursiv_merged_short_forms(nicht_verwendet.into_iter());
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

            impl <
                't, $([<'t $suffix:snake:lower>],)+ F, T, Fehler,
                $([<T $suffix:camel>],)+
            >
                Kombiniere<'t, T, Fehler> for (
                    F,
                    $(ZwischenErgebnis<
                        [<'t $suffix:snake:lower>],
                        [<T $suffix:camel>],
                        Fehler,
                        Argumente<
                            [<'t $suffix:snake:lower>],
                            [<T $suffix:camel>],
                            Fehler,
                        >
                    >),+
                )
            where
                F: 't + Fn($([<T $suffix:camel>]),+) -> T,
                Fehler: Debug,
                $(
                    [<'t $suffix:snake:lower>]: 't,
                    [<T $suffix:camel>]: Debug,
                )+
            {
                #[inline]
                fn parse<'a>(
                    self: Box<Self>,
                    args: Box<dyn '_ + Iterator<Item = Option<&'a OsStr>>>,
                ) -> (ZwischenErgebnis<'t, T, Fehler,  Argumente<'t, T, Fehler>>, Vec<Option<&'a OsStr>>) {
                    todo!()
                }

                #[inline]
                fn parse_merged_short_forms(
                    self: Box<Self>,
                    args: Box<dyn '_ + Iterator<Item = Option<ArgumentInput>>>,
                ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<ArgumentInput>>) {
                    todo!()
                }

                #[inline]
                fn erzeuge_hilfe_text(
                    &self,
                    variante: &dyn ErzeugeHilfeText,
                    meta_standard: &str,
                    meta_erlaubte_werte: &str,
                ) -> NonEmpty<hilfe::Alternativen> {
                    todo!()
                }

                #[inline]
                fn debug_fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                    todo!()
                }
            }
        }
    };
}

impl_kombiniere_tuple!(A);
impl_kombiniere_tuple!(A, B);
// impl_kombiniere_tuple!(A, B, C);
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
