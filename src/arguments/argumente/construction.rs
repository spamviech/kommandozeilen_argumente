//! Constructors for [`Argumente`].

use nonempty::NonEmpty;

use crate::arguments::{
    Arguments, argumente::Argumente, combine::Combine, single_argument::EinzelArgument,
};

impl<'t, T, Fehler> Argumente<'t, T, Fehler> {
    /// Erzeuge eine [`Argumente::EinzelArgument`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`Argumente::EinzelArgument`]-variant with sensible type parameters.
    #[inline]
    pub fn einzel_argument(einzel_argument: EinzelArgument<'t, T, Fehler>) -> Self {
        Arguments::single_argument(einzel_argument.into()).into()
    }
}

#[expect(clippy::multiple_inherent_impl, reason = "Split into multiple sub-modules.")]
impl<'t, T, Fehler> Argumente<'t, T, Fehler> {
    /// Erzeuge eine [`Argumente::Kombiniere`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English synonym
    /// [`combine`](Self::combine)
    #[inline]
    pub fn kombiniere(kombiniere: impl 't + Combine<'t, T, Fehler>) -> Self {
        Arguments::combine(kombiniere).into()
    }

    /// Create a [`Argumente::Kombiniere`]-variant with sensible type parameters.
    ///
    /// ## Deutsches Synonym
    /// [`kombiniere`](Self::combine)
    #[inline]
    pub fn combine(combine: impl 't + Combine<'t, T, Fehler>) -> Self {
        Self::kombiniere(combine)
    }

    /// Erzeuge eine [`Argumente::Alternativen`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English synonym
    /// [`alternatives`](Self::alternatives)
    #[inline]
    pub fn alternativen(alternativen: NonEmpty<Self>) -> Self {
        Arguments::alternatives(alternativen.map(Into::into)).into()
    }

    /// Create a [`Argumente::Alternativen`]-variant with sensible type parameters.
    ///
    /// ## Deutsches Synonym
    /// [`alternativen`](Self::alternativen)
    #[inline]
    pub fn alternatives(alternatives: NonEmpty<Self>) -> Self {
        Self::alternativen(alternatives)
    }
}
