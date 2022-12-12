use std::io::{Read, Seek};

use crate::{Bytes, DataType, Parser};

#[derive(Debug, thiserror::Error)]
#[error("failed when parsing a big-endian {type_}")]
pub struct BeNumberError {
    source: Box<dyn std::error::Error>,
    type_: DataType,
}

impl BeNumberError {
    fn new(source: Box<dyn std::error::Error>, t: DataType) -> Self {
        Self { source, type_: t }
    }
}

#[derive(Debug, Clone)]
pub struct BeNumber {
    data_type: DataType,
}

pub fn be_number(data_type: DataType) -> BeNumber {
    BeNumber { data_type }
}

impl Parser for BeNumber {
    type Error = BeNumberError;
    type Item = Vec<u8>;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek,
    {
        match self.data_type {
            DataType::I8 => Bytes::<1>
                .map(|n| i8::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .map_err(|e| BeNumberError::new(e, DataType::I8)),
            DataType::U8 => Bytes::<1>
                .map(|n| u8::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .map_err(|e| BeNumberError::new(e, DataType::U8)),
            DataType::I16 => Bytes::<2>
                .map(|n| i16::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .map_err(|e| BeNumberError::new(e, DataType::I16)),
            DataType::U16 => Bytes::<2>
                .map(|n| u16::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .map_err(|e| BeNumberError::new(e, DataType::U16)),
            DataType::I32 => Bytes::<4>
                .map(|n| i32::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .map_err(|e| BeNumberError::new(e, DataType::I32)),
            DataType::U32 => Bytes::<4>
                .map(|n| u32::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .map_err(|e| BeNumberError::new(e, DataType::U32)),
            DataType::F32 => Bytes::<4>
                .map(|n| f32::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .map_err(|e| BeNumberError::new(e, DataType::F32)),
            DataType::F64 => Bytes::<8>
                .map(|n| f64::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .map_err(|e| BeNumberError::new(e, DataType::F64)),
        }
    }
}
