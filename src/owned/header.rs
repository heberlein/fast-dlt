use bytes_utils::Str;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// #[cfg_attr(test, derive(Arbitrary))]
pub struct StorageHeader {
    pub(crate) seconds: u32,
    pub(crate) microseconds: i32,
    pub(crate) ecu_id: Str,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// #[cfg_attr(test, derive(Arbitrary))]
pub struct StandardHeader {
    pub(crate) header_type: u8,
    pub(crate) message_counter: u8,
    pub(crate) length: u16,
    pub(crate) ecu_id: Option<Str>,
    pub(crate) session_id: Option<u32>,
    pub(crate) timestamp: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// #[cfg_attr(test, derive(Arbitrary))]
pub struct ExtendedHeader {
    pub(crate) message_info: u8,
    pub(crate) number_of_arguments: u8,
    pub(crate) app_id: Str,
    pub(crate) ctx_id: Str,
}
