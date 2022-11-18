use crate::{DltError, Result};
use std::str;

#[derive(Debug, PartialEq, Eq)]
pub struct StorageHeader<'a> {
    seconds: u32,
    microseconds: i32,
    ecu_id: &'a str,
}
impl<'a> StorageHeader<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self> {
        // TODO: check if buf is long enough  once, return error if not
        if &buf[..4] != b"DLT\x01" {
            return Err(DltError::MissingDltPattern);
        }
        let seconds = u32::from_le_bytes(buf[4..8].try_into()?);
        let microseconds = i32::from_le_bytes(buf[8..12].try_into()?);
        let ecu_id = str::from_utf8(&buf[12..16])?;
        Ok(Self {
            seconds,
            microseconds,
            ecu_id,
        })
    }

    pub fn num_bytes(&self) -> usize {
        4 /*DLT pattern*/ 
        + 4 /*seconds*/ 
        + 4 /*microseconds*/ 
        + 4 /*ecu id*/
    }
}

#[repr(u8)]
enum StdHeaderMask {
    UseExtendedHeader = 0b00000001,
    MsbFirst = 0b00000010,
    WithEcuId = 0b00000100,
    WithSessionId = 0b00001000,
    WithTimestamp = 0b00010000,
    VersionNumber = 0b11100000,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StandardHeader<'a> {
    header_type: u8,
    message_counter: u8,
    pub length: u16,
    ecu_id: Option<&'a str>,
    session_id: Option<u32>,
    timestamp: Option<u32>,
}

impl<'a> StandardHeader<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self> {
        // TODO: check if buf is long enough  once, return error if not
        let header_type = buf[0];
        let message_counter = buf[1];

        // use mem::transmute to convert to [u8;2]? only if todo is implemented
        let length = u16::from_be_bytes(buf[2..4].try_into()?);

        let mut optionals_offset = 0;
        let ecu_id = if header_type & StdHeaderMask::WithEcuId as u8 == 0 {
            None
        } else {
            optionals_offset += 4;
            Some(str::from_utf8(
                &buf[optionals_offset..optionals_offset + 4],
            )?)
        };
        let session_id = if header_type & StdHeaderMask::WithSessionId as u8 == 0 {
            None
        } else {
            optionals_offset += 4;
            Some(u32::from_be_bytes(
                buf[optionals_offset..optionals_offset + 4].try_into()?,
            ))
        };
        let timestamp = if header_type & StdHeaderMask::WithTimestamp as u8 == 0 {
            None
        } else {
            optionals_offset += 4;
            Some(u32::from_be_bytes(
                buf[optionals_offset..optionals_offset + 4].try_into()?,
            ))
        };

        Ok(Self {
            header_type,
            message_counter,
            length,
            ecu_id,
            session_id,
            timestamp,
        })
    }

    pub fn use_extended_header(&self) -> bool {
        self.header_type & StdHeaderMask::UseExtendedHeader as u8 != 0
    }
    pub fn msb_first(&self) -> bool {
        self.header_type & StdHeaderMask::MsbFirst as u8 != 0
    }
    pub fn with_ecu_id(&self) -> bool {
        self.header_type & StdHeaderMask::WithEcuId as u8 != 0
    }
    pub fn with_session_id(&self) -> bool {
        self.header_type & StdHeaderMask::WithSessionId as u8 != 0
    }
    pub fn with_timestamp(&self) -> bool {
        self.header_type & StdHeaderMask::WithTimestamp as u8 != 0
    }
    pub fn version(&self) -> u8 {
        self.header_type & StdHeaderMask::VersionNumber as u8
    }

    pub fn num_bytes(&self) -> usize {
        1 /*header type*/
        + 1 /*message_counter */
        + 2 /*length */
        + self.ecu_id.is_some() as usize * 4
        + self.session_id.is_some() as usize * 4
        + self.timestamp.is_some() as usize * 4
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExtendedHeader<'a> {
    message_info: u8,
    number_of_arguments: u8,
    application_id: &'a str,
    context_id: &'a str,
}

impl<'a> ExtendedHeader<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self> {
        let message_info = buf[0];
        let number_of_arguments = buf[1];
        let application_id = str::from_utf8(&buf[2..6])?;
        let context_id = str::from_utf8(&buf[6..10])?;
        Ok(Self {
            message_info,
            number_of_arguments,
            application_id,
            context_id,
        })
    }

    pub fn num_bytes(&self) -> usize {
        1 /*message_info*/
        + 1 /*number_of_arguments*/
        + 4 /*application_id*/
        + 4 /*context id*/
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn storage_header() {
        let bytes = b"DLT\x01\r\x00\x00\x00%\x00\x00\x00TEST";
        let header = StorageHeader::new(bytes).unwrap();
        assert_eq!(
            header,
            StorageHeader {
                seconds: 13,
                microseconds: 37,
                ecu_id: "TEST"
            }
        );
    }

    #[test]
    fn standard_header() {
        let bytes = b"|\n\x00dTEST\x00\x00\x00\x03\x00\x00\x059";
        let header = StandardHeader::new(bytes).unwrap();
        assert_eq!(
            header,
            StandardHeader {
                header_type: 124,
                message_counter: 10,
                length: 100,
                ecu_id: Some("TEST"),
                session_id: Some(3),
                timestamp: Some(1337)
            }
        )
    }
    #[test]
    fn extended_header() {
        let bytes = b"@\x07APPLCONT";
        let header = ExtendedHeader::new(bytes).unwrap();
        assert_eq!(
            header,
            ExtendedHeader {
                message_info: 64,
                number_of_arguments: 7,
                application_id: "APPL",
                context_id: "CONT"
            }
        )
    }
}
