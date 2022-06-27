use either::{Either, Left, Right};
use std::io::{Read, Seek, SeekFrom};
use crate::{Error, Parser};

#[derive(Debug, Clone)]
pub struct ChainEither<P, Q, R> {
    tail: P,
    a: Q,
    b: R
}

impl<P, Q, R> ChainEither<P, Q, R> {
    pub fn new(tail: P, a: Q, b: R) -> Self {
        ChainEither { tail, a, b }
    }
}

impl<P, Q, R> Parser for ChainEither<P, Q, R>
where
    P: Parser,
    Q: Parser,
    R: Parser,
{
    type Item = (P::Item, Either<Q::Item, R::Item>);

    fn parse<S>(self, r: &mut S) -> Result<Self::Item, Error> where Self: Sized, S: Read + Seek {
        let tail = self.tail.parse(r)?;

        let position = r.stream_position()?;

        match self.a.parse(r) {
            Ok(a) => Ok((tail, Left(a))),
            Err(_) => {
                let _ = r.seek(SeekFrom::Start(position))?;

                match self.b.parse(r) {
                    Ok(b) => Ok((tail, Right(b))),
                    Err(b) => Err(b),
                }
            }
        }
    }
}
