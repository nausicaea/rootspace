use std::io::{Read, Seek, SeekFrom};

use super::Parser;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Repeat<P, Q> {
    tail: P,
    at_least_once: Q,
}

impl<P, Q> Repeat<P, Q> {
    pub fn new(tail: P, at_least_once: Q) -> Self {
        Repeat {
            tail,
            at_least_once, 
        }
    }
}

impl<P, Q> Parser for Repeat<P, Q>
where
    P: Parser,
    Q: Parser + Clone,
{
    type Item = (P::Item, Vec<Q::Item>);

    fn parse<S>(self, r: &mut S) -> Result<Self::Item, Error> where Self:Sized, S: Read + Seek, {
        let tail_p = self.tail.parse(r)?;

        let mut at_least_once_ps = vec![];

        loop {
            let position = r.stream_position()?;

            match self.at_least_once.clone().parse(r) {
                Ok(alop) => {
                    at_least_once_ps.push(alop);
                },
                Err(_) => {
                    let _ = r.seek(SeekFrom::Start(position))?;
                    return Ok((tail_p, at_least_once_ps));
                },
            }
        }
    }
}
