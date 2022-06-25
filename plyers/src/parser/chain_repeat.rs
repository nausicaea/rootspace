use std::io::{Read, Seek, SeekFrom};

use super::Parser;
use crate::error::Error;

#[derive(Debug)]
pub struct ChainRepeat<P, Q, R> {
    tail: P,
    at_least_once: Q,
    until: R,
}

impl<P, Q, R> ChainRepeat<P, Q, R> {
    pub fn new(tail: P, at_least_once: Q, until: R) -> Self {
        ChainRepeat { 
            tail,
            at_least_once, 
            until,
        }
    }
}

impl<P, Q, R> Parser for ChainRepeat<P, Q, R>
where
    P: Parser,
    Q: Parser + Clone,
    R: Parser + Clone,
{
    type Item = (P::Item, Vec<Q::Item>, R::Item);

    fn parse<S>(self, r: &mut S) -> Result<Self::Item, Error> where Self:Sized, S: Read + Seek, {
        let tail_p = self.tail.parse(r)?;

        let mut at_least_once_ps = vec![];

        let until_p = loop {
            at_least_once_ps.push(self.at_least_once.clone().parse(r)?);

            let position = r.stream_position()?;

            let until_r = self.until.clone().parse(r);
            match until_r {
                Ok(until_p) => break until_p,
                Err(Error::UnexpectedByte(_, _)) => {
                    let _ = r.seek(SeekFrom::Start(position))?;
                },
                Err(e) => return Err(e),
            }
        };


        Ok((tail_p, at_least_once_ps, until_p))
    }
}
