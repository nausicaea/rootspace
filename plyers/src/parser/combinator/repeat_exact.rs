use std::io::{Read, Seek};

use crate::Parser;

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
    Q::Error: std::error::Error + 'static,
{
    type Item = Vec<Q::Item>;
    type Error = Box<dyn std::error::Error + 'static>;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek,
    {
        let mut repeated_ps = vec![];

        for _ in 0..self.n {
            let p = self.repeated.clone().parse(r)?;

            repeated_ps.push(p);
        }

        Ok(repeated_ps)
    }
}
