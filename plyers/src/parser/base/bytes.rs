use std::io::{Read, Seek};


use crate::{
    parser::{error::{StreamError, AddressWrapper}, read_byte::ReadByte},
    Parser,
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BytesError(#[from] AddressWrapper<StreamError>);

#[derive(Debug, Clone, Default)]
pub struct Bytes<const N: usize>;

impl<const N: usize> Parser for Bytes<N> {
    type Error = BytesError;
    type Item = [u8; N];

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek,
    {
        let mut p = [0; N];
        for i in 0..N {
            let (b, _) = r.read_byte()?;
            p[i] = b;
        }
        Ok(p)
    }
}
