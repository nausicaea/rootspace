use std::io::{Read, Seek};
use crate::{Error, Parser};

pub struct RepeatExact<Q> {
    repeated: Q,
    n: usize,
}

pub fn repeat_exact<Q>(repeated: Q, n: usize) -> RepeatExact<Q> {
        RepeatExact { repeated, n }
    }

impl<Q> Parser for RepeatExact<Q>
where
    Q: Parser + Clone,
{
    type Item = Vec<Q::Item>;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self: Sized, R: Read + Seek {
        let mut repeated_ps = vec![];

        for _ in 0..self.n {
            repeated_ps.push(self.repeated.clone().parse(r)?);
        }

        Ok(repeated_ps)
    }
}