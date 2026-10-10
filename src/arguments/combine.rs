//! Combine multiple [`Arguments`] into a new one based on a function.

use std::{
    ffi::OsString,
    fmt::{self, Debug, Formatter},
};

use nonempty::{NonEmpty, nonempty};
use paste::paste;

use crate::{
    arguments::{
        ArgumentInput, Arguments, ParsedLongForms, ParsedMergedShortForms, ParsedShortForms,
        argumente::Argumente,
        help::{self, CreateHelpText},
    },
    outcome::{IntermediateResult, ZwischenErgebnis},
};

/// Combine multiple arguments with the given function.
#[macro_export]
macro_rules! combine {
    ($function: expr, $a: expr, $b: expr, $c: expr, $d: expr, $e: expr, $f: expr, $g: expr $(, $tail: ident)+ $(,)?) => {
        #[allow(clippy::shadow_unrelated, clippy::shadow_same, clippy::shadow_reuse)]
        {
            let combine = $crate::arguments::Arguments::combine((
                |a, b, c, d, e, f, g| (a, b, c, d, e, f, g),
                $crate::arguments::Arguments::from($a),
                $crate::arguments::Arguments::from($b),
                $crate::arguments::Arguments::from($c),
                $crate::arguments::Arguments::from($d),
                $crate::arguments::Arguments::from($e),
                $crate::arguments::Arguments::from($f),
                $crate::arguments::Arguments::from($g),
            ));
            let uncurry_function = move |(a, b, c, d, e, f, g) $(, $tail)+| {
                $function(a, b, c, d, e, f, g $(, $tail)+)
            };
            $crate::combine!(uncurry_function, combine $(, $tail)+)
        }
    };
    ($function: expr $(,)?) => {
        $crate::arguments::Arguments::combine(($function,))
    };
    ($function: expr, $($tail: expr),+ $(,)?) => {
        $crate::arguments::Arguments::combine((
            $function, $($crate::arguments::Arguments::from($tail)),+
        ))
    };
}

/// Combine multiple arguments with the given function.
///
/// ## Compatibility
///
/// Use [`combine!`] in new code.
#[macro_export]
macro_rules! kombiniere {
    ($($tokens:tt)*) => {
        $crate::arguments::argumente::Argumente::from($crate::combine!($($tokens)*))
    };
}

/// Allow combining multiple arguments.
pub trait Combine<'t, T, Fehler> {
    /// Create help-text entries for the combined arguments.
    #[inline]
    fn create_help_text(
        &self,
        _variante: &dyn CreateHelpText,
        _meta_standard: &str,
        _meta_erlaubte_werte: &str,
    ) -> NonEmpty<help::Alternativen> {
        nonempty![help::Alternativen::Leer]
    }

    /// Format this combination for [`Debug`].
    #[inline]
    fn debug_fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "<closure>")
    }

    /// Parses merged short-form arguments for this combination.
    ///
    /// Tuple implementations provide the staged composition in a later parsing phase.
    #[inline]
    fn parse_merged_short_forms(
        &self,
        args: Box<dyn Iterator<Item = OsString> + '_>,
    ) -> NonEmpty<ParsedMergedShortForms<'t>>
    {
        let _ = args;
        todo!()
    }

    /// Parses merged short-form arguments from staged input.
    #[inline]
    fn parse_merged_short_inputs(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedMergedShortForms<'t>> {
        self.parse_merged_short_forms(Box::new(args.filter_map(|argument| match argument {
            ArgumentInput::Unchanged(argument) => Some(argument),
            ArgumentInput::AdjustedMergedShortNames(_) => None,
        })))
    }

    /// Parses standalone short-form arguments for this combination.
    #[inline]
    fn parse_short_forms(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedShortForms<'t>>
    {
        let _ = args;
        todo!()
    }

    /// Parses long-form arguments for this combination.
    #[inline]
    fn parse_long_forms(
        &self,
        args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
    ) -> NonEmpty<ParsedLongForms<'t>>
    {
        let _ = args;
        todo!()
    }
}

/// Implement the [`Combine`] trait for a tuple `(f, a0, a1, ...)`.
macro_rules! impl_combine_tuple {
    ($($suffix: ident),+ $(,)?) => {
        paste! {
            impl <
                't, $([<'t $suffix:snake:lower>],)+ F, T, Error,
                $([<T $suffix:camel>],)+
            >
                Combine<'t, T, Error> for (
                    F,
                    $(Arguments<
                        [<'t $suffix:snake:lower>],
                        [<T $suffix:camel>],
                        Error,
                    >),+
                )
            where
                F: 't + Fn($([<T $suffix:camel>]),+) -> T,
                Error: Debug,
                $(
                    [<'t $suffix:snake:lower>]: 't,
                    [<T $suffix:camel>]: Debug,
                )+
            {
                #[inline]
                fn create_help_text(
                    &self,
                    variant: &dyn CreateHelpText,
                    meta_default: &str,
                    meta_possible_values: &str,
                ) -> NonEmpty<help::Alternativen> {
                    let (_function, $([<arg_ $suffix:snake:lower>]),+) = self;
                    let mut helps = Vec::new();
                    $(
                        helps.extend(
                            [<arg_ $suffix:lower>]
                                .create_help_text(variant, meta_default, meta_possible_values)
                                .map(Into::into)
                        );
                    )+
                    NonEmpty::from_vec(helps).expect("At least one suffix as a macro argument!")
                }

                #[inline]
                fn parse_merged_short_forms(
                    &self,
                    args: Box<dyn Iterator<Item = OsString> + '_>,
                ) -> NonEmpty<ParsedMergedShortForms<'t>> {
                    self.parse_merged_short_inputs(Box::new(args.map(ArgumentInput::Unchanged)))
                }

                #[inline]
                fn parse_merged_short_inputs(
                    &self,
                    args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
                ) -> NonEmpty<ParsedMergedShortForms<'t>> {
                    let (_function, $([<arg_ $suffix:snake:lower>]),+) = self;
                    let mut candidates = vec![ParsedMergedShortForms::empty()];
                    candidates[0].remaining = args.collect();
                    $(
                        candidates = candidates.into_iter().flat_map(|candidate| {
                            [<arg_ $suffix:snake:lower>]
                                .parse_merged_short_inputs(candidate.remaining.clone().into_iter())
                                .into_iter()
                                .map(move |parsed| candidate.clone().merge(parsed))
                        }).collect();
                    )+
                    NonEmpty::from_vec(candidates).expect("Tuple combinations have at least one candidate.")
                }

                #[inline]
                fn parse_short_forms(
                    &self,
                    args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
                ) -> NonEmpty<ParsedShortForms<'t>>
                {
                    let (_function, $([<arg_ $suffix:snake:lower>]),+) = self;
                    let mut candidates = vec![ParsedShortForms::empty()];
                    candidates[0].remaining = args.collect();
                    $(
                        candidates = candidates.into_iter().flat_map(|candidate| {
                            [<arg_ $suffix:snake:lower>]
                                .parse_short_forms(candidate.remaining.clone().into_iter())
                                .into_iter()
                                .map(move |parsed| candidate.clone().merge(parsed))
                        }).collect();
                    )+
                    NonEmpty::from_vec(candidates).expect("Tuple combinations have at least one candidate.")
                }

                #[inline]
                fn parse_long_forms(
                    &self,
                    args: Box<dyn Iterator<Item = ArgumentInput> + '_>,
                ) -> NonEmpty<ParsedLongForms<'t>>
                {
                    let (_function, $([<arg_ $suffix:snake:lower>]),+) = self;
                    let mut candidates = vec![ParsedLongForms::empty()];
                    candidates[0].remaining = args.collect();
                    $(
                        candidates = candidates.into_iter().flat_map(|candidate| {
                            [<arg_ $suffix:snake:lower>]
                                .parse_long_forms(candidate.remaining.clone().into_iter())
                                .into_iter()
                                .map(move |parsed| candidate.clone().merge(parsed))
                        }).collect();
                    )+
                    NonEmpty::from_vec(candidates).expect("Tuple combinations have at least one candidate.")
                }

                #[inline]
                fn debug_fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                    let (_function, $([<arg_ $suffix:snake:lower>]),+) = self;
                    write!(formatter, "(<closure>")?;
                    $(
                        write!(formatter, "{:?}", [<arg_ $suffix:snake:lower>])?;
                    )+
                    write!(formatter, ")")
                }
            }

            impl <
                't, $([<'t $suffix:snake:lower>],)+ F, T, Error,
                $([<T $suffix:camel>],)+
            >
                Combine<'t, T, Error> for (
                    F,
                    $(IntermediateResult<
                        [<'t $suffix:snake:lower>],
                        [<T $suffix:camel>],
                        Error,
                        Arguments<
                            [<'t $suffix:snake:lower>],
                            [<T $suffix:camel>],
                            Error,
                        >
                    >),+
                )
            where
                F: 't + Fn($([<T $suffix:camel>]),+) -> T,
                Error: Debug,
                $(
                    [<'t $suffix:snake:lower>]: 't,
                    [<T $suffix:camel>]: Debug,
                )+
            {}

            impl <
                't, $([<'t $suffix:snake:lower>],)+ F, T, Fehler,
                $([<T $suffix:camel>],)+
            >
                Combine<'t, T, Fehler> for (
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
                fn create_help_text(
                    &self,
                    variante: &dyn CreateHelpText,
                    meta_standard: &str,
                    meta_erlaubte_werte: &str,
                ) -> NonEmpty<help::Alternativen> {
                    let (_f, $([<arg_ $suffix:snake:lower>]),+) = self;
                    let mut hilfe_texte = Vec::new();
                    $(
                        hilfe_texte.extend(
                            [<arg_ $suffix:lower>]
                                .erzeuge_hilfe_text(variante, meta_standard, meta_erlaubte_werte)
                        );
                    )+
                    NonEmpty::from_vec(hilfe_texte).expect("At least one suffix as a macro argument!")
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
                Combine<'t, T, Fehler> for (
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
            {}
        }
    };
}

impl<'t, F, T, Fehler> Combine<'t, T, Fehler> for (F,)
where
    F: 't + Fn() -> T,
    Fehler: Debug,
{
    #[inline]
    fn create_help_text(
        &self,
        _variante: &dyn CreateHelpText,
        _meta_standard: &str,
        _meta_erlaubte_werte: &str,
    ) -> NonEmpty<help::Alternativen> {
        nonempty![help::Alternativen::Leer]
    }

    #[inline]
    fn debug_fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "(<closure>)")
    }
}

impl_combine_tuple!(A);
impl_combine_tuple!(A, B);
impl_combine_tuple!(A, B, C);
impl_combine_tuple!(A, B, C, D);
impl_combine_tuple!(A, B, C, D, E);
impl_combine_tuple!(A, B, C, D, E, F);
impl_combine_tuple!(A, B, C, D, E, F, G);
impl_combine_tuple!(A, B, C, D, E, F, G, H);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_combine_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
