//! Continuation state while parsing a single command-line argument.

use std::{
    ffi::OsStr,
    fmt::{self, Debug},
    result::Result,
};

use super::ParseError;
use crate::description::AdjustedMergedShortNames;

/// Type alias for [`SingeArgResult`]. Parses the directly following argument as the value.
pub type ParseNextArgFn<'t, T, E> = Box<dyn 't + Fn(&OsStr) -> SingleArgResult<'t, T, E>>;

/// Type alias for [`SingeArgResult`]. Parses a following argument in this parsing context.
pub type ParseFollowingArgFn<'t, T, E> =
    Box<dyn 't + Fn(&OsStr) -> Option<SingleArgResult<'t, T, E>>>;

/// Result of parsing one argument.
pub enum SingleArgResult<'t, T, E> {
    /// The argument was completely parsed.
    FullParse {
        /// The current argument after removing the parsed short name.
        adjusted_arg: Option<AdjustedMergedShortNames>,
        /// The parsing result.
        result: Result<T, ParseError<E>>,
    },
    /// Only the argument name was parsed; the next argument is its value.
    NameOnly {
        /// The current argument after removing the parsed short name.
        adjusted_arg: Option<AdjustedMergedShortNames>,
        /// Parses the directly following argument as the value.
        parse_next_arg: ParseNextArgFn<'t, T, E>,
    },
    /// More input is required.
    IncompleteParse {
        /// The current argument after removing the parsed short name.
        adjusted_arg: Option<AdjustedMergedShortNames>,
        /// Parses a following argument in this parsing context.
        parse_following_arg: ParseFollowingArgFn<'t, T, E>,
    },
}

impl<T: Debug, E: Debug> Debug for SingleArgResult<'_, T, E> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FullParse { adjusted_arg, result } => {
                formatter.debug_tuple("FullParse").field(adjusted_arg).field(result).finish()
            },
            Self::NameOnly { adjusted_arg, .. } => formatter
                .debug_struct("NameOnly")
                .field("adjusted_arg", adjusted_arg)
                .field("parse_next_arg", &"<closure>")
                .finish(),
            Self::IncompleteParse { adjusted_arg, .. } => formatter
                .debug_struct("IncompleteParse")
                .field("adjusted_arg", adjusted_arg)
                .field("parse_following_arg", &"<closure>")
                .finish(),
        }
    }
}

impl<'t, T: 't, E: 't> SingleArgResult<'t, T, E> {
    /// Converts a successful value.
    #[inline]
    pub fn convert<S>(self, mapper: impl 't + Clone + Fn(T) -> S) -> SingleArgResult<'t, S, E> {
        match self {
            Self::FullParse { adjusted_arg, result } => {
                SingleArgResult::FullParse { adjusted_arg, result: result.map(mapper) }
            },
            Self::NameOnly { adjusted_arg, parse_next_arg } => SingleArgResult::NameOnly {
                adjusted_arg,
                parse_next_arg: Box::new(move |arg| parse_next_arg(arg).convert(mapper.clone())),
            },
            Self::IncompleteParse { adjusted_arg, parse_following_arg } => {
                SingleArgResult::IncompleteParse {
                    adjusted_arg,
                    parse_following_arg: Box::new(move |arg| {
                        parse_following_arg(arg).map(|value| value.convert(mapper.clone()))
                    }),
                }
            },
        }
    }
}
