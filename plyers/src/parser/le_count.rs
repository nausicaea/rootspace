use std::io::{Read, Seek};
use crate::{Bytes, CountType, Error, Parser};

#[derive(Debug, Clone)]
pub struct LeCount {
    count_type: CountType,
}

pub fn le_count(count_type: CountType) -> LeCount {
    LeCount { count_type }
}

impl Parser for LeCount {
    type Item = usize;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self: Sized, R: Read + Seek {
        match self.count_type {
            CountType::U8 => {
                Bytes::<1>.map(|n| u8::from_le_bytes(n) as usize).parse(r)
            },
            CountType::U16 => {
                Bytes::<2>.map(|n| u16::from_le_bytes(n) as usize).parse(r)
            },
            CountType::U32 => {
                Bytes::<4>.map(|n| u32::from_le_bytes(n) as usize).parse(r)
            },
        }
    }
}
