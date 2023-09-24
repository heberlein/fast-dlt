use fallible_iterator::FallibleIterator;

use crate::error::DltError;
use crate::message::DltMessage;

const MIN_MESSAGE_LENGTH: usize = 16 /*Storage Header*/ + 4 /*Smallest Standard Header, no Extended Header */;
#[derive(Debug)]
pub struct DltFile<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> DltFile<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, offset: 0 }
    }
}

impl<'a> IntoIterator for DltFile<'a> {
    type Item = Result<DltMessage<'a>, DltError>;

    type IntoIter = fallible_iterator::Iterator<Self>;

    fn into_iter(self) -> Self::IntoIter {
        self.iterator()
    }
}

impl<'a> FallibleIterator for DltFile<'a> {
    type Item = DltMessage<'a>;
    type Error = DltError;

    fn next(&mut self) -> Result<Option<DltMessage<'a>>, DltError> {
        if self.offset >= self.buf.len() {
            Ok(None)
        } else {
            let message = DltMessage::parse_at(self.offset, self.buf)?;
            self.offset += message.len();
            Ok(Some(message))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.buf[self.offset..].len() / MIN_MESSAGE_LENGTH))
    }
}
