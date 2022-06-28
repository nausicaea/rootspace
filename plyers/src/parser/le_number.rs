use std::io::{Read, Seek};
use crate::{Bytes, DataType, Error, Parser};

#[derive(Debug, Clone)]
pub struct LeNumber {
    data_type: DataType,
}

pub fn le_number(data_type: DataType) -> LeNumber {
    LeNumber { data_type }
}

impl Parser for LeNumber {
    type Item = Vec<u8>;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self: Sized, R: Read + Seek {
        match self.data_type {
            DataType::I8 => {
                Bytes::<1>.map(|n| i8::from_le_bytes(n).to_ne_bytes().into_iter().collect()).parse(r)
            }
            DataType::U8 => {
                Bytes::<1>.map(|n| u8::from_le_bytes(n).to_ne_bytes().into_iter().collect()).parse(r)
            }
            DataType::I16 => {
                Bytes::<2>.map(|n| i16::from_le_bytes(n).to_ne_bytes().into_iter().collect()).parse(r)
            }
            DataType::U16 => {
                Bytes::<2>.map(|n| u16::from_le_bytes(n).to_ne_bytes().into_iter().collect()).parse(r)
            }
            DataType::I32 => {
                Bytes::<4>.map(|n| i32::from_le_bytes(n).to_ne_bytes().into_iter().collect()).parse(r)
            }
            DataType::U32 => {
                Bytes::<4>.map(|n| u32::from_le_bytes(n).to_ne_bytes().into_iter().collect()).parse(r)
            }
            DataType::F32 => {
                Bytes::<4>.map(|n| f32::from_le_bytes(n).to_ne_bytes().into_iter().collect()).parse(r)
            }
            DataType::F64 => {
                Bytes::<8>.map(|n| f64::from_le_bytes(n).to_ne_bytes().into_iter().collect()).parse(r)
            }
        }
    }
}
