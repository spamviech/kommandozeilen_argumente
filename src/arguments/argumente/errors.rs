//! Error conversion methods for [`Argumente`].

use void::Void;

use crate::arguments::{Arguments, argumente::Argumente};

#[expect(clippy::multiple_inherent_impl, reason = "Split into multiple sub-modules.")]
impl<'t, T, Fehler> Argumente<'t, T, Fehler> {
    /// Konvertiere den Fehler mit der spezifizierten Funktion.
    ///
    /// ## English synonym
    /// [`Arguments::convert_error`]
    #[inline]
    pub fn konvertiere_fehler<NeuerFehler>(
        self,
        mapper: impl 't + Fn(Fehler) -> NeuerFehler + Clone,
        anzeige_neuer_fehler: impl 't + Fn(&NeuerFehler) -> String + Clone,
    ) -> Argumente<'t, T, NeuerFehler> {
        Arguments::from(self).convert_error(mapper, anzeige_neuer_fehler).into()
    }

    /// [`konvertiere_fehler`](Self::konvertiere_fehler) mit [`From::from`].
    ///
    /// ## English synonym
    /// [`Arguments::error_from`]
    #[inline]
    pub fn fehler_from<NeuerFehler: From<Fehler>>(
        self,
        anzeige_neuer_fehler: impl 't + Fn(&NeuerFehler) -> String + Clone,
    ) -> Argumente<'t, T, NeuerFehler> {
        self.konvertiere_fehler(NeuerFehler::from, anzeige_neuer_fehler)
    }
}

impl<'t, T> Argumente<'t, T, Void> {
    /// [`konvertiere_fehler`](Self::konvertiere_fehler) mit [`void::unreachable`].
    ///
    /// ## English synonym
    /// [`Arguments::error_from_void`]
    #[inline]
    pub fn fehler_from_void<NeuerFehler>(
        self,
        anzeige_neuer_fehler: impl 't + Fn(&NeuerFehler) -> String + Clone,
    ) -> Argumente<'t, T, NeuerFehler> {
        self.konvertiere_fehler(|void| void::unreachable(void), anzeige_neuer_fehler)
    }
}
