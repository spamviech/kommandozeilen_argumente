//! Help-text and early-exit methods for [`Argumente`].

use std::fmt::Debug;

use nonempty::{NonEmpty, nonempty};
use void::Void;

use crate::{
    arguments::{
        Arguments,
        argumente::Argumente,
        help::{self, CreateHelpText},
    },
    description::Beschreibung,
    language::Sprache,
};

impl<T, Fehler> Argumente<'_, T, Fehler> {
    /// Erzeugt die Syntax und den zugehörigen Hilfetext dieses Arguments.
    #[inline]
    #[must_use]
    pub fn erzeuge_hilfe_text(
        &self,
        variante: &dyn CreateHelpText,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> NonEmpty<help::Alternativen> {
        match self {
            Self::EinzelArgument(argument) => {
                nonempty![help::Alternativen::EinzelArgument(
                    variante
                        .create_help_text(
                            argument.als_string_wert().into(),
                            meta_standard,
                            meta_erlaubte_werte,
                        )
                        .into()
                )]
            },
            Self::Kombiniere(kombiniere) => kombiniere
                .create_help_text(variante, meta_standard, meta_erlaubte_werte)
                .map(Into::into),
            Self::Alternativen(alternativen) => {
                Box::as_ref(alternativen).as_ref().map(|argument| {
                    help::Alternativen::Alternativen(Box::new(argument.erzeuge_hilfe_text(
                        variante,
                        meta_standard,
                        meta_erlaubte_werte,
                    )))
                })
            },
        }
    }
}
impl<'t, T: Debug, Fehler: Debug> Argumente<'t, T, Fehler> {
    /// Füge eine [`FrühesBeenden`]-Flag hinzu, wodurch die Programm-Version anzeigt wird.
    ///
    /// ## English synonym
    /// [`Arguments::with_version_early_exit`]
    #[inline]
    pub fn mit_version_frühes_beenden(
        self,
        eigene_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_version: &str,
    ) -> Self {
        Arguments::from(self)
            .with_version_early_exit(eigene_beschreibung.into(), programm_name, programm_version)
            .into()
    }

    /// Variante von [`mit_version_frühes_beenden`](Self::mit_version_frühes_beenden),
    /// basierend auf einer [`Sprache`].
    ///
    /// ## English synonym
    /// [`Arguments::with_version_early_exit_with_language`]
    #[inline]
    pub fn mit_version_frühes_beenden_mit_sprache(
        self,
        programm_name: &str,
        programm_version: &str,
        sprache: Sprache,
    ) -> Self {
        Arguments::from(self)
            .with_version_early_exit_with_language(programm_name, programm_version, sprache.into())
            .into()
    }

    /// Füge eine [`FrühesBeenden`]-Flag hinzu, wodurch der Hilfe-Text für alle Argumente anzeigt
    /// wird.
    ///
    /// ### Panics
    /// Wenn die Syntax-Beschreibung (inklusive normalem + Alternativen-Präfix) für ein Argument
    /// länger als [`usize::MAX`] ist.
    ///
    /// ## English synonym
    /// [`Arguments::with_help_early_exit`]
    #[inline]
    #[expect(
        clippy::too_many_arguments,
        reason = "This public localization API accepts each independently configurable message \
                  explicitly."
    )]
    pub fn mit_hilfe_frühes_beenden(
        self,
        variante: &dyn CreateHelpText,
        eigene_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: Option<&str>,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
        meta_optionen: &str,
        meta_syntax_präfix: &str,
        meta_syntax_padding: char,
        meta_alternative_präfix: &str,
        meta_alternative_trennzeichen: char,
    ) -> Self {
        Arguments::from(self)
            .with_help_early_exit(
                variante,
                eigene_beschreibung.into(),
                programm_name,
                programm_beschreibung,
                programm_version,
                meta_standard,
                meta_erlaubte_werte,
                meta_optionen,
                meta_syntax_präfix,
                meta_syntax_padding,
                meta_alternative_präfix,
                meta_alternative_trennzeichen,
            )
            .into()
    }

    /// Variante von [`mit_hilfe_frühes_beenden`](Self::mit_hilfe_frühes_beenden),
    /// basierend auf einer [`Sprache`].
    ///
    /// ## English synonym
    /// [`Arguments::with_help_early_exit_with_language`]
    #[inline]
    pub fn mit_hilfe_frühes_beenden_mit_sprache(
        self,
        variante: &dyn CreateHelpText,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: Option<&str>,
        sprache: Sprache,
    ) -> Self {
        Arguments::from(self)
            .with_help_early_exit_with_language(
                variante,
                programm_name,
                programm_beschreibung,
                programm_version,
                sprache.into(),
            )
            .into()
    }

    /// Füge [`FrühesBeenden`]-Flags hinzu, wodurch die Programm-Version,
    /// bzw. der Hilfe-Text für alle Argumente anzeigt wird.
    ///
    /// ### Panics
    /// Wenn die Syntax-Beschreibung (inklusive normalem + Alternativen-Präfix) für ein Argument
    /// länger als [`usize::MAX`] ist.
    ///
    /// ## English synonym
    /// [`Arguments::with_help_and_version_early_exit`]
    #[inline]
    #[expect(
        clippy::too_many_arguments,
        reason = "This public localization API accepts each independently configurable message \
                  explicitly."
    )]
    pub fn mit_hilfe_und_version_frühes_beenden(
        self,
        variante: &dyn CreateHelpText,
        version_beschreibung: Beschreibung<'t, Void>,
        hilfe_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: &str,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
        meta_optionen: &str,
        meta_syntax_präfix: &str,
        meta_syntax_padding: char,
        meta_alternative_präfix: &str,
        meta_alternative_trennzeichen: char,
    ) -> Self {
        Arguments::from(self)
            .with_help_and_version_early_exit(
                variante,
                version_beschreibung.into(),
                hilfe_beschreibung.into(),
                programm_name,
                programm_beschreibung,
                programm_version,
                meta_standard,
                meta_erlaubte_werte,
                meta_optionen,
                meta_syntax_präfix,
                meta_syntax_padding,
                meta_alternative_präfix,
                meta_alternative_trennzeichen,
            )
            .into()
    }

    /// Variante von
    /// [`mit_hilfe_und_version_frühes_beenden`](Argumente::mit_hilfe_und_version_frühes_beenden).
    ///
    /// ## English synonym
    /// [`Arguments::with_help_and_version_early_exit_with_language`]
    #[inline]
    pub fn mit_hilfe_und_version_frühes_beenden_mit_sprache(
        self,
        variante: &dyn CreateHelpText,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: &str,
        sprache: Sprache,
    ) -> Self {
        Arguments::from(self)
            .with_help_and_version_early_exit_with_language(
                variante,
                programm_name,
                programm_beschreibung,
                programm_version,
                sprache.into(),
            )
            .into()
    }
}
