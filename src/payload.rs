#![allow(unused)]
use std::{fmt::Display, marker::PhantomData};

use fallible_iterator::FallibleIterator;
use simdutf8::basic::from_utf8;

use crate::{
    error::FatalError,
    error::{ParseError, Result},
};
#[derive(Debug)]
pub struct NonVerbosePayload<'a> {
    message_id: u32,
    data: &'a [u8],
}

impl<'a> NonVerbosePayload<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self> {
        let message_id = u32::from_be_bytes(buf[0..4].try_into()?);
        let data = &buf[4..];
        Ok(Self { message_id, data })
    }

    pub fn as_str(&self) -> Result<&str> {
        Ok(from_utf8(self.data)?.trim_end_matches('\0'))
    }

    pub fn len(&self) -> usize {
        4 + self.data.len()
    }
}

impl<'a> Display for NonVerbosePayload<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:x}] ", self.message_id)?;
        self.data
            .iter()
            .try_for_each(|byte| write!(f, "{byte:x}"))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct VerbosePayload<'a> {
    data: &'a [u8],
    msb_first: bool,
}

impl<'a> VerbosePayload<'a> {
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
                Err(_) => write!(f, "ARGERROR")?,
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

#[derive(Debug)]
pub struct Arguments<'a> {
    data: &'a [u8],
    index: usize,
    msb_first: bool,
}

impl<'a> FallibleIterator for Arguments<'a> {
    type Item = Argument<'a>;
    type Error = ParseError;

    fn next(&mut self) -> Result<Option<Argument<'a>>> {
        if self.index >= self.data.len() {
            return Ok(None);
        }
        let arg = Argument::new(&self.data[self.index..], self.msb_first)?;
        self.index += arg.len();
        Ok(Some(arg))
    }
}

impl<'a> IntoIterator for Arguments<'a> {
    type Item = Result<Argument<'a>>;

    type IntoIter = fallible_iterator::Iterator<Self>;

    fn into_iter(self) -> Self::IntoIter {
        self.iterator()
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
    fn new(buf: &'a [u8], msb_first: bool) -> Result<Argument<'_>> {
        macro_rules! parse_value {
            ($type: ty,  $slice: expr) => {
                if msb_first {
                    <$type>::from_be_bytes($slice.try_into()?)
                } else {
                    <$type>::from_le_bytes($slice.try_into()?)
                }
            };
        }

        let type_info = u32::from_le_bytes(buf[0..4].try_into()?);
        let var_info = (); // TODO
        let type_length = type_info & TypeInfoMask::Length as u32;
        let r#type = type_info & TypeInfoMask::Type as u32;

        let value = match r#type {
            x if x == ArgType::Bool as u32 => Value::Bool(buf[0] != 0),
            x if x == ArgType::Signed as u32 => match type_length {
                0x01 => Value::I8(buf[0] as i8),
                0x02 => Value::I16(parse_value!(i16, buf[4..6])),
                0x03 => Value::I32(parse_value!(i32, buf[4..8])),
                0x04 => Value::I64(parse_value!(i64, buf[4..12])),
                0x05 => Value::I128(parse_value!(i128, buf[4..20])),
                _ => unreachable!(),
            },
            x if x == ArgType::Unsigned as u32 => match type_length {
                0x01 => Value::U8(buf[0]),
                0x02 => Value::U16(parse_value!(u16, buf[4..6])),
                0x03 => Value::U32(parse_value!(u32, buf[4..8])),
                0x04 => Value::U64(parse_value!(u64, buf[4..12])),
                0x05 => Value::U128(parse_value!(u128, buf[4..20])),
                _ => unreachable!(),
            },
            x if x == ArgType::Float as u32 => match type_length {
                0x01 => unimplemented!(),
                0x02 => unimplemented!(),
                0x03 => Value::F32(parse_value!(f32, buf[4..8])),
                0x04 => Value::F64(parse_value!(f64, buf[4..12])),
                0x05 => unimplemented!(),
                _ => unreachable!(),
            },

            x if x == ArgType::Array as u32 => todo!(),
            x if x == ArgType::String as u32 => {
                let length = parse_value!(u16, buf[4..6]);
                Value::String(from_utf8(&buf[6..6 + length as usize])?.trim_end_matches('\0'))
            }
            x if x == ArgType::Raw as u32 => {
                let length = parse_value!(u16, buf[4..6]);
                Value::Raw(&buf[6..6 + length as usize])
            }
            x if x == ArgType::VariableInfo as u32 => todo!(),
            x if x == ArgType::FixedPoint as u32 => todo!(),
            x if x == ArgType::TraceInfo as u32 => todo!(),
            x if x == ArgType::Struct as u32 => todo!(),
            x => unreachable!("{x:x}"),
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
    //Array, // intentionally unsupported
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

// NOTE: use https://crates.io/crates/fixed for fixed point values?
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
            Value::F32(fl) => write!(f, "{fl}"),
            Value::F64(fl) => write!(f, "{fl}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Raw(r) => r.iter().try_for_each(|byte| write!(f, "{byte:x?}")),
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
    VariableInfo = 0b00000000000000000000100000000000,
    FixedPoint =   0b00000000000000000001000000000000,
    TraceInfo =    0b00000000000000000010000000000000,
    Struct =       0b00000000000000000100000000000000,
}

#[derive(Debug, Clone, Copy)]
pub enum StringCoding {
    Ascii,
    Utf8,
}
