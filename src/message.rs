use std::fmt::Display;

use bytes::Buf;

use crate::{
    error::DltError,
    header::{
        BusInfo, ControlInfo, ExtendedHeader, LogInfo, MessageType, MessageTypeInfo,
        StandardHeader, StorageHeader, TraceInfo,
    },
    payload::{NonVerbosePayload, Payload, VerbosePayload},
};

#[derive(Debug)]
pub struct DltMessage<'a> {
    pub storage_header: StorageHeader<'a>,
    pub standard_header: StandardHeader<'a>,
    pub extended_header: Option<ExtendedHeader<'a>>,
    pub payload: Payload<'a>,
    source: &'a [u8],
}

impl<'a> DltMessage<'a> {
    pub fn from_slice(mut buf: &'a [u8]) -> Result<Self, DltError> {
        let source = buf;
        let storage_header = StorageHeader::from_slice(buf)?;
        buf.advance(storage_header.len());

        let standard_header = StandardHeader::from_slice(buf)?;
        buf.advance(standard_header.len());

        let extended_header = if standard_header.use_extended_header() {
            let extended_header = ExtendedHeader::from_slice(buf).map_err(|err| DltError {
                advance_by: Some(storage_header.len() + standard_header.length as usize),
                source: err,
            })?;
            buf.advance(extended_header.len());
            Some(extended_header)
        } else {
            None
        };

        let payload_length = standard_header.length as usize
            - standard_header.len()
            - extended_header.as_ref().map_or(0, ExtendedHeader::len);
        let payload = if extended_header
            .as_ref()
            .map_or(false, ExtendedHeader::verbose)
        {
            Payload::Verbose(
                VerbosePayload::from_slice(buf, payload_length, standard_header.msb_first())
                    .map_err(|err| DltError {
                        advance_by: Some(storage_header.len() + standard_header.length as usize),
                        source: err,
                    })?,
            )
        } else {
            Payload::NonVerbose(
                NonVerbosePayload::from_slice(buf, payload_length, standard_header.msb_first())
                    .map_err(|err| DltError {
                        advance_by: Some(storage_header.len() + standard_header.length as usize),
                        source: err,
                    })?,
            )
        };

        Ok(DltMessage {
            source: &source[..storage_header.len() + standard_header.length as usize],
            storage_header,
            standard_header,
            extended_header,
            payload,
        })
    }

    // The length of the message in bytes
    pub fn len(&self) -> usize {
        self.storage_header.len() + self.standard_header.length as usize
    }

    pub fn ecu_id(&self) -> &str {
        self.storage_header.ecu_id
    }
    pub fn app_id(&self) -> Option<&str> {
        self.extended_header
            .as_ref()
            .map(|ext_hdr| ext_hdr.application_id)
    }
    pub fn context_id(&self) -> Option<&str> {
        self.extended_header
            .as_ref()
            .map(|ext_hdr| ext_hdr.context_id)
    }

    pub fn message_type(&self) -> Option<MessageType> {
        self.extended_header
            .as_ref()
            .map(ExtendedHeader::message_type)
    }

    pub fn type_info(&self) -> Option<MessageTypeInfo> {
        self.extended_header.as_ref().map(ExtendedHeader::type_info)
    }

    pub fn verbose(&self) -> bool {
        self.extended_header
            .as_ref()
            .map_or(false, ExtendedHeader::verbose)
    }

    pub fn timestamp(&self) -> Option<u32> {
        self.standard_header.timestamp
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.source
    }
}

impl<'a> Display for DltMessage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(dt) = speedate::DateTime::from_timestamp(
            self.storage_header.seconds as i64,
            self.storage_header.microseconds as u32,
        ) {
            write!(
                f,
                "{:0>4}/{:0>2}/{:0>2} {} ",
                dt.date.year, dt.date.month, dt.date.day, dt.time,
            )?
        };

        if let Some(timestamp) = self.standard_header.timestamp {
            write!(f, "{}.{:0>4} ", timestamp / 10000, timestamp % 10000)?;
        }

        write!(f, "{:0>3} ", self.standard_header.message_counter)?;

        if let Some(ecu_id) = self.standard_header.ecu_id {
            write!(f, "{ecu_id} ")?;
        } else {
            write!(f, "{} ", self.storage_header.ecu_id)?;
        }

        if let Some(ref ext_hdr) = self.extended_header {
            write!(f, "{} {} ", ext_hdr.application_id, ext_hdr.context_id)?;
        }

        if let Some(session_id) = self.standard_header.session_id {
            write!(f, "{session_id} ")?;
        }

        if let Some(ref ext_hdr) = self.extended_header {
            match ext_hdr.type_info() {
                MessageTypeInfo::Log(LogInfo::Fatal) => write!(f, "log fatal "),
                MessageTypeInfo::Log(LogInfo::Error) => write!(f, "log error "),
                MessageTypeInfo::Log(LogInfo::Warn) => write!(f, "log warn "),
                MessageTypeInfo::Log(LogInfo::Info) => write!(f, "log info "),
                MessageTypeInfo::Log(LogInfo::Debug) => write!(f, "log debug "),
                MessageTypeInfo::Log(LogInfo::Verbose) => write!(f, "log verbose "),
                MessageTypeInfo::Trace(TraceInfo::Variable) => write!(f, "app_trace variable "),
                MessageTypeInfo::Trace(TraceInfo::FunctionIn) => write!(f, "app_trace func_in "),
                MessageTypeInfo::Trace(TraceInfo::FunctionOut) => write!(f, "app_trace func_out "),
                MessageTypeInfo::Trace(TraceInfo::State) => write!(f, "app_trace state "),
                MessageTypeInfo::Trace(TraceInfo::Vfb) => write!(f, "app_trace vfb "),
                MessageTypeInfo::Bus(BusInfo::Ipc) => write!(f, "nw_trace ipc "),
                MessageTypeInfo::Bus(BusInfo::Can) => write!(f, "nw_trace can "),
                MessageTypeInfo::Bus(BusInfo::Flexray) => write!(f, "nw_trace flexray "),
                MessageTypeInfo::Bus(BusInfo::Most) => write!(f, "nw_trace most "),
                MessageTypeInfo::Bus(BusInfo::Ethernet) => write!(f, "nw_trace ethernet "),
                MessageTypeInfo::Bus(BusInfo::SomeIP) => write!(f, "nw_trace some_ip "),
                MessageTypeInfo::Control(ControlInfo::Request) => write!(f, "control request "),
                MessageTypeInfo::Control(ControlInfo::Response) => write!(f, "control response "),
            }?;
        }

        if let Some(ref ext_hdr) = self.extended_header {
            if ext_hdr.verbose() {
                write!(f, "verbose ")?;
            } else {
                write!(f, "non-verbose ")?;
            }
        } else {
            write!(f, "non-verbose ")?;
        }

        if let Some(ref ext_hdr) = self.extended_header {
            write!(f, "{} ", ext_hdr.number_of_arguments)?;
        }

        write!(f, "{}", self.payload)?;

        Ok(())
    }
}
