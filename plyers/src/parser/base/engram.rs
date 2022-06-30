use std::io::{Read, Seek};
use anyhow::{bail, Context};

use crate::Parser;
use crate::{error::Error as EError, read_byte};
use crate::parser::error::{EngramError, AddressWrapper};
use crate::parser::read_byte::ReadByte;

#[derive(Debug, Clone)]
pub struct Engram {
    pattern: &'static [u8],
}

impl Parser for Engram {
    type Item = &'static [u8];

    fn parse<R>(self, r: &mut R) -> anyhow::Result<Self::Item>
    where
        Self: Sized,
        R: Read + Seek,
    {
        for t in self.pattern {
            let (byte, position) = r.read_byte()?;

            if byte != *t {
                bail!(AddressWrapper::new(EngramError::new(byte, *t, self.pattern), position));
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
