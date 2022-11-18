#![allow(unused)]

use crate::{ExtendedHeader, Payload, Result, StandardHeader, StorageHeader};

#[derive(Debug)]
pub struct DltMessage<'a> {
    storage_header: StorageHeader<'a>,
    standard_header: StandardHeader<'a>,
    extended_header: Option<ExtendedHeader<'a>>,
    payload: (),
}

impl<'a> DltMessage<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self> {
        let storage_header = StorageHeader::new(buf)?;
        let standard_header = StandardHeader::new(&buf[storage_header.num_bytes()..])?;
        let extended_header = if standard_header.use_extended_header() {
            Some(ExtendedHeader::new(
                &buf[storage_header.num_bytes() + standard_header.num_bytes()..],
            )?)
        } else {
            None
        };
        Ok(DltMessage {
            storage_header,
            standard_header,
            extended_header,
            payload: (),
        })
    }

    pub fn num_bytes(&self) -> usize {
        self.storage_header.num_bytes() + self.standard_header.length as usize
    }
}
