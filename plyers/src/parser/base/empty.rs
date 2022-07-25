use std::io::{Read, Seek};

use crate::{Parser};

#[derive(Debug, thiserror::Error)]
#[error("error in Empty parser that should never occur")]
pub struct EmptyError;

#[derive(Debug, Clone)]
pub struct Empty;

pub fn empty() -> Empty {
    Empty
}

impl Parser for Empty {
    type Error = EmptyError;
    type Item = ();

    fn parse<R>(self, _: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek,
    {
        Ok(())
    }
}
