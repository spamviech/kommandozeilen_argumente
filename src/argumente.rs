//! Definition von akzeptierten Kommandozeilen-Argumenten.

use std::{
    borrow::Cow,
    env,
    ffi::{OsStr, OsString},
    fmt::Debug,
    path::Path,
};

use nonempty::{nonempty, NonEmpty};
use void::Void;

use crate::{
    argumente::{
        einzelargument::EinzelArgument,
        frühes_beenden::FrühesBeenden,
        hilfe::{ErzeugeHilfeText, Hilfe},
        kombiniere::Kombiniere,
    },
    beschreibung::Beschreibung,
    ergebnis::{Ergebnis, ParseFehler},
    sprache::Sprache,
};

pub mod einzelargument;
pub mod flag;
#[path = "argumente/frühes_beenden.rs"]
pub mod frühes_beenden;
pub mod hilfe;
pub mod kombiniere;
pub mod wert;

#[cfg_attr(all(doc, not(doctest)), doc(cfg(feature = "derive")))]
pub use self::wert::EnumArgument;

// TODO Name/Version für Hilfetext angeben, als alternative für macros (derive-Feature)
// TODO Unterbefehle/subcommands
// TODO Positions-basierte Argumente
// TODO Argument-Gruppen (nur eine dieser N Flags kann gleichzeitig aktiv sein)
// TODO Feature-gates für automatische Hilfe, verschmelzen von flag-kurzformen, ...
//      benötigen extra Felder in Argumente-Struktur, könnte Performance verbessern
// TODO tests mit Unicode-namen
// TODO OneOf/Either für alternative Parse-Möglichkeiten
//      alternativ-Methode (analog kombinierte2), besondere Methode für Either-Typen?
// TODO Standard-Wert, sofern nur der Name gegeben ist (unterschiedlich zu Name kommt nicht vor)
//      z.B. nichts: -O0, -O: -O1, -O=N für explizite Angabe
//      vgl. mit Flag-Argumenten, kann zu parse-Problemen wegen Mehrdeutigkeit führen
//      kann durch alternativ-Methode erzeugt werden (erst Wert, dann Flag)
//          dazu spezialisierte Methode bereitstellen

/// Konfiguration der Kommandozeilen-Argumente.
///
/// ## English synonym
/// [`Arguments`]
#[derive(Debug)]
#[allow(clippy::large_enum_variant, clippy::module_name_repetitions)]
#[must_use]
pub enum Argumente<'t, T, Bool, Parse, Anzeige, K> {
    /// Ein einzelnes Argument.
    ///
    /// ## English
    /// A single argument.
    EinzelArgument(EinzelArgument<'t, T, Bool, Parse, Anzeige>),
    /// Die Kombination mehrerer Argumente, kodiert über den [`Kombiniere`]-trait.
    ///
    /// ## English
    /// The combination of multiple arguments, encoded via the [`Kombiniere`]-trait.
    Kombiniere(K),
    /// Alternative Kommandozeilen-Argumente. Beim parsen wird das erste [`Ergebnis`] verwendet,
    /// dass kein [`Ergebnis::Fehler`] ist.
    ///
    /// ## English
    /// Alternative command line arguments. Parsing takes the first non-[`Error`](Ergebnis::Fehler)
    /// [`Result`](crate::Result).
    Alternativen(Box<NonEmpty<Self>>),
}

/// Configuration of command line arguments.
///
/// ## Deutsches Synonym
/// [`Argumente`]
pub type Arguments<'t, T, Bool, Parse, Anzeige, K> = Argumente<'t, T, Bool, Parse, Anzeige, K>;

impl<'t, T, Bool, Parse, Anzeige, K> From<EinzelArgument<'t, T, Bool, Parse, Anzeige>>
    for Argumente<'t, T, Bool, Parse, Anzeige, K>
{
    #[inline]
    fn from(argument: EinzelArgument<'t, T, Bool, Parse, Anzeige>) -> Self {
        Argumente::EinzelArgument(argument)
    }
}

impl<T, Bool, Parse, Anzeige, K> From<NonEmpty<Self>>
    for Argumente<'_, T, Bool, Parse, Anzeige, K>
{
    #[inline]
    fn from(alternativen: NonEmpty<Self>) -> Self {
        Argumente::Alternativen(Box::new(alternativen))
    }
}

impl<T, Bool, Parse, Anzeige, K> From<Box<NonEmpty<Self>>>
    for Argumente<'_, T, Bool, Parse, Anzeige, K>
{
    #[inline]
    fn from(alternativen: Box<NonEmpty<Self>>) -> Self {
        Argumente::Alternativen(alternativen)
    }
}

impl<'t, T, Bool, Parse, Anzeige> Argumente<'t, T, Bool, Parse, Anzeige, Void> {
    /// Erzeuge eine [`Argumente::EinzelArgument`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`Argumente::EinzelArgument`]-variant with sensible type parameters.
    #[inline]
    pub fn einzel_argument(einzel_argument: EinzelArgument<'t, T, Bool, Parse, Anzeige>) -> Self {
        Argumente::EinzelArgument(einzel_argument)
    }
}

impl<T, Bool, Parse, Anzeige, K> Argumente<'_, T, Bool, Parse, Anzeige, K> {
    /// Erzeuge eine [`Argumente::Kombiniere`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`Argumente::Kombiniere`]-variant with sensible type parameters.
    #[inline]
    pub fn kombiniere(kombiniere: K) -> Self {
        Argumente::Kombiniere(kombiniere)
    }

    /// Erzeuge eine [`Argumente::Alternativen`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`Argumente::Alternativen`]-variant with sensible type parameters.
    #[inline]
    pub fn alternativen(alternativen: NonEmpty<Self>) -> Self {
        Argumente::Alternativen(Box::new(alternativen))
    }

    /// Erzeuge eine [`Argumente::Alternativen`]-Variante mit sinnvollen Typ-Parametern.
    ///
    /// ## English
    /// Create a [`Argumente::Alternativen`]-variant with sensible type parameters.
    #[inline]
    pub fn alternativen_boxed(alternativen: Box<NonEmpty<Self>>) -> Self {
        Argumente::Alternativen(alternativen)
    }
}

impl<'t, T, Bool, Parse, Fehler, Anzeige, K> Argumente<'t, T, Bool, Parse, Anzeige, K>
where
    Bool: Fn(bool) -> T,
    Parse: Fn(&OsStr) -> Result<T, ParseFehler<Fehler>>,
    K: Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige>,
{
    /// Parse die übergebenen Argumente und erzeuge den zugehörigen Wert.
    ///
    /// ## English
    /// Parse the given arguments and return the corresponding value.
    #[inline]
    pub fn parse(
        self,
        args: impl Iterator<Item = Option<OsString>>,
    ) -> (Ergebnis<'t, T, Fehler>, Vec<Option<OsString>>) {
        use Argumente::{Alternativen, EinzelArgument, Kombiniere};
        use Ergebnis::{Fehler, FrühesBeenden, Wert};
        match self {
            EinzelArgument(arg) => arg.parse(args),
            Kombiniere(kombiniere) => kombiniere.parse(args),
            Alternativen(alternativen) => {
                // TODO only accept parsing without leftover args?
                let NonEmpty { head, tail } = *alternativen;
                let args_vec: Vec<_> = args.into_iter().collect();
                tail.into_iter().fold(
                    head.parse(args_vec.clone().into_iter()),
                    |(ergebnis, nicht_verwendet), arg| match ergebnis {
                        Fehler(mut fehler0) => match arg.parse(args_vec.clone().into_iter()) {
                            (Fehler(fehler1), nicht_verwendet1) => {
                                fehler0.extend(fehler1);
                                let von_keinem_verwendet = nicht_verwendet
                                    .into_iter()
                                    .filter(|os_string| nicht_verwendet1.contains(os_string))
                                    .collect();
                                (Fehler(fehler0), von_keinem_verwendet)
                            },
                            end_ergebnis => end_ergebnis,
                        },
                        Wert(_) | FrühesBeenden(_) => (ergebnis, nicht_verwendet),
                    },
                )
            },
        }
    }
}

impl<'t, T, Bool, Parse, Anzeige, K> Argumente<'t, T, Bool, Parse, Anzeige, K> {
    /// Erzeuge die Anzeige für die Syntax des Arguments und den zugehörigen Hilfetext.
    ///
    /// ## English
    /// Create the Message for the syntax of the arguments and the corresponding help text.
    #[inline]
    #[must_use]
    // panic when a programming error occurs
    #[allow(clippy::missing_panics_doc)]
    pub fn erzeuge_hilfe_text<H: ErzeugeHilfeText, Fehler>(
        &self,
        meta_standard: &str,
        meta_erlaubte_werte: &str,
    ) -> NonEmpty<hilfe::Alternativen<'_>>
    where
        Anzeige: Fn(&T) -> String,
        K: Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige>,
    {
        match self {
            Argumente::EinzelArgument(arg) => {
                nonempty![hilfe::Alternativen::EinzelArgument(
                    arg.erzeuge_hilfe_text(meta_standard, meta_erlaubte_werte)
                )]
            },
            Argumente::Kombiniere(kombiniere) => {
                kombiniere.erzeuge_hilfe_text::<H>(meta_standard, meta_erlaubte_werte)
            },
            Argumente::Alternativen(alternativen) => {
                // TODO use alternativen.as_ref().flat_map(...), coming in nonempty > 0.10.0
                NonEmpty::collect(alternativen.iter().map(|arg| {
                    hilfe::Alternativen::Alternativen(Box::new(
                        arg.erzeuge_hilfe_text::<H, Fehler>(meta_standard, meta_erlaubte_werte),
                    ))
                }))
                .expect("NonEmpty::map(...) hat mindestens ein Argument!")
            },
        }
    }
}

/// [`FrühesBeenden`]-Argument für den Hilfe-Text.
type HilfeFrühesBeenden<'t> = Argumente<
    't,
    (),
    fn(bool) -> (),
    fn(&OsStr) -> Result<(), ParseFehler<Void>>,
    fn(&()) -> String,
    Void,
>;
/// Kombinieren des Arguments mit mit dem [`FrühesBeenden`]-Arguments für den Hilfe-Text.
type KombiniereHilfe<'t, T, Bool, Parse, Anzeige, K> =
    (fn(T, ()) -> T, Argumente<'t, T, Bool, Parse, Anzeige, K>, HilfeFrühesBeenden<'t>);
/// Argument mit zugehörigem [`FrühesBeenden`]-Argument für den Hilfe-Text.
type ArgumenteMitHilfe<'t, T, Bool, Parse, Anzeige, K> =
    Argumente<'t, T, Bool, Parse, Anzeige, KombiniereHilfe<'t, T, Bool, Parse, Anzeige, K>>;

impl<'t, T, Bool, Parse, Anzeige, K> Argumente<'t, T, Bool, Parse, Anzeige, K> {
    /// Füge eine [`FrühesBeenden`]-Flag hinzu, wodurch die Programm-Version anzeigt wird.
    ///
    /// ## English
    /// Add an [`EarlyExit`](crate::frühes_beenden::EarlyExit`)-flag, showing the program version.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn mit_version_frühes_beenden(
        self,
        eigene_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_version: &str,
    ) -> ArgumenteMitHilfe<'t, T, Bool, Parse, Anzeige, K> {
        let name_und_version = format!("{programm_name} {programm_version}");
        let frühes_beenden = FrühesBeenden {
            beschreibung: eigene_beschreibung,
            nachricht: Cow::Owned(name_und_version),
        };
        let kombiniere: KombiniereHilfe<'_, T, Bool, Parse, Anzeige, K> =
            (|wert: T, ()| wert, self, Argumente::from(EinzelArgument::from(frühes_beenden)));
        Argumente::Kombiniere(kombiniere)
    }

    /// Füge eine [`FrühesBeenden`]-Flag hinzu, wodurch der Hilfe-Text für alle Argumente anzeigt wird.
    ///
    /// ### Panics
    /// Wenn die Syntax-Beschreibung (inklusive normalem + Alternativen-Präfix) für ein Argument
    /// länger als [`usize::MAX`] ist.
    ///
    /// ## English
    /// Add an [`EarlyExit`](crate::frühes_beenden::EarlyExit`)-Flag, showing the help text for all arguments.
    ///
    /// ### Panics
    /// If the syntax-description (including normal + alternativ prefixes) for an argument exceeds [`usize::MAX`].
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn mit_hilfe_frühes_beenden<H, Fehler>(
        self,
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
    ) -> ArgumenteMitHilfe<'t, T, Bool, Parse, Anzeige, K>
    where
        H: ErzeugeHilfeText,
        Anzeige: Fn(&T) -> String,
        K: Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige>,
        Bool: Fn(bool) -> T,
    {
        let mut hilfen = self.erzeuge_hilfe_text::<H, Fehler>(meta_standard, meta_erlaubte_werte);
        let dummy = Cow::Borrowed("");
        let mut frühes_beenden =
            FrühesBeenden { beschreibung: eigene_beschreibung, nachricht: dummy };
        hilfen.push(hilfe::Alternativen::EinzelArgument(frühes_beenden.erzeuge_hilfe_text()));
        let hilfen = hilfen;
        let max_syntax_breite = max_syntax_breite(&hilfen, meta_alternative_präfix);
        let current_exe = env::current_exe().ok();
        let exe_name = current_exe
            .as_deref()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
            .unwrap_or(programm_name);
        let mut name = String::from(programm_name);
        if let Some(version) = programm_version {
            name.push(' ');
            name.push_str(version);
        }
        let programm_beschreibung = programm_beschreibung
            .map(|beschreibung| format!("\n{beschreibung}"))
            .unwrap_or_default();
        let mut hilfe_text = format!(
            "{name}{programm_beschreibung}\n\n{exe_name} [{meta_optionen}]\n\n{meta_optionen}:\n"
        );
        for hilfe in hilfen {
            schreibe_argument_oder_alternativen(
                &mut hilfe_text,
                Cow::Borrowed(meta_syntax_präfix),
                #[allow(clippy::arithmetic_side_effects)]
                {
                    meta_syntax_präfix.len() + max_syntax_breite + 1
                },
                meta_syntax_padding,
                &hilfe,
                meta_alternative_präfix,
                meta_alternative_trennzeichen,
            );
        }
        frühes_beenden.nachricht = Cow::Owned(hilfe_text);
        let kombiniere: KombiniereHilfe<'_, T, Bool, Parse, Anzeige, K> =
            (|wert: T, ()| wert, self, Argumente::from(EinzelArgument::from(frühes_beenden)));
        Argumente::Kombiniere(kombiniere)
    }

    /// Variante von [`mit_hilfe_frühes_beenden`](Self::mit_hilfe_frühes_beenden),
    /// basierend auf einer [`Sprache`].
    ///
    /// ## English
    /// Variant of [`mit_hilfe_frühes_beenden`](Self::mit_hilfe_frühes_beenden),
    /// based on a [`Language`](crate::sprache::Language).
    #[inline]
    pub fn mit_hilfe_frühes_beenden_mit_sprache<H, Fehler>(
        self,
        eigene_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: Option<&str>,
        sprache: Sprache,
    ) -> ArgumenteMitHilfe<'t, T, Bool, Parse, Anzeige, K>
    where
        H: ErzeugeHilfeText,
        Anzeige: Fn(&T) -> String,
        K: Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige>,
        Bool: Fn(bool) -> T,
    {
        self.mit_hilfe_frühes_beenden::<H, Fehler>(
            eigene_beschreibung,
            programm_name,
            programm_beschreibung,
            programm_version,
            sprache.standard,
            sprache.erlaubte_werte,
            sprache.optionen,
            sprache.syntax_präfix,
            sprache.syntax_padding,
            sprache.alternative_präfix,
            sprache.alternative_trennzeichen,
        )
    }

    /// Füge [`FrühesBeenden`]-Flags hinzu, wodurch die Programm-Version,
    /// bzw. der Hilfe-Text für alle Argumente anzeigt wird.
    ///
    /// ### Panics
    /// Wenn die Syntax-Beschreibung (inklusive normalem + Alternativen-Präfix) für ein Argument
    /// länger als [`usize::MAX`] ist.
    ///
    /// ## English
    /// Add [`EarlyExit`](crate::frühes_beenden::EarlyExit`)-Flags, showing the program version,
    /// or the help text for all arguments.
    ///
    /// ### Panics
    /// If the syntax-description (including normal + alternativ prefixes) for an argument exceeds [`usize::MAX`].
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn mit_hilfe_und_version_frühes_beenden<H, Fehler>(
        self,
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
    ) -> ArgumenteMitHilfe<
        't,
        T,
        Bool,
        Parse,
        Anzeige,
        KombiniereHilfe<'t, T, Bool, Parse, Anzeige, K>,
    >
    where
        H: ErzeugeHilfeText,
        Anzeige: Fn(&T) -> String,
        K: Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige>,
        Bool: Fn(bool) -> T,
        Parse: Fn(&OsStr) -> Result<T, ParseFehler<Fehler>>,
        Fehler: From<Void>,
    {
        self.mit_version_frühes_beenden(version_beschreibung, programm_name, programm_version)
            .mit_hilfe_frühes_beenden::<H, Fehler>(
                hilfe_beschreibung,
                programm_name,
                programm_beschreibung,
                Some(programm_version),
                meta_standard,
                meta_erlaubte_werte,
                meta_optionen,
                meta_syntax_präfix,
                meta_syntax_padding,
                meta_alternative_präfix,
                meta_alternative_trennzeichen,
            )
    }

    /// Variante von [`mit_hilfe_und_version_frühes_beenden`](Argumente::mit_hilfe_und_version_frühes_beenden).
    ///
    /// ## English
    /// Variant of [`mit_hilfe_und_version_frühes_beenden`](Argumente::mit_hilfe_und_version_frühes_beenden).
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn mit_hilfe_und_version_frühes_beenden_mit_sprache<H, Fehler>(
        self,
        version_beschreibung: Beschreibung<'t, Void>,
        hilfe_beschreibung: Beschreibung<'t, Void>,
        programm_name: &str,
        programm_beschreibung: Option<&str>,
        programm_version: &str,
        sprache: Sprache,
    ) -> ArgumenteMitHilfe<
        't,
        T,
        Bool,
        Parse,
        Anzeige,
        KombiniereHilfe<'t, T, Bool, Parse, Anzeige, K>,
    >
    where
        H: ErzeugeHilfeText,
        Anzeige: Fn(&T) -> String,
        K: Kombiniere<'t, T, Bool, Parse, Fehler, Anzeige>,
        Bool: Fn(bool) -> T,
        Parse: Fn(&OsStr) -> Result<T, ParseFehler<Fehler>>,
        Fehler: From<Void>,
    {
        self.mit_hilfe_und_version_frühes_beenden::<H, Fehler>(
            version_beschreibung,
            hilfe_beschreibung,
            programm_name,
            programm_beschreibung,
            programm_version,
            sprache.standard,
            sprache.erlaubte_werte,
            sprache.optionen,
            sprache.syntax_präfix,
            sprache.syntax_padding,
            sprache.alternative_präfix,
            sprache.alternative_trennzeichen,
        )
    }
}

/// Berechne die maximale Breite für die Syntax eines Argumentes.
///
/// Hilfsfunktion für [`Argumente::mit_hilfe_frühes_beenden`]
///
/// ## Panics
/// Programmierfehler, wenn `NonEmpty::iter().map(...)` kein Element hat.
fn max_syntax_breite(
    hilfen: &NonEmpty<hilfe::Alternativen<'_>>,
    alternative_präfix: &str,
) -> usize {
    hilfen
        .iter()
        .map(|arg| match arg {
            hilfe::Alternativen::EinzelArgument(arg) => arg.syntax.len(),
            hilfe::Alternativen::Alternativen(alternativen) => {
                #[allow(clippy::arithmetic_side_effects)]
                {
                    alternative_präfix.len() + max_syntax_breite(alternativen, alternative_präfix)
                }
            },
        })
        .max()
        .expect("NonEmpty")
}

/// Schreibe den Hilfetext für den aktuellen Eintrag oder alle Alternativen.
///
/// Hilfsfunktion für [`Argumente::mit_hilfe_frühes_beenden`]
///
/// ## Panics
/// If `max_syntax_breite < aktueller_präfix.len() + syntax.len()` for any entry.
/// If `max_syntax_breite < aktueller_präfix.len()` for any entry.
fn schreibe_argument_oder_alternativen(
    string: &mut String,
    aktueller_präfix: Cow<'_, str>,
    max_syntax_breite: usize,
    syntax_padding: char,
    eintrag: &hilfe::Alternativen<'_>,
    alternative_präfix: &str,
    alternative_trennzeichen: char,
) {
    match eintrag {
        hilfe::Alternativen::EinzelArgument(arg) => {
            let Hilfe { syntax, hilfe } = arg;
            string.push_str(&aktueller_präfix);
            string.push_str(syntax);
            #[allow(clippy::arithmetic_side_effects)]
            let padding = max_syntax_breite - aktueller_präfix.len() - syntax.len();
            let mut buffer: [u8; 4] = [0; 4];
            let padding_string = syntax_padding.encode_utf8(&mut buffer).repeat(padding);
            string.push_str(&padding_string);
            if let Some(hilfe) = hilfe {
                string.push_str(hilfe);
            }
            string.push('\n');
        },
        hilfe::Alternativen::Alternativen(alternativen) => {
            #[allow(clippy::arithmetic_side_effects)]
            let trennzeile_breite = max_syntax_breite - aktueller_präfix.len();
            let mut buffer: [u8; 4] = [0; 4];
            let trennzeile = format!(
                "{aktueller_präfix}{}",
                alternative_trennzeichen.encode_utf8(&mut buffer).repeat(trennzeile_breite)
            );
            let mut neuer_präfix = aktueller_präfix.into_owned();
            neuer_präfix.push_str(alternative_präfix);
            let mut first = true;
            for alternative in alternativen.iter() {
                if first {
                    first = false;
                } else {
                    string.push_str(&trennzeile);
                    string.push('\n');
                }
                schreibe_argument_oder_alternativen(
                    string,
                    Cow::Borrowed(&neuer_präfix),
                    max_syntax_breite,
                    syntax_padding,
                    alternative,
                    alternative_präfix,
                    alternative_trennzeichen,
                );
            }
        },
    }
}
