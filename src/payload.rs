#![allow(unused)]
use std::{fmt::Display, marker::PhantomData};

use crate::{error::Result, DltError};
#[derive(Debug)]
pub struct NonVerbosePayload<'a> {
    message_id: u32,
    data: &'a [u8],
}

impl<'a> NonVerbosePayload<'a> {
    pub fn new(buf: &'a [u8]) -> Result<Self> {
        let message_id = u32::from_be_bytes(buf[0..4].try_into()?);
        let data = &[];
        Ok(Self { message_id, data })
    }

    pub fn as_str(&self) -> Result<&str> {
        Ok(std::str::from_utf8(self.data)?)
    }
}

#[derive(Debug)]
pub struct VerbosePayload<'a> {
    data: Option<&'a [u8]>, // TODO: temporary hack
}

impl<'a> VerbosePayload<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { data: None }
    }
}

#[derive(Debug)]
pub enum Payload<'a> {
    NonVerbose(NonVerbosePayload<'a>),
    Verbose(VerbosePayload<'a>),
}

impl<'a> Payload<'a> {
    pub fn as_non_verbose(&self) -> Option<&NonVerbosePayload> {
        match self {
            Payload::NonVerbose(nv) => Some(nv),
            Payload::Verbose(_) => None,
        }
    }
    pub fn as_verbose(&self) -> Option<&VerbosePayload> {
        match self {
            Payload::NonVerbose(_) => None,
            Payload::Verbose(v) => Some(v),
        }
    }
}

/*trait Argument {
    type ValueType;

    fn value(&self) -> Self::ValueType;
    fn size(&self) -> usize;
}*/

struct ArgString<'a> {
    data: &'a str,
}

struct Argument<'a> {
    kind: ArgumentKind,
    data: &'a [u8],
}

impl<'a> Argument<'a> {
    fn as_str(&self) -> Result<&str> {
        match self.kind {
            ArgumentKind::String => Ok(std::str::from_utf8(self.data)?),
            kind => Err(DltError::KindMismatch(kind, ArgumentKind::String)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ArgumentKind {
    U8,
    U16,
    U32,
    U64,
    String,
}

impl Display for ArgumentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
