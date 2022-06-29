use std::io::{Read, Seek, SeekFrom};
use anyhow::Context;

use either::{Either as EEither, Left, Right};

use crate::{Error, Parser};

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
    R: Parser,
{
    type Item = EEither<Q::Item, R::Item>;

    fn parse<S>(self, r: &mut S) -> anyhow::Result<Self::Item>
    where
        Self: Sized,
        S: Read + Seek,
    {
        let position = r.stream_position()
            .context("when reading the stream position in Either")?;

        match self.a.parse(r) {
            Ok(a) => Ok(Left(a)),
            Err(e) => {
                let _ = r.seek(SeekFrom::Start(position))
                    .with_context(|| format!("when setting the stream position in Either after parser a has failed with: {}", e))?;

                self.b.parse(r)
                    .map(|b| Right(b))
                    .context("when applying parser b in Either")
            }
        }
    }
}
