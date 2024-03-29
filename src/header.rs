use crate::{error::{DltError, ParseError}, get_str, get_slice};
use std::str;
use bytes::Buf; 
use simdutf8::basic::from_utf8;

#[derive(Debug, PartialEq, Eq)]
pub struct StorageHeader<'a> {
    pub seconds: u32,
    pub microseconds: i32,
    pub ecu_id: &'a str,
}
impl<'a> StorageHeader<'a> {

    const MIN_LENGTH: usize = 16;

    pub fn from_slice(mut buf: &'a [u8]) -> Result<Self, ParseError> {

        if buf.remaining() < Self::MIN_LENGTH {
            return Err(ParseError::NotEnoughData{needed: Self::MIN_LENGTH, available: buf.len()})
        }

        // check for DLT pattern
        if get_slice!(buf, 4) != b"DLT\x01" {
            return Err(ParseError::MissingDltPattern)
        }

 
        // unwrapping is ok here, because we check if there's enough data ahead of this
        let seconds = buf.get_u32_le();
        let microseconds = buf.get_i32_le();
        let ecu_id = get_str!(buf, 4)?.trim_end_matches('\0');

        Ok(Self {
            seconds,
            microseconds,
            ecu_id,
        })}


    pub fn len(&self) -> usize {
        4 /*DLT pattern*/ 
        + 4 /*seconds*/ 
        + 4 /*microseconds*/ 
        + 4 /*ecu id*/
    }
}

#[rustfmt::skip]
#[derive(Debug)]
#[repr(u8)]
enum StdHeaderMask {
    UseExtendedHeader = 0b00000001,
    MsbFirst =          0b00000010,
    WithEcuId =         0b00000100,
    WithSessionId =     0b00001000,
    WithTimestamp =     0b00010000,
    VersionNumber =     0b11100000,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StandardHeader<'a> {
    header_type: u8,
    pub message_counter: u8,
    pub length: u16,
    pub ecu_id: Option<&'a str>,
    pub session_id: Option<u32>,
    pub timestamp: Option<u32>,
}

impl<'a> StandardHeader<'a> {

    const MIN_LENGTH: usize = 4;

    pub fn from_slice(mut buf: &'a [u8]) -> Result<Self, DltError> {
        if buf.remaining() < Self::MIN_LENGTH {
            return Err(ParseError::NotEnoughData{needed: Self::MIN_LENGTH, available: buf.remaining()}.into());
        }

        // since we verified `data.len() < Self::MIN_LENGTH`
        // we can now safely read `header_type`, `message_counter` and `length`
        // without checking if enough bytes are available everytime
        let header_type = buf.get_u8();
        let message_counter = buf.get_u8();
        let length = buf.get_u16();

        let with_ecu_id = header_type & StdHeaderMask::WithEcuId as u8 != 0;
        let with_session_id = header_type & StdHeaderMask::WithSessionId as u8 != 0;
        let with_timestamp = header_type & StdHeaderMask::WithTimestamp as u8 != 0;
        // each of `ecu_id`, `session_id` and `timestamp` is 4 bytes long
        let must_have_remaining = 4 * (with_ecu_id as usize + with_session_id as usize + with_timestamp as usize);

        if must_have_remaining > buf.remaining(){
            return Err(ParseError::NotEnoughData{needed: must_have_remaining, available: buf.remaining()}.into());
        }

        let ecu_id = if with_ecu_id {
            Some(get_str!(buf, 4)?.trim_end_matches('\0'))
        } else {
            None
        };

        let session_id = if with_session_id {
            Some(buf.get_u32())
        } else {
            None
        };

        let timestamp = if with_timestamp {
         Some(buf.get_u32())
        } else {
            None
        };

        Ok(StandardHeader { header_type, message_counter, length, ecu_id, session_id, timestamp })
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

    pub fn big_endian(&self) -> bool {
        self.header_type & StdHeaderMask::MsbFirst as u8 != 0
    }

    pub fn len(&self) -> usize {
        1 /*header type*/
        + 1 /*message_counter */
        + 2 /*length */
        + self.ecu_id.is_some() as usize * 4
        + self.session_id.is_some() as usize * 4
        + self.timestamp.is_some() as usize * 4
    }
}


#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Log = 0x0,
    AppTrace = 0x1,
    NwTrace = 0x2,
    Control = 0x3
}

#[derive(Debug, Clone, Copy)]
pub enum LogInfo {
    Fatal = 0x1,
    Error = 0x2,
    Warn = 0x3,
    Info = 0x4,
    Debug = 0x5,
    Verbose = 0x6
}

#[derive(Debug, Clone, Copy)]
pub enum TraceInfo {
    Variable = 0x1,
    FunctionIn = 0x2,
    FunctionOut = 0x3,
    State = 0x4,
    Vfb = 0x5
}

#[derive(Debug, Clone, Copy)]
pub enum BusInfo {
    Ipc = 0x1,
    Can = 0x2,
    Flexray = 0x3,
    Most = 0x4,
    Ethernet = 0x5,
    SomeIP = 0x6,
    // UserDefined

}


#[derive(Debug, Clone, Copy)]
pub enum ControlInfo {
    Request = 0x1,
    Response = 0x2,
    // Time = 0x3 ??
}

#[derive(Debug, Clone, Copy)]
pub enum MessageTypeInfo {
    Log(LogInfo),
    Trace(TraceInfo),
    Bus(BusInfo),
    Control(ControlInfo)
}



#[derive(Debug, PartialEq, Eq)]
pub struct ExtendedHeader<'a> {
    message_info: u8,
    pub number_of_arguments: u8,
    pub application_id: &'a str,
    pub context_id: &'a str,
}

impl<'a> ExtendedHeader<'a> {

    const MIN_LENGTH: usize = 10;

    pub fn from_slice(mut buf: &'a [u8]) -> Result<Self, ParseError> {
        if buf.len() < Self::MIN_LENGTH {
            return Err(ParseError::NotEnoughData{needed: Self::MIN_LENGTH, available: buf.len()})
        }

        let message_info = buf.get_u8();
        let number_of_arguments = buf.get_u8();
        let application_id = get_str!(buf, 4)?.trim_end_matches('\0');
        let context_id = get_str!(buf, 4)?.trim_end_matches('\0');
        Ok(Self {
            message_info,
            number_of_arguments,
            application_id,
            context_id,
        })
}


    pub fn verbose(&self) -> bool {
        self.message_info & 0b00000001 != 0
    }

    pub fn message_type(&self) -> MessageType {
        match (self.message_info & 0b00001110) >> 1 {
            0x0 => MessageType::Log,
            0x1 => MessageType::AppTrace,
            0x2 => MessageType::NwTrace,
            0x3 => MessageType::Control,
            _ => unreachable!()
        }

    }

    pub fn type_info(&self) -> MessageTypeInfo {
        match (self.message_type(), (self.message_info & 0b11110000) >> 4) {
            (MessageType::Log, 0x1) => MessageTypeInfo::Log(LogInfo::Fatal),
            (MessageType::Log, 0x2) => MessageTypeInfo::Log(LogInfo::Error),
            (MessageType::Log, 0x3) => MessageTypeInfo::Log(LogInfo::Warn),
            (MessageType::Log, 0x4) => MessageTypeInfo::Log(LogInfo::Info),
            (MessageType::Log, 0x5) => MessageTypeInfo::Log(LogInfo::Debug),
            (MessageType::Log, 0x6) => MessageTypeInfo::Log(LogInfo::Verbose),
            (MessageType::AppTrace, 0x1) => MessageTypeInfo::Trace(TraceInfo::Variable),
            (MessageType::AppTrace, 0x2) => MessageTypeInfo::Trace(TraceInfo::FunctionIn),
            (MessageType::AppTrace, 0x3) => MessageTypeInfo::Trace(TraceInfo::FunctionOut),
            (MessageType::AppTrace, 0x4) => MessageTypeInfo::Trace(TraceInfo::State),
            (MessageType::AppTrace, 0x5) => MessageTypeInfo::Trace(TraceInfo::Vfb),
            (MessageType::NwTrace, 0x1)=> MessageTypeInfo::Bus(BusInfo::Ipc),
            (MessageType::NwTrace, 0x2)=> MessageTypeInfo::Bus(BusInfo::Can),
            (MessageType::NwTrace, 0x3)=> MessageTypeInfo::Bus(BusInfo::Flexray),
            (MessageType::NwTrace, 0x4)=> MessageTypeInfo::Bus(BusInfo::Most),
            (MessageType::NwTrace, 0x5)=> MessageTypeInfo::Bus(BusInfo::Ethernet),
            (MessageType::NwTrace, 0x6)=> MessageTypeInfo::Bus(BusInfo::SomeIP),
            (MessageType::Control, 0x1) => MessageTypeInfo::Control(ControlInfo::Request),
            (MessageType::Control, 0x2) => MessageTypeInfo::Control(ControlInfo::Response),
            (message_type, info) => unreachable!("Unexpected: ({message_type:?}, {info})")
        }
    }

    pub fn len(&self) -> usize {
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
        let header = StorageHeader::from_slice(bytes).unwrap();
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
        let header = StandardHeader::from_slice(bytes).unwrap();
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
        let header = ExtendedHeader::from_slice(bytes).unwrap();
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
