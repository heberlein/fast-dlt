// Release R22-11 Breaks backward-compatibility, TODO use #[cfg(feature = "r22-11")] to have both versions at the same time
// https://www.autosar.org/fileadmin/standards/foundation/22-11/AUTOSAR_PRS_LogAndTraceProtocol.pdf

// Currently targeting release R20-11
// https://www.autosar.org/fileadmin/standards/R20-11/FO/AUTOSAR_PRS_LogAndTraceProtocol.pdf

#![warn(missing_debug_implementations, rust_2018_idioms)]
pub mod borrowed;
pub mod error;
pub mod owned;

#[macro_export]
macro_rules! get_str {
    ($buf:expr, $len: expr) => {{
        let ret = $buf.get(..$len);
        let ret = ret.ok_or(ParseError::NotEnoughData {
            needed: $len,
            available: $buf.remaining(),
        })?;

        $buf.advance($len);
        from_utf8(ret).map_err(ParseError::from)
    }};
}

#[macro_export]
macro_rules! get_slice {
    ($buf: expr, $len: expr) => {{
        let ret = $buf.get(..$len);
        let ret = ret.ok_or(ParseError::NotEnoughData {
            needed: $len,
            available: $buf.remaining(),
        })?;

        $buf.advance($len);
        ret
    }};
}
