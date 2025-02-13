use bytes::Bytes;

use super::{
    header::{ExtendedHeader, StandardHeader},
    payload::Payload,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Message {
    pub(crate) bytes: Bytes,
    pub(crate) standard_header: StandardHeader,
    pub(crate) extended_header: Option<ExtendedHeader>,
    pub(crate) payload: Payload,
}

impl Message {
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes.to_vec()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}
