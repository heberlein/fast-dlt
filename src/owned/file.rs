use bytes::Bytes;

use super::message::Message;

pub struct File {
    bytes: Bytes,
}

impl File {
    pub fn from_owner<T>(owner: T) -> Self
    where
        T: AsRef<[u8]> + Send + 'static,
    {
        Self {
            bytes: Bytes::from_owner(owner),
        }
    }
}

impl Iterator for File {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        let start = memchr::memmem::find(self.bytes.as_ref(), b"DLT\x01")?;
        todo!()
    }
}
