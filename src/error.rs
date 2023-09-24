use thiserror::Error;

#[derive(Debug, Error)]
pub enum DltError {
    #[error("Recoverable error at byte {index}: {cause}")]
    Recoverable {
        /// The length of the faulty message
        message_len: u16,
        index: usize,
        #[source]
        cause: ParseError,
    },

    #[error("Fatal error at byte {index}: {cause}")]
    Fatal {
        index: usize,
        #[source]
        cause: ParseError,
    },
}

impl DltError {
    #[inline(always)]
    pub fn recoverable_at(index: usize, message_len: u16, cause: impl Into<ParseError>) -> Self {
        Self::Recoverable {
            message_len,
            index,
            cause: cause.into(),
        }
    }

    #[inline(always)]
    pub fn fatal_at(index: usize, cause: impl Into<ParseError>) -> Self {
        Self::Fatal {
            index,
            cause: cause.into(),
        }
    }
}

impl DltError {
    pub fn unimplemented_arg(index: usize, len: u16, arg: &'static str) -> Self {
        Self::Recoverable {
            message_len: len,
            index,
            cause: ParseError::UnimplementedArgumentType(arg),
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Missing DLT pattern!")]
    MissingDltPattern,

    #[error("Failed to parse extended header")]
    ExtendedHeader,

    #[error("input buffer is too short: {index} > {length}")]
    BufferTooShort { index: usize, length: usize },

    #[error("Not enough data!")]
    NotEnoughData { needed: usize, available: usize },

    #[error("Arguments of type {0} are not (yet) supported")]
    UnimplementedArgumentType(&'static str),

    #[error("Invalid UTF-8 string!")]
    BadUTF8(#[from] simdutf8::basic::Utf8Error),

    #[error("Failed to convert slice!")]
    BadSliceConvert(#[from] std::array::TryFromSliceError),
}
