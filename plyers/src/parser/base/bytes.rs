use std::io::{Read, Seek};
use anyhow::Context;

use crate::{Error, Parser};

#[derive(Debug, Clone, Default)]
pub struct Bytes<const N: usize>;

impl<const N: usize> Parser for Bytes<N> {
    type Item = [u8; N];

    fn parse<R>(self, r: &mut R) -> anyhow::Result<Self::Item>
    where
        Self: Sized,
        R: Read + Seek,
    {
        let mut p = [0; N];
        for i in 0..N {
            let (b, _) = crate::read_byte(r)
                .with_context(|| format!("when parsing a sequence of {} bytes into an array", N))?;
            p[i] = b;
        }
        Ok(p)
    }
}
