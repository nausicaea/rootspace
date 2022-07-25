use std::io::{Read, Seek};


use crate::{
    parser::{
        error::{AddressWrapper, StreamError},
        read_byte::ReadByte,
    },
    Parser,
};

#[derive(Debug, thiserror::Error)]
#[error("received byte {received:#x}, but expected byte {expected:#x} in engram {engram:?}")]
pub struct UnexpectedByte {
    received: u8,
    expected: u8,
    engram: &'static [u8],
}

#[derive(Debug, thiserror::Error)]
pub enum EngramError {
    #[error(transparent)]
    UnexpectedByte(AddressWrapper<UnexpectedByte>),
    #[error(transparent)]
    Se(#[from] AddressWrapper<StreamError>),
}

impl EngramError {
    fn unexpected_byte(received: u8, expected: u8, engram: &'static [u8], position: u64) -> Self {
        Self::UnexpectedByte(
            AddressWrapper::new(
                UnexpectedByte {
                    received,
                    expected,
                    engram,
                },
                position,
            )
        )
    }
}

#[derive(Debug, Clone)]
pub struct Engram {
    pattern: &'static [u8],
}

impl Parser for Engram {
    type Error = EngramError;
    type Item = &'static [u8];

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek,
    {
        for t in self.pattern {
            let (byte, position) = r.read_byte()?;

            if byte != *t {
                return Err(EngramError::unexpected_byte(byte, *t, self.pattern, position));
            }
        }

        Ok(self.pattern)
    }
}

pub fn engram(e: &'static [u8]) -> Engram {
    if e.is_empty() {
        panic!("engram cannot match an empty pattern");
    }

    Engram { pattern: e }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::to_reader;

    #[test]
    fn engram_parses_a_single_fixed_word() {
        let mut stream = to_reader("hello");

        let r = engram(b"hello").parse(&mut stream);

        match r {
            Ok(b"hello") => (),
            other => panic!("Expected Ok(b\"hello\"), got: {:?}", other),
        }
    }

    #[test]
    fn engram_fails_on_the_first_wrong_byte() {
        let mut stream = to_reader("hallo");

        let r = engram(b"hello").parse(&mut stream);
        assert!(r.is_err(), "{:?}", r.unwrap());
    }

    #[test]
    #[should_panic]
    fn engram_panics_on_empty_pattern() {
        let _ = engram(&[]);
    }
}
