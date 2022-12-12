use std::io::{Read, Seek, SeekFrom};

use crate::Parser;

#[derive(Debug, Clone)]
pub struct RepeatUntil<Q, R> {
    at_least_once: Q,
    until: R,
}

pub fn repeat_until<Q, R>(at_least_once: Q, until: R) -> RepeatUntil<Q, R> {
    RepeatUntil { at_least_once, until }
}

impl<Q, R> Parser for RepeatUntil<Q, R>
where
    Q: Parser + Clone,
    Q::Error: std::error::Error + 'static,
    R: Parser + Clone,
{
    type Error = Box<dyn std::error::Error + 'static>;
    type Item = (Vec<Q::Item>, R::Item);

    fn parse<S>(self, r: &mut S) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        S: Read + Seek,
    {
        let mut at_least_once_ps = vec![];

        let until_p = loop {
            let alop = self.at_least_once.clone().parse(r)?;
            at_least_once_ps.push(alop);

            let position = r.stream_position()?;

            match self.until.clone().parse(r) {
                Ok(until_p) => break until_p,
                Err(_) => {
                    r.seek(SeekFrom::Start(position))?;
                }
            }
        };

        Ok((at_least_once_ps, until_p))
    }
}
