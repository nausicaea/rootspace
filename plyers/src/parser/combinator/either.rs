use std::io::{Read, Seek, SeekFrom};

use either::{Either as EEither, Left, Right};

use crate::Parser;

#[derive(Debug, Clone)]
pub struct Either<Q, R> {
    a: Q,
    b: R,
}

pub fn either<Q, R>(a: Q, b: R) -> Either<Q, R> {
    Either { a, b }
}

impl<Q, R> Parser for Either<Q, R>
where
    Q: Parser,
    Q::Error: std::error::Error + 'static,
    R: Parser,
    R::Error: std::error::Error + 'static,
{
    type Error = Box<dyn std::error::Error + 'static>;
    type Item = EEither<Q::Item, R::Item>;

    fn parse<S>(self, r: &mut S) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        S: Read + Seek,
    {
        let position = r.stream_position()?;

        match self.a.parse(r) {
            Ok(a) => Ok(Left(a)),
            Err(e) => {
                let _ = r.seek(SeekFrom::Start(position))?;

                let b = self.b.parse(r).map(|b| Right(b))?;

                Ok(b)
            }
        }
    }
}
