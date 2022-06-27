use std::io::{Read, Seek, SeekFrom};
use crate::{Error, Parser};

#[derive(Debug, Clone)]
pub struct ChainOptional<P, Q> {
    tail: P,
    a: Q,
}

impl<P, Q> ChainOptional<P, Q> {
    pub fn new(tail: P, a: Q) -> Self {
        ChainOptional {
            tail,
            a,
        }
    }
}

impl<P, Q> Parser for ChainOptional<P, Q>
where
    P: Parser,
    Q: Parser,
{
    type Item = (P::Item, Option<Q::Item>);

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self: Sized, R: Read + Seek {
        let tail = self.tail.parse(r)?;

        let position = r.stream_position()?;

        match self.a.parse(r) {
            Ok(a) => Ok((tail, Some(a))),
            Err(_) => {
                r.seek(SeekFrom::Start(position))?;
                Ok((tail, None))
            }
        }
    }
}