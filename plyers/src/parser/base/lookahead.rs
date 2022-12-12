use std::io::{Read, Seek, SeekFrom};

use crate::{
    parser::{
        error::{AddressWrapper, StreamError},
        read_byte::ReadByte,
    },
    Parser,
};

#[derive(Debug, thiserror::Error)]
#[error("received byte {received:#x}, but expected byte {expected:#x}")]
pub struct UnexpectedByte {
    received: u8,
    expected: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum LookaheadError {
    #[error(transparent)]
    UnexpectedByte(AddressWrapper<UnexpectedByte>),
    #[error(transparent)]
    Se(#[from] AddressWrapper<StreamError>),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl LookaheadError {
    fn unexpected_byte(received: u8, expected: u8, position: u64) -> Self {
        LookaheadError::UnexpectedByte(AddressWrapper::new(UnexpectedByte { received, expected }, position))
    }
}

#[derive(Debug, Clone)]
pub struct Lookahead {
    token: u8,
}

impl Parser for Lookahead {
    type Error = LookaheadError;
    type Item = ();

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek,
    {
        let (byte, position) = r.read_byte()?;

        let _ = r.seek(SeekFrom::Start(position)).map_err(|e| LookaheadError::Io(e))?;

        if byte != self.token {
            return Err(LookaheadError::unexpected_byte(byte, self.token, position));
        } else {
            return Ok(());
        }
    }
}

pub fn lookahead(t: u8) -> Lookahead {
    Lookahead { token: t }
}

#[cfg(test)]
mod tests {
    use std::io::Seek;

    use super::*;
    use crate::to_reader;

    #[test]
    fn lookahead_parses_a_single_fixed_byte_but_does_not_advance_reader() {
        let mut stream = to_reader("hello");

        let position_before = stream.stream_position().unwrap();

        lookahead(b'h').parse(&mut stream).unwrap();

        let position_after = stream.stream_position().unwrap();

        assert_eq!(position_before, position_after);
    }
}
