use crate::error::DltError;
use crate::message::DltMessage;
use bytes::Buf;

const MIN_MESSAGE_LENGTH: usize = 16 /*Storage Header*/ + 4 /*Smallest Standard Header, no Extended Header */;
#[derive(Debug)]
pub struct DltFile<'a> {
    buf: &'a [u8],
}

impl<'a> DltFile<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }
}

impl<'a> Iterator for DltFile<'a> {
    type Item = Result<DltMessage<'a>, DltError>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.buf.has_remaining() {
            None
        } else {
            match DltMessage::from_slice(self.buf) {
                Ok(message) => {
                    self.buf.advance(message.len());
                    Some(Ok(message))
                }
                // in case of an error we first try to advance the buffer to the start of the next message
                // and then simply yield the error
                Err(err) => {
                    if let Some(advance_by) = err.advance_by {
                        self.buf.advance(advance_by);
                        Some(Err(err))
                    } else {
                        // we use this instead of `memchr::memmem::find`
                        // because the malformed message could still have a valid `DLT\x01`
                        // pattern and we would then try to parse the malformed message forever
                        match memchr::memmem::find_iter(self.buf, b"DLT\x01")
                            .find(|&index| index > 0)
                        {
                            Some(start) => {
                                self.buf.advance(start);
                                Some(Err(err))
                            }
                            None => None,
                        }
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.buf.remaining() / MIN_MESSAGE_LENGTH))
    }
}
