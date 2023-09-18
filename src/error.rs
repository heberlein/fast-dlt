use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Recoverable Error at byte {index}: {cause}")]
    Recoverable {
        message_len: usize,
        index: usize,
        cause: RecoverableError,
    },

    #[error("Fatal Error: {0}")]
    Fatal(#[from] FatalError),
}

impl From<std::array::TryFromSliceError> for ParseError {
    fn from(value: std::array::TryFromSliceError) -> Self {
        Self::Fatal(value.into())
    }
}

impl From<simdutf8::basic::Utf8Error> for ParseError {
    fn from(value: simdutf8::basic::Utf8Error) -> Self {
        Self::Fatal(value.into())
    }
}

impl ParseError {
    pub fn unimplemented_arg(index: usize, len: usize, arg: impl Into<String>) -> Self {
        Self::Recoverable {
            message_len: len,
            index: index,
            cause: RecoverableError::UnimplementedArgumentType(arg.into()),
        }
    }
}

#[derive(Debug, Error)]
pub enum RecoverableError {
    #[error("Arguments of type {0} are not (yet) supported!")]
    UnimplementedArgumentType(String),
}

#[derive(Debug, Error)]
pub enum FatalError {
    #[error("Missing DLT pattern!")]
    MissingDltPattern,
    #[error("Not enough data!")]
    NotEnoughData,
    #[error("Invalid UTF-8 string!")]
    BadUTF8(#[from] simdutf8::basic::Utf8Error),
    #[error("Failed to convert slice!")]
    BadSliceConvert(#[from] std::array::TryFromSliceError),
}

pub type Result<T> = std::result::Result<T, ParseError>;
