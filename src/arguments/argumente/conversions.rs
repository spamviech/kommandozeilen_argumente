//! Conversions and formatting for [`Argumente`].

use std::{
    borrow::Cow,
    fmt::{self, Debug, Display},
};

use nonempty::NonEmpty;

use crate::{
    arguments::{
        Arguments,
        argumente::Argumente,
        combine::Combine,
        early_exit::FrühesBeenden,
        flag::Flag,
        single_argument::EinzelArgument,
        value::{Value, Wert},
    },
    dyn_to_owned,
};

/// Hilfs-Typ um [`Combine::debug_fmt`] in der [`Debug`]-Implementierung zu verwenden.
struct KombiniereDebug<'s, 't, T, Fehler>(&'s dyn Combine<'t, T, Fehler>);

impl<T, Fehler> Debug for KombiniereDebug<'_, '_, T, Fehler> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.debug_fmt(formatter)
    }
}

impl<T: Debug, Fehler: Debug> Debug for Argumente<'_, T, Fehler> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Argumente::EinzelArgument(arg0) => {
                formatter.debug_tuple("EinzelArgument").field(arg0).finish()
            },
            Argumente::Kombiniere(arg0) => {
                formatter.debug_tuple("Kombiniere").field(&KombiniereDebug(&**arg0)).finish()
            },
            Argumente::Alternativen(arg0) => {
                formatter.debug_tuple("Alternativen").field(arg0).finish()
            },
        }
    }
}

impl<'t, T, Error> From<Arguments<'t, T, Error>> for Argumente<'t, T, Error> {
    #[inline]
    fn from(arguments: Arguments<'t, T, Error>) -> Self {
        match arguments {
            Arguments::Single(argument) => Self::EinzelArgument(argument.into()),
            Arguments::Combined(combine) => Self::Kombiniere(combine),
            Arguments::Alternatives(alternatives) => {
                Self::Alternativen(Box::new(alternatives.map(Into::into)))
            },
        }
    }
}

impl<'t, T, Fehler> From<Argumente<'t, T, Fehler>> for Arguments<'t, T, Fehler> {
    #[inline]
    fn from(argumente: Argumente<'t, T, Fehler>) -> Self {
        match argumente {
            Argumente::EinzelArgument(argument) => Self::Single(argument.into()),
            Argumente::Kombiniere(combine) => Self::Combined(combine),
            Argumente::Alternativen(alternatives) => {
                Self::Alternatives(Box::new(alternatives.map(Into::into)))
            },
        }
    }
}

#[expect(clippy::multiple_inherent_impl, reason = "Split into multiple sub-modules.")]
impl<'t, T, Fehler> Argumente<'t, T, Fehler> {
    /// Konvertiert den geparsten Wert mit `konvertiere`.
    ///
    /// Die ursprüngliche Argument-Konfiguration bleibt für Erkennung, Validierung und Hilfetext
    /// zuständig; `konvertiere` wird erst nach erfolgreicher Akkumulation eines Werts aufgerufen.
    #[inline]
    pub fn konvertiere<S>(
        konvertiere: impl 't + Fn(T) -> S,
        argumente: Argumente<'t, T, Fehler>,
    ) -> Argumente<'t, S, Fehler> {
        Arguments::convert(konvertiere, argumente.into()).into()
    }
}

impl<'t, T, Fehler> From<EinzelArgument<'t, T, Fehler>> for Arguments<'t, T, Fehler> {
    #[inline]
    fn from(argument: EinzelArgument<'t, T, Fehler>) -> Self {
        Self::Single(argument.into())
    }
}

impl<'t, T, Fehler> From<Wert<'t, T, Fehler>> for Arguments<'t, T, Fehler> {
    #[inline]
    fn from(value: Wert<'t, T, Fehler>) -> Self {
        Self::Single(Value::from(value).into())
    }
}
impl<'t, T, Fehler> From<EinzelArgument<'t, T, Fehler>> for Argumente<'t, T, Fehler> {
    #[inline]
    fn from(argument: EinzelArgument<'t, T, Fehler>) -> Self {
        Argumente::EinzelArgument(argument)
    }
}

impl<'t, T, Fehler> From<Flag<'t, T>> for Argumente<'t, T, Fehler> {
    #[inline]
    fn from(flag: Flag<'t, T>) -> Self {
        Argumente::EinzelArgument(EinzelArgument::from(flag))
    }
}

impl<'t, T, Fehler, Anzeige: 't + Fn(&T) -> String + Clone> From<(FrühesBeenden<'t>, T, Anzeige)>
    for Argumente<'t, T, Fehler>
{
    #[inline]
    fn from((frühes_beenden, wert, anzeige): (FrühesBeenden<'t>, T, Anzeige)) -> Self {
        let anzeige_boxed: Box<dyn 't + dyn_to_owned::Show<'t, T>> = Box::new(anzeige);
        Argumente::EinzelArgument(EinzelArgument::FrühesBeenden {
            frühes_beenden,
            wert,
            anzeige: Cow::Owned(anzeige_boxed),
        })
    }
}

impl<'t, T: Display, Fehler> From<(FrühesBeenden<'t>, T)> for Argumente<'t, T, Fehler> {
    #[inline]
    fn from((frühes_beenden, wert): (FrühesBeenden<'t>, T)) -> Self {
        Argumente::EinzelArgument(EinzelArgument::FrühesBeenden {
            frühes_beenden,
            wert,
            anzeige: Cow::Borrowed(&ToString::to_string),
        })
    }
}

impl<'t, Fehler> From<FrühesBeenden<'t>> for Argumente<'t, (), Fehler> {
    #[inline]
    fn from(frühes_beenden: FrühesBeenden<'t>) -> Self {
        Argumente::EinzelArgument(EinzelArgument::FrühesBeenden {
            frühes_beenden,
            wert: (),
            anzeige: Cow::Borrowed(&|wert| format!("{wert:?}")),
        })
    }
}

impl<'t, T, Fehler> From<Wert<'t, T, Fehler>> for Argumente<'t, T, Fehler> {
    #[inline]
    fn from(wert: Wert<'t, T, Fehler>) -> Self {
        Argumente::EinzelArgument(EinzelArgument::Wert(wert))
    }
}

impl<'t, T, Error> From<Value<'t, T, Error>> for Argumente<'t, T, Error> {
    #[inline]
    fn from(value: Value<'t, T, Error>) -> Self {
        Self::from(Wert::from(value))
    }
}

impl<T, Fehler> From<NonEmpty<Self>> for Argumente<'_, T, Fehler> {
    #[inline]
    fn from(alternativen: NonEmpty<Self>) -> Self {
        Argumente::Alternativen(Box::new(alternativen))
    }
}

impl<T, Fehler> From<Box<NonEmpty<Self>>> for Argumente<'_, T, Fehler> {
    #[inline]
    fn from(alternativen: Box<NonEmpty<Self>>) -> Self {
        Argumente::Alternativen(alternativen)
    }
}
