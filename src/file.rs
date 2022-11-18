use crate::{DltFilter, DltMessage, Result};

#[derive(Debug)]
pub struct DltFile<'a> {
    pub messages: Vec<DltMessage<'a>>,
}

impl<'a> DltFile<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self> {
        let mut offset = 0;
        let mut messages = vec![];
        loop {
            if offset >= buf.len() {
                break;
            }
            let message = DltMessage::new(&buf[offset..])?;
            offset += message.num_bytes();
            messages.push(message);
        }

        Ok(DltFile { messages })
    }

    pub fn filtered(buf: &'a [u8], filter: DltFilter) -> Self {
        todo!()
    }
}
