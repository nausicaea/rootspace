use std::io::{Read, Seek};
use crate::read_byte;
use crate::error::Error;
use super::Parser;

#[derive(Debug, Clone)]
pub struct Engram<'a> {
    pattern: &'a [u8],
}

impl<'a> Parser for Engram<'a> {
    type Item = ();

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read + Seek {
        let mut index: usize = 0;
        loop {
            let (byte, position) = read_byte(r)?;

            if byte != self.pattern[index] {
                return Err(Error::UnexpectedByte(byte, position));
            }

            index += 1;

            if index >= self.pattern.len() {
                return Ok(());
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
    use std::io::SeekFrom;
    use crate::to_reader;
    use super::*;

    #[test]
    fn engram_parses_a_single_fixed_word() {
        let mut stream = to_reader("hello");

        let r = engram(b"hello").parse(&mut stream);

        match r {
            Ok(()) => (),
            other => panic!("Expected Ok(), got: {:?}", other),
        }
    }

    #[test]
    fn engram_fails_on_the_first_wrong_byte() {
        let mut stream = to_reader("hallo");

        let r = engram(b"hello").parse(&mut stream);

        match r {
            Err(Error::UnexpectedByte(b'a', SeekFrom::Start(1))) => (),
            other => panic!("Expected Error::UnexpectedByte(b'a'), got: {:?}", other),
        }
    }

    #[test]
    #[should_panic]
    fn engram_panics_on_empty_pattern() {
        let _ = engram(&[]);
    }
}
