use bytes::Bytes;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VerbosePayload {
    pub(crate) data: Bytes,
    pub(crate) msb_first: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NonVerbosePayload {
    pub(crate) message_id: u32,
    pub(crate) data: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Payload {
    Verbose(VerbosePayload),
    NonVerbose(NonVerbosePayload),
}
