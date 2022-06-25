use std::io::{Read, Seek};
use crate::{Error, Parser};

#[derive(Debug, Clone)]
pub struct Empty;

pub fn empty() -> Empty {
    Empty
}

impl Parser for Empty {
    type Item = ();

    fn parse<R>(self, _r: &mut R) -> Result<Self::Item, Error> where Self: Sized, R: Read + Seek {
        Ok(())
    }
}