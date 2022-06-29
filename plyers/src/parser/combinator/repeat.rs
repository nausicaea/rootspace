use std::io::{Read, Seek, SeekFrom};

use crate::Parser;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Repeat<P> {
    at_least_once: P,
}

pub fn repeat<P>(at_least_once: P) -> Repeat<P> {
    Repeat { at_least_once }
}

impl<P> Parser for Repeat<P>
where
    P: Parser + Clone,
{
    type Item = Vec<P::Item>;

    fn parse<S>(self, r: &mut S) -> anyhow::Result<Self::Item>
    where
        Self: Sized,
        S: Read + Seek,
    {
        let mut at_least_once_ps = vec![];

        loop {
            let position = r.stream_position()?;

            match self.at_least_once.clone().parse(r) {
                Ok(alop) => {
                    at_least_once_ps.push(alop);
                }
                Err(_) => {
                    let _ = r.seek(SeekFrom::Start(position))?;
                    return Ok(at_least_once_ps);
                }
            }
        }
    }
}
