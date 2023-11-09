use std::{error::Error, fmt::Display};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Missing DLT pattern!")]
    MissingDltPattern,

    #[error("Not enough data!")]
    NotEnoughData { needed: usize, available: usize },

    #[error("Arguments of type {0} are not (yet) supported")]
    UnimplementedArgumentType(&'static str),

    #[error("{0} unsupported")]
    Unsupported(&'static str),

    #[error("No such argument type: {0}")]
    UnknownArgumentType(u32),

    #[error("Invalid UTF-8 string!")]
    BadUTF8(#[from] simdutf8::basic::Utf8Error),
}

#[derive(Debug)]
pub struct DltError {
    pub(crate) advance_by: Option<usize>,
    pub(crate) source: ParseError,
}

impl Display for DltError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "DLT Error: {}", self.source)
    }
}

impl Error for DltError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl<T: Into<ParseError>> From<T> for DltError {
    fn from(value: T) -> Self {
        DltError {
            advance_by: None,
            source: value.into(),
        }
    }
}
