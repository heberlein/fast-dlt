use thiserror::Error;

#[derive(Debug, Error)]
pub enum DltError {
    #[error("Missing DLT pattern!")]
    MissingDltPattern,
    #[error("Not enough data!")]
    NotEnoughData,
    #[error("Invalid UTF-8 string!")]
    BadUTF8(#[from] simdutf8::basic::Utf8Error),
    #[error("Failed to convert slice!")]
    BadSliceConvert(#[from] std::array::TryFromSliceError),
}

pub type Result<T> = std::result::Result<T, DltError>;
