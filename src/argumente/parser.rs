//! Definition für Parse-Algorithmus

use std::fmt::{self, Debug};

use crate::{beschreibung::ArgumentInput, ergebnis::Ergebnis};

/// Noch nicht verwendete Argumente.
type ArgumentList = Vec<Option<ArgumentInput>>;

/// Einzelnes Ergebnis einer Parser-Funktion.
type ParserResult<'t, T, F> = (ArgumentList, Ergebnis<'t, T, F>);

/// Aktueller Zustand des Parse-Vorgangs.
pub struct Parser<'t, T, F>(pub Box<dyn Fn(ArgumentList) -> Vec<ParserResult<'t, T, F>>>);

impl<T, F> Debug for Parser<'_, T, F> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Parser").field(&"<closure>").finish()
    }
}
