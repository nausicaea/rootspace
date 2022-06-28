use std::io::{Read, Seek};
use crate::{Bytes, CountType, Error, Parser};

#[derive(Debug, Clone)]
pub struct BeCount {
    count_type: CountType,
}

pub fn be_count(count_type: CountType) -> BeCount {
    BeCount { count_type }
}

impl Parser for BeCount {
    type Item = usize;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self: Sized, R: Read + Seek {
        match self.count_type {
            CountType::U8 => {
                Bytes::<1>.map(|n| u8::from_be_bytes(n) as usize).parse(r)
            },
            CountType::U16 => {
                Bytes::<2>.map(|n| u16::from_be_bytes(n) as usize).parse(r)
            },
            CountType::U32 => {
                Bytes::<4>.map(|n| u32::from_be_bytes(n) as usize).parse(r)
            },
        }
    }
}
