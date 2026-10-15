//! Helpers for implementing [`ToOwned`] on `dyn Fn` trait objects.

use std::ffi::OsStr;

/// Implements [`ToOwned`] for a `dyn Fn` trait object.
macro_rules! impl_dyn_to_owned {
    ($trait:ident <$dyn:lifetime, $($($lifetime:lifetime),+ ,)? $($parameter:ident),* $(,)?>, $($function:tt)*) => {
        /// Helper trait that permits implementing [`ToOwned`] for a `dyn Fn` trait object.
        pub trait $trait<$dyn, $($($lifetime),+ ,)? $($parameter),*>:
            $dyn + $($function)* + ::dyn_clone::DynClone
        {}

        ::dyn_clone::clone_trait_object!(<$($($lifetime),+ ,)? $($parameter),*>$trait<'_, $($($lifetime),+ ,)? $($parameter),*>);

        impl<$dyn, $($($lifetime),+ ,)? $($parameter,)* Function: $dyn + Clone + $($function)*>
            $trait<$dyn, $($($lifetime),+ ,)? $($parameter),*> for Function
        {}

        impl<$dyn, $($($lifetime),+ ,)? $($parameter),*> ToOwned
            for dyn $dyn + $trait<$dyn, $($($lifetime),+ ,)? $($parameter),*>
        {
            type Owned = Box<dyn $dyn + $trait<$dyn, $($($lifetime),+ ,)? $($parameter),*>>;

            #[inline]
            fn to_owned(&self) -> Self::Owned {
                ::dyn_clone::clone_box(self)
            }
        }
    };
}

impl_dyn_to_owned!(Bool<'t, T>, Fn(bool) -> T);
impl_dyn_to_owned!(Parse<'t, T, Error>, Fn(&OsStr) -> Result<T, Error>);
impl_dyn_to_owned!(Show<'t, T>, Fn(&T) -> String);
