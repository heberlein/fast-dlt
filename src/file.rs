use fallible_iterator::FallibleIterator;

use crate::{DltError, DltFilter, DltMessage, Result};

#[derive(Debug)]
pub struct DltFile<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> DltFile<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    pub fn filtered(buf: &'a [u8], filter: DltFilter) -> Self {
        todo!()
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
}
