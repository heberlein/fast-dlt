#![allow(unused)]

use crate::{
    ExtendedHeader, NonVerbosePayload, Payload, Result, StandardHeader, StorageHeader,
    VerbosePayload,
};

#[derive(Debug)]
pub struct DltMessage<'a> {
    pub storage_header: StorageHeader<'a>,
    pub standard_header: StandardHeader<'a>,
    pub extended_header: Option<ExtendedHeader<'a>>,
    pub payload: Payload<'a>,
}

impl<'a> DltMessage<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self> {
        let mut offset = 0;
        let storage_header = StorageHeader::new(buf)?;
        offset += storage_header.num_bytes();
        let standard_header = StandardHeader::new(&buf[offset..])?;
        offset += standard_header.num_bytes();
        let extended_header = if standard_header.use_extended_header() {
            let extended_header = ExtendedHeader::new(&buf[offset..])?;
            offset += extended_header.num_bytes();
            Some(extended_header)
        } else {
            None
        };
        let payload_end = standard_header.length as usize + storage_header.num_bytes();
        let payload = if extended_header.as_ref().map_or(false, |hdr| hdr.verbose()) {
            Payload::Verbose(VerbosePayload::new(&buf[offset..payload_end]))
        } else {
            Payload::NonVerbose(NonVerbosePayload::new(&buf[offset..payload_end])?)
        };

        let msg = DltMessage {
            storage_header,
            standard_header,
            extended_header,
            payload,
        };

        Ok(msg)
    }

    pub fn num_bytes(&self) -> usize {
        self.storage_header.num_bytes() + self.standard_header.length as usize
    }
}
