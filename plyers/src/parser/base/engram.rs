use std::io::{Read, Seek};
use anyhow::Context;

use crate::Parser;
use crate::{error::Error as EError, read_byte};

#[derive(Debug, Clone)]
pub struct Engram<'a> {
    pattern: &'a [u8],
}

impl<'a> Parser for Engram<'a> {
    type Item = &'a [u8];

    fn parse<R>(self, r: &mut R) -> anyhow::Result<Self::Item>
    where
        Self: Sized,
        R: Read + Seek,
    {
        let mut index: usize = 0;
        loop {
            let (byte, position) = read_byte(r)
                .context("when searching for a fixed byte pattern (i.e. engram)")?;

            if byte != self.pattern[index] {
                anyhow::bail!(EError::UnexpectedByte(byte, position));
            }

            index += 1;

            if index >= self.pattern.len() {
                return Ok(self.pattern);
            }
        }
    }
}

pub fn engram(e: &[u8]) -> Engram {
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
