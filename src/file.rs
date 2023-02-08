use fallible_iterator::{FallibleIterator, Iterator};

use crate::{DltError, DltMessage, Result};

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
    type Item = Result<DltMessage<'a>>;

    type IntoIter = fallible_iterator::Iterator<Self>;

    fn into_iter(self) -> Self::IntoIter {
        self.iterator()
    }
}

impl<'a> FallibleIterator for DltFile<'a> {
    type Item = DltMessage<'a>;
    type Error = DltError;

    fn next(&mut self) -> Result<Option<DltMessage<'a>>> {
        if self.offset >= self.buf.len() {
            Ok(None)
        } else {
            let message = DltMessage::new(&self.buf[self.offset..])?;
            self.offset += message.num_bytes();
            Ok(Some(message))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.buf[self.offset..].len() / MIN_MESSAGE_LENGTH))
    }
}

#[allow(unused)]
struct AllMessages {}
#[allow(unused)]
struct WithEcuId {}
#[allow(unused)]
struct WithAppId {}
