use std::io::{Read, Seek, SeekFrom};

use crate::Parser;

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
    type Error = Box<dyn std::error::Error + 'static>;

    fn parse<S>(self, r: &mut S) -> Result<Self::Item, Self::Error>
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
                    r.seek(SeekFrom::Start(position))?;

                    return Ok(at_least_once_ps);
                }
            }
        }
    }
}
