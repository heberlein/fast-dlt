#![allow(unused)]
use std::{fmt::Display, marker::PhantomData};

use bytes::Buf;
use simdutf8::basic::{from_utf8, Utf8Error};

use crate::{
    error::{DltError, ParseError},
    get_slice, get_str,
};
#[derive(Debug)]
pub struct NonVerbosePayload<'a> {
    message_id: u32,
    data: &'a [u8],
}

impl<'a> NonVerbosePayload<'a> {
    pub fn from_slice(
        mut buf: &'a [u8],
        length: usize,
        msb_first: bool,
    ) -> Result<Self, ParseError> {
        if length > buf.remaining() {
            return Err(ParseError::NotEnoughData {
                needed: length,
                available: buf.remaining(),
            });
        }

        let message_id = if msb_first {
            buf.get_u32()
        } else {
            buf.get_u32_le()
        };
        let data = buf.get(..length).ok_or_else(|| ParseError::NotEnoughData {
            needed: length,
            available: buf.remaining(),
        })?;
        Ok(Self { message_id, data })
    }

    pub fn as_str(&self) -> Result<&str, Utf8Error> {
        Ok(from_utf8(self.data)?.trim_end_matches('\0'))
    }

    pub fn len(&self) -> usize {
        4 + self.data.len()
    }
}

impl<'a> Display for NonVerbosePayload<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] ", self.message_id)?;
        self.data
            .iter()
            .try_for_each(|byte| write!(f, "{byte:02x}"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct VerbosePayload<'a> {
    data: &'a [u8],
    msb_first: bool,
}

impl<'a> VerbosePayload<'a> {
    pub fn from_slice(buf: &'a [u8], length: usize, msb_first: bool) -> Result<Self, ParseError> {
        let data = buf.get(..length).ok_or_else(|| ParseError::NotEnoughData {
            needed: length,
            available: buf.remaining(),
        })?;
        Ok(Self { data, msb_first })
    }

    pub fn new(buf: &'a [u8], msb_first: bool) -> Self {
        Self {
            data: buf,
            msb_first,
        }
    }

    pub fn arguments(&self) -> Arguments<'a> {
        Arguments {
            data: self.data,
            index: 0,
            msb_first: self.msb_first,
            fatal: false,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<'a> Display for VerbosePayload<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for arg in self.arguments() {
            match arg {
                Ok(arg) => write!(f, "{arg} ")?,
                Err(_) => write!(f, "ARGERROR")?, // TODO: this could be better
            };
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Payload<'a> {
    NonVerbose(NonVerbosePayload<'a>),
    Verbose(VerbosePayload<'a>),
}

impl<'a> Payload<'a> {
    pub fn as_non_verbose(&self) -> Option<&NonVerbosePayload<'a>> {
        match self {
            Payload::NonVerbose(nv) => Some(nv),
            Payload::Verbose(_) => None,
        }
    }
    pub fn as_verbose(&self) -> Option<&VerbosePayload<'a>> {
        match self {
            Payload::NonVerbose(_) => None,
            Payload::Verbose(v) => Some(v),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Payload::NonVerbose(nv) => nv.len(),
            Payload::Verbose(v) => v.len(),
        }
    }
}

impl<'a> Display for Payload<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Payload::NonVerbose(nv) => write!(f, "{nv}"),
            Payload::Verbose(v) => write!(f, "{v}"),
        }
    }
}

#[rustfmt::skip]
#[derive(Debug)]
#[repr(u32)]
enum TypeInfoMask {
    Type =         0b00000000000000000110011111110000,
    Length =       0b00000000000000000000000000001111,
    VariableInfo = 0b00000000000000000000100000000000,
    FixedPoint =   0b00000000000000000001000000000000,
    StringCoding = 0b00000000000000111000000000000000,
}

#[rustfmt::skip]
#[derive(Debug)]
#[repr(u32)]
enum TypeInfo {
    Size8 =        0b00000000000000000000000000000001,
    Size16 =       0b00000000000000000000000000000010,
    Size32 =       0b00000000000000000000000000000011,
    Size64 =       0b00000000000000000000000000000100,
    Size128 =      0b00000000000000000000000000000101,
    Bool =         0b00000000000000000000000000010000,
    Signed =       0b00000000000000000000000000100000,
    Unsigned =     0b00000000000000000000000001000000,
    Float =        0b00000000000000000000000010000000,
    Array =        0b00000000000000000000000100000000,
    String =       0b00000000000000000000001000000000,
    Raw =          0b00000000000000000000010000000000,
    VariableInfo = 0b00000000000000000000100000000000,
    FixedPoint =   0b00000000000000000001000000000000,
    TraceInfo =    0b00000000000000000010000000000000,
    Struct =       0b00000000000000000100000000000000,
    Ascii =        0b00000000000000000000000000000000,
    Utf8 =         0b00000000000000001000000000000000,
}

// TODO: make sure error handling in Arguments actually makes sense
/// An iterator over the arguments of a verbose payload.
/// If an argument can not be parsed, this iterator will return an error and will not attempt to parse any further arguments
#[derive(Debug)]
pub struct Arguments<'a> {
    data: &'a [u8],
    index: usize,
    msb_first: bool,
    fatal: bool,
}

impl<'a> Iterator for Arguments<'a> {
    type Item = Result<Argument<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.fatal || self.index >= self.data.len() {
            return None;
        }
        match Argument::new(&self.data[self.index..], self.msb_first) {
            Ok(arg) => {
                self.index += arg.len();
                Some(Ok(arg))
            }
            Err(err) => {
                self.fatal = true;
                Some(Err(err))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Argument<'a> {
    type_info: u32,
    name: Option<&'a str>,
    unit: Option<&'a str>,
    value: Value<'a>,
}

impl<'a> Argument<'a> {
    const MIN_LENGTH: usize = todo!();

    fn new(mut buf: &'a [u8], msb_first: bool) -> Result<Argument<'_>, ParseError> {
        macro_rules! msb {
            ($be: expr, $le: expr) => {{
                if msb_first {
                    $be
                } else {
                    $le
                }
            }};
        }

        let type_info = buf.get_u32_le();
        let var_info = (type_info & TypeInfo::VariableInfo as u32) != 0;
        let fixed_point = (type_info & TypeInfo::FixedPoint as u32) != 0;

        let type_length = type_info & TypeInfoMask::Length as u32;
        let r#type = type_info & TypeInfoMask::Type as u32;

        let arg_type = match ArgType::try_from(r#type) {
            Ok(arg_type) => arg_type,
            Err(unknown) => return Err(ParseError::UnknownArgumentType(unknown)),
        };

        if var_info {
            return Err(ParseError::Unsupported("var info"));
        }
        if fixed_point {
            return Err(ParseError::Unsupported("fixed point"));
        }

        let value = match arg_type {
            ArgType::Bool => Value::Bool(buf.get_u8() != 0),
            ArgType::Signed => match type_length {
                0x01 => Value::I8(buf.get_i8()),
                0x02 => Value::I16(msb!(buf.get_i16(), buf.get_i16_le())),
                0x03 => Value::I32(msb!(buf.get_i32(), buf.get_i32_le())),
                0x04 => Value::I64(msb!(buf.get_i64(), buf.get_i64_le())),
                0x05 => Value::I128(msb!(buf.get_i128(), buf.get_i128_le())),
                _ => unreachable!(),
            },
            ArgType::Unsigned => match type_length {
                0x01 => Value::U8(buf.get_u8()),
                0x02 => Value::U16(msb!(buf.get_u16(), buf.get_u16_le())),
                0x03 => Value::U32(msb!(buf.get_u32(), buf.get_u32_le())),
                0x04 => Value::U64(msb!(buf.get_u64(), buf.get_u64_le())),
                0x05 => Value::U128(msb!(buf.get_u128(), buf.get_u128_le())),
                _ => unreachable!(),
            },
            ArgType::Float => match type_length {
                0x01 => unreachable!(),
                0x02 => return Err(ParseError::Unsupported("f16")),
                0x03 => Value::F32(msb!(buf.get_f32(), buf.get_f32_le())),
                0x04 => Value::F64(msb!(buf.get_f64(), buf.get_f64_le())),
                0x05 => return Err(ParseError::Unsupported("f128")),
                _ => unreachable!(),
            },
            ArgType::Array => {
                return Err(ParseError::UnimplementedArgumentType("array"));
            }
            ArgType::String => {
                let length = msb!(buf.get_u16(), buf.get_u16_le());
                Value::String(get_str!(buf, length as usize)?.trim_end_matches('\0'))
            }
            ArgType::Raw => {
                let length = msb!(buf.get_u16(), buf.get_u16_le());
                Value::Raw(get_slice!(buf, length as usize))
            }
            ArgType::Struct => {
                return Err(ParseError::UnimplementedArgumentType("struct"));
            }
        };
        Ok(Argument {
            type_info,
            name: None,
            unit: None,
            value,
        })
    }

    fn len(&self) -> usize {
        4 + self.value.len()
    }
}

impl<'a> Display for Argument<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Clone)]
pub enum Value<'a> {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F32(f32),
    F64(f64),
    //Array, // unsupported
    String(&'a str),
    Raw(&'a [u8]),
}

impl<'a> Value<'a> {
    fn len(&self) -> usize {
        match self {
            Value::U8(_) | Value::I8(_) | Value::Bool(_) => 1,
            Value::U16(_) | Value::I16(_) => 2,
            Value::U32(_) | Value::I32(_) | Value::F32(_) => 4,
            Value::U64(_) | Value::I64(_) | Value::F64(_) => 8,
            Value::U128(_) | Value::I128(_) => 16,
            Value::String(s) => s.len() + 1 + 2, /*length of string is u16*/
            Value::Raw(r) => r.len() + 2,        /*length of raw is u16*/
        }
    }
}

impl<'a> Display for Value<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::U8(u) => write!(f, "{u}"),
            Value::U16(u) => write!(f, "{u}"),
            Value::U32(u) => write!(f, "{u}"),
            Value::U64(u) => write!(f, "{u}"),
            Value::U128(u) => write!(f, "{u}"),
            Value::I8(i) => write!(f, "{i}"),
            Value::I16(i) => write!(f, "{i}"),
            Value::I32(i) => write!(f, "{i}"),
            Value::I64(i) => write!(f, "{i}"),
            Value::I128(i) => write!(f, "{i}"),
            Value::F32(fl) => write!(f, "{fl:?}"),
            Value::F64(fl) => write!(f, "{fl:?}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Raw(r) => r.iter().try_for_each(|byte| write!(f, "{byte:02x}")),
        }
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum ArgType {
    Bool =         0b00000000000000000000000000010000,
    Signed =       0b00000000000000000000000000100000,
    Unsigned =     0b00000000000000000000000001000000,
    Float =        0b00000000000000000000000010000000,
    Array =        0b00000000000000000000000100000000,
    String =       0b00000000000000000000001000000000,
    Raw =          0b00000000000000000000010000000000,
    // VariableInfo = 0b00000000000000000000100000000000,
    // FixedPoint =   0b00000000000000000001000000000000,
    // TraceInfo =    0b00000000000000000010000000000000,
    Struct =       0b00000000000000000100000000000000,
}

impl TryFrom<u32> for ArgType {
    type Error = u32;

    #[inline]
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0b00000000000000000000000000010000 => Ok(Self::Bool),
            0b00000000000000000000000000100000 => Ok(Self::Signed),
            0b00000000000000000000000001000000 => Ok(Self::Unsigned),
            0b00000000000000000000000010000000 => Ok(Self::Float),
            0b00000000000000000000000100000000 => Ok(Self::Array),
            0b00000000000000000000001000000000 => Ok(Self::String),
            0b00000000000000000000010000000000 => Ok(Self::Raw),
            0b00000000000000000100000000000000 => Ok(Self::Struct),
            other => Err(other),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StringCoding {
    Ascii,
    Utf8,
}
