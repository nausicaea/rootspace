use std::io::{Read, Seek};
use anyhow::Context;

use crate::{Bytes, DataType, Error, Parser};

#[derive(Debug, Clone)]
pub struct BeNumber {
    data_type: DataType,
}

pub fn be_number(data_type: DataType) -> BeNumber {
    BeNumber { data_type }
}

impl Parser for BeNumber {
    type Item = Vec<u8>;

    fn parse<R>(self, r: &mut R) -> anyhow::Result<Self::Item>
    where
        Self: Sized,
        R: Read + Seek,
    {
        match self.data_type {
            DataType::I8 => Bytes::<1>
                .map(|n| i8::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .context("when parsing and converting a big-endian i8 to a native byte sequence"),
            DataType::U8 => Bytes::<1>
                .map(|n| u8::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .context("when parsing and converting a big-endian u8 to a native byte sequence"),
            DataType::I16 => Bytes::<2>
                .map(|n| i16::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .context("when parsing and converting a big-endian i16 to a native byte sequence"),
            DataType::U16 => Bytes::<2>
                .map(|n| u16::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .context("when parsing and converting a big-endian u16 to a native byte sequence"),
            DataType::I32 => Bytes::<4>
                .map(|n| i32::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .context("when parsing and converting a big-endian i32 to a native byte sequence"),
            DataType::U32 => Bytes::<4>
                .map(|n| u32::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .context("when parsing and converting a big-endian u32 to a native byte sequence"),
            DataType::F32 => Bytes::<4>
                .map(|n| f32::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .context("when parsing and converting a big-endian f32 to a native byte sequence"),
            DataType::F64 => Bytes::<8>
                .map(|n| f64::from_be_bytes(n).to_ne_bytes().into_iter().collect())
                .parse(r)
                .context("when parsing and converting a big-endian f64 to a native byte sequence"),
        }
    }
}
