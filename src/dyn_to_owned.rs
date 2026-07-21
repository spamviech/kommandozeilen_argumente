//! Erstelle eine [`ToOwned`]-Implementierung für ein `dyn Fn` trait-Objekt.

use std::ffi::OsStr;

use crate::ParseFehler;

/// Erstelle eine [`ToOwned`]-Implementierung für ein `dyn Fn` trait-Objekt.
macro_rules! erstelle_dyn_to_owned {
    ($trait: ident <$dyn: lifetime, $($($lt: lifetime),+ ,)? $($param: ident),* $(,)?>, $($fn: tt)*) => {
        /// Hilfs-Trait, damit [`ToOwned`] für ein `dyn Fn` trait-Objekt implementiert werden kann.
        ///
        /// ## English
        /// Helper trait to implement [`ToOwned`] for a `dyn Fn` trait object.
        pub trait $trait<$dyn, $($($lt),+ ,)? $($param),*>: $dyn + $($fn)* + ::dyn_clone::DynClone {}

        ::dyn_clone::clone_trait_object!(<$($($lt),+ ,)? $($param),*>$trait<'_, $($($lt),+ ,)? $($param),*>);

        impl<$dyn, $($($lt),+ ,)? $($param,)* F: $dyn + Clone + $($fn)*> $trait<$dyn, $($($lt),+ ,)? $($param),*> for F {}

        impl<$dyn, $($($lt),+ ,)? $($param),*> ToOwned
            for dyn $dyn + $trait<$dyn, $($($lt),+ ,)? $($param),*>
        {
            type Owned = Box<dyn $dyn + $trait<$dyn, $($($lt),+ ,)? $($param),*>>;

            #[inline]
            fn to_owned(&self) -> Self::Owned {
                ::dyn_clone::clone_box(self)
            }
        }
    };
}

erstelle_dyn_to_owned!(Bool<'t, T>, Fn(bool) -> T);
erstelle_dyn_to_owned!(Parse<'t, T, Fehler>, Fn(&OsStr) -> Result<T, ParseFehler<Fehler>>);
erstelle_dyn_to_owned!(Show<'t, T>, Fn(&T) -> String);
