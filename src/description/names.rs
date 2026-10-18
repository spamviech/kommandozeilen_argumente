//! Flexible long- and short-name construction traits.

use std::convert::AsRef;

use nonempty::NonEmpty;

use crate::unicode::{Case, Compare, Normalized};

/// One or more long names.
pub trait LongNames<'t> {
    /// Converts to a non-empty collection of comparisons.
    fn long_names(self) -> NonEmpty<Compare<'t>>;
}
/// Implements [`LongNames`] for a supported long-name representation.
macro_rules! impl_long_names {
    ($type:ty) => {
        impl<'t> LongNames<'t> for $type {
            #[inline]
            fn long_names(self) -> NonEmpty<Compare<'t>> {
                NonEmpty::singleton(self.into())
            }
        }
        impl<'t> LongNames<'t> for ($type, Case) {
            #[inline]
            fn long_names(self) -> NonEmpty<Compare<'t>> {
                NonEmpty::singleton(self.into())
            }
        }
        impl<'t> LongNames<'t> for NonEmpty<$type> {
            #[inline]
            fn long_names(self) -> NonEmpty<Compare<'t>> {
                let NonEmpty { head, tail } = self;
                NonEmpty { head: head.into(), tail: tail.into_iter().map(Into::into).collect() }
            }
        }
        impl<'t> LongNames<'t> for NonEmpty<($type, Case)> {
            #[inline]
            fn long_names(self) -> NonEmpty<Compare<'t>> {
                let NonEmpty { head, tail } = self;
                NonEmpty { head: head.into(), tail: tail.into_iter().map(Into::into).collect() }
            }
        }
    };
}
impl_long_names! {String}
impl_long_names! {&'t str}
impl_long_names! {Normalized<'t>}
impl<'t> LongNames<'t> for Compare<'t> {
    #[inline]
    fn long_names(self) -> NonEmpty<Compare<'t>> {
        NonEmpty::singleton(self)
    }
}
impl<'t> LongNames<'t> for NonEmpty<Compare<'t>> {
    #[inline]
    fn long_names(self) -> NonEmpty<Compare<'t>> {
        self
    }
}
impl<'t, S: AsRef<str>> LongNames<'t> for &'t NonEmpty<S> {
    #[inline]
    fn long_names(self) -> NonEmpty<Compare<'t>> {
        let NonEmpty { head, tail } = self;
        NonEmpty {
            head: head.as_ref().into(),
            tail: tail.iter().map(|value| value.as_ref().into()).collect(),
        }
    }
}

/// Zero or more short names.
pub trait ShortNames<'t> {
    /// Converts to comparisons.
    fn short_names(self) -> Vec<Compare<'t>>;
}
/// Implements [`ShortNames`] for a supported short-name representation.
macro_rules! impl_short_names {
    ($type:ty) => {
        impl<'t> ShortNames<'t> for $type {
            #[inline]
            fn short_names(self) -> Vec<Compare<'t>> {
                vec![self.into()]
            }
        }
        impl<'t> ShortNames<'t> for ($type, Case) {
            #[inline]
            fn short_names(self) -> Vec<Compare<'t>> {
                vec![self.into()]
            }
        }
        macro_rules! impl_into_iter {
            ($collection: ident) => {
                impl<'t> ShortNames<'t> for $collection<$type> {
                    #[inline]
                    fn short_names(self) -> Vec<Compare<'t>> {
                        self.into_iter().map(Into::into).collect()
                    }
                }
                impl<'t> ShortNames<'t> for $collection<($type, Case)> {
                    #[inline]
                    fn short_names(self) -> Vec<Compare<'t>> {
                        self.into_iter().map(Into::into).collect()
                    }
                }
            };
        }
        impl_into_iter! {Option}
        impl_into_iter! {Vec}
        impl_into_iter! {NonEmpty}
    };
}
impl_short_names! {String}
impl_short_names! {&'t str}
impl_short_names! {Normalized<'t>}
impl<'t> ShortNames<'t> for Vec<Compare<'t>> {
    #[inline]
    fn short_names(self) -> Vec<Compare<'t>> {
        self
    }
}
impl<'t, S: AsRef<str>> ShortNames<'t> for &'t Vec<S> {
    #[inline]
    fn short_names(self) -> Vec<Compare<'t>> {
        self.iter().map(|value| value.as_ref().into()).collect()
    }
}
