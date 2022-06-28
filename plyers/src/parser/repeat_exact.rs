use std::io::{Read, Seek};
use crate::{Error, Parser};

pub struct RepeatExact<P, Q> {
    tail: P,
    repeated: Q,
    n: usize,
}

impl<P, Q> RepeatExact<P, Q> {
    pub fn new(tail: P, repeated: Q, n: usize) -> Self {
        RepeatExact { tail, repeated, n }
    }
}

impl<P, Q> Parser for RepeatExact<P, Q>
where
    P: Parser,
    Q: Parser + Clone,
{
    type Item = (P::Item, Vec<Q::Item>);

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self: Sized, R: Read + Seek {
        let tail_p = self.tail.parse(r)?;

        let mut repeated_ps = vec![];

        for _ in 0..self.n {
            repeated_ps.push(self.repeated.clone().parse(r)?);
        }

        Ok((tail_p, repeated_ps))
    }
}