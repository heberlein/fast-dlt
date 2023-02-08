// Release R22-11 Breaks backward-compatibility, TODO use #[cfg(feature = "r22-11")] to have both versions at the same time
// https://www.autosar.org/fileadmin/standards/foundation/22-11/AUTOSAR_PRS_LogAndTraceProtocol.pdf

// Currently targeting release R20-11
// https://www.autosar.org/fileadmin/standards/foundation/20-11/AUTOSAR_PRS_LogAndTraceProtocol.pdf

#![warn(missing_debug_implementations, rust_2018_idioms)]
mod error;
pub use crate::error::*;

mod header;
pub use crate::header::*;

mod message;
pub use crate::message::*;

mod payload;
pub use crate::payload::*;

mod file;
pub use crate::file::*;

mod filter;
pub use crate::filter::*;

// TODO: use Cow<'a, str> everywhere?
