#![allow(unused)]

use std::fmt::Display;

use chrono::{TimeZone, Utc};

use crate::{
    BusInfo, ControlInfo, ExtendedHeader, LogInfo, MessageTypeInfo, NonVerbosePayload, Payload,
    Result, StandardHeader, StorageHeader, TraceInfo, VerbosePayload,
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
            Payload::Verbose(VerbosePayload::new(
                &buf[offset..payload_end],
                standard_header.msb_first(),
            ))
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

impl<'a> Display for DltMessage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{:0<6} ",
            // This is unfortunately really slow, ~50% of cpu time in this function is just this
            Utc.timestamp_millis_opt(self.storage_header.seconds as i64 * 1000)
                .map(|dt| dt.format("%Y/%m/%d %H:%M:%S"))
                .unwrap(),
            self.storage_header.microseconds
        )?;

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
            write!(f, "{} ", ext_hdr.number_of_arguments)?;
        }

        write!(f, "{}", self.payload)?;

        Ok(())
    }
}
