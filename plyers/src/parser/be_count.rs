use std::io::{Read, Seek};

use crate::{Bytes, CountType, Parser};

#[derive(Debug, thiserror::Error)]
#[error("failed when parsing a big-endian {type_}")]
pub struct BeCountError {
    source: Box<dyn std::error::Error>,
    type_: CountType,
}

impl BeCountError {
    fn new(source: Box<dyn std::error::Error>, t: CountType) -> Self {
        Self { source, type_: t }
    }
}

#[derive(Debug, Clone)]
pub struct BeCount {
    count_type: CountType,
}

pub fn be_count(count_type: CountType) -> BeCount {
    BeCount { count_type }
}

impl Parser for BeCount {
    type Error = BeCountError;
    type Item = usize;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek,
    {
        match self.count_type {
            CountType::U8 => Bytes::<1>
                .map(|n| u8::from_be_bytes(n) as usize)
                .parse(r)
                .map_err(|e| BeCountError::new(e, CountType::U8)),
            CountType::U16 => Bytes::<2>
                .map(|n| u16::from_be_bytes(n) as usize)
                .parse(r)
                .map_err(|e| BeCountError::new(e, CountType::U16)),
            CountType::U32 => Bytes::<4>
                .map(|n| u32::from_be_bytes(n) as usize)
                .parse(r)
                .map_err(|e| BeCountError::new(e, CountType::U32)),
        }
    }
}
