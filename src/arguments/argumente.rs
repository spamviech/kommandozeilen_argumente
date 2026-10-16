//! Deutsche Spiegeltypen für Kommandozeilen-Argumente.

pub use crate::arguments::parsing::{ParsedLongForms, ParsedMergedShortForms, ParsedShortForms};
/// Compatibility re-exports for established German module paths.
pub use crate::arguments::{combine, early_exit, flag, help, single_argument, value};

mod construction;
mod conversions;
mod errors;
mod hilfe;
mod parsing;

use nonempty::NonEmpty;

use crate::arguments::{combine::Combine, single_argument::EinzelArgument};

/// Konfiguration der Kommandozeilen-Argumente.
///
/// ## English synonym
/// [`Arguments`](crate::arguments::Arguments)
#[expect(
    clippy::large_enum_variant,
    reason = "The alternatives variant is intentionally boxed only where needed for the public \
              representation."
)]
#[must_use]
pub enum Argumente<'t, T, Fehler> {
    /// Ein einzelnes Argument.
    ///
    /// ## English
    /// A single argument.
    EinzelArgument(EinzelArgument<'t, T, Fehler>),
    /// Die Kombination mehrerer Argumente, kodiert über den [`Combine`]-Trait.
    ///
    /// ## English
    /// The combination of multiple arguments, encoded via the [`Combine`]-trait.
    Kombiniere(Box<dyn 't + Combine<'t, T, Fehler>>),
    /// Alternative Kommandozeilen-Argumente. Beim parsen wird das erste
    /// [`Ergebnis`](crate::Ergebnis) verwendet,
    /// dass kein [`Ergebnis::Fehler`](crate::Ergebnis::Fehler) ist.
    ///
    /// ## English
    /// Alternative command line arguments. Parsing takes the first non-[`Error`](crate::Error)
    /// [`Result`](crate::Result).
    Alternativen(Box<NonEmpty<Self>>),
}
