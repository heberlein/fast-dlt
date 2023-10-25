use crate::error::{DltError, ParseError};
use crate::message::DltMessage;

const MIN_MESSAGE_LENGTH: usize = 16 /*Storage Header*/ + 4 /*Smallest Standard Header, no Extended Header */;
#[derive(Debug)]
pub struct DltFile<'a> {
    buf: &'a [u8],
    offset: usize,
    fatal: bool,
}

impl<'a> DltFile<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            offset: 0,
            fatal: false,
        }
    }

    /// silently skip over recoverable errors
    pub fn skip_recoverable_errors(
        self,
    ) -> impl Iterator<Item = Result<DltMessage<'a>, ParseError>> {
        self.filter_map(|msg| match msg {
            Ok(msg) => Some(Ok(msg)),
            Err(DltError::Recoverable { .. }) => None,
            Err(DltError::Fatal { cause, .. }) => Some(Err(cause)),
        })
    }
}

impl<'a> Iterator for DltFile<'a> {
    type Item = Result<DltMessage<'a>, DltError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.fatal || self.offset >= self.buf.len() {
            None
        } else {
            match DltMessage::parse_at(self.offset, self.buf) {
                Ok(message) => {
                    self.offset += message.len();
                    Some(Ok(message))
                }
                Err(DltError::Recoverable {
                    message_len,
                    index,
                    cause,
                }) => {
                    self.offset += message_len as usize;
                    Some(Err(DltError::Recoverable {
                        message_len,
                        index,
                        cause,
                    }))
                }
                Err(err) => {
                    self.fatal = true;
                    Some(Err(err))
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.buf[self.offset..].len() / MIN_MESSAGE_LENGTH))
    }
}
