use std::io::{Read, Seek, SeekFrom};

use crate::Parser;

#[derive(Debug, Clone)]
pub struct Optional<P>(P);

pub fn optional<P>(p: P) -> Optional<P> {
    Optional(p)
}

impl<P> Parser for Optional<P>
where
    P: Parser,
{
    type Error = Box<dyn std::error::Error + 'static>;
    type Item = Option<P::Item>;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek,
    {
        let position = r.stream_position()?;

        match self.0.parse(r) {
            Ok(p) => Ok(Some(p)),
            Err(_) => {
                r.seek(SeekFrom::Start(position))?;

                Ok(None)
            }
        }
    }
}
