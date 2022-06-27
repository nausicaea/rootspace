use std::io::{Read, Seek, SeekFrom};
use crate::{Error, Parser, read_byte};

pub struct Lookahead {
    token: u8,
}

impl Parser for Lookahead {
    type Item = ();

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self: Sized, R: Read + Seek {
        let (byte, position) = read_byte(r)?;

        let _ = r.seek(SeekFrom::Start(position))?;

        if byte != self.token {
            return Err(Error::UnexpectedByte(byte, position));
        } else {
            return Ok(());
        }
    }
}

pub fn lookahead(t: u8) -> Lookahead {
    Lookahead {
        token: t,
    }
}

#[cfg(test)]
mod tests {
    use crate::to_reader;
    use std::io::Seek;
    use super::*;

    #[test]
    fn lookahead_parses_a_single_fixed_byte_but_does_not_advance_reader() {
        let mut stream = to_reader("hello");

        let position_before = stream.stream_position().unwrap();

        lookahead(b'h').parse(&mut stream).unwrap();

        let position_after = stream.stream_position().unwrap();

        assert_eq!(position_before, position_after);
    }
}