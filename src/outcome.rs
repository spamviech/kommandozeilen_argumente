//! Results and errors produced while parsing command-line arguments.

mod result;
mod single_argument;

#[cfg(feature = "german")]
mod german;

#[cfg(feature = "german")]
pub use self::german::{
    AlternativenFehler, Ergebnis, Fehler, KommentierterParseFehler, ParseFehler, ZwischenErgebnis,
};
pub use self::{
    result::{
        AlternativeErrors, AnnotatedParseError, Error, IntermediateResult, ParseError, Result,
    },
    single_argument::{ParseFollowingArgFn, ParseNextArgFn, SingleArgResult},
};
