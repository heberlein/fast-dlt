use thiserror::Error;

use crate::ArgumentKind;

#[derive(Debug, Error)]
pub enum DltError {
    #[error("Missing DLT pattern!")]
    MissingDltPattern,
    #[error("Invalid UTF-8 string!")]
    BadUTF8(#[from] std::str::Utf8Error),
    #[error("Failed to convert slice!")]
    BadSliceConvert(#[from] std::array::TryFromSliceError),
    #[error("Mismatched Argument kind: expected {0}, got {1}")]
    KindMismatch(ArgumentKind, ArgumentKind),
}

pub type Result<T> = std::result::Result<T, DltError>;
