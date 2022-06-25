use std::io::Read;
use crate::utilities::read_byte;
use crate::error::Error;
use super::Parser;

#[derive(Debug)]
pub struct Engram<'a> {
    pattern: &'a [u8],
    exhausted: bool,
}

impl<'a> Parser for Engram<'a> {
    type Item = ();

    fn parse<R>(&mut self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read {
        if !self.exhausted {
            let mut index: usize = 0;
            loop {
                let byte = read_byte(r)?;

                if byte != self.pattern[index] {
                    self.exhausted = true;
                    return Err(Error::UnexpectedByte(byte));
                }

                index += 1;

                if index >= self.pattern.len() {
                    self.exhausted = true;
                    return Ok(());
                }
            }
        }

        Err(Error::ParserExhausted)
    }
}

pub fn engram(e: &[u8]) -> Engram {
    if e.is_empty() {
        panic!("engram cannot match an empty pattern");
    }

    Engram { pattern: e, exhausted: false }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engram_parses_a_single_fixed_word() {
        let source = "hello";
        let mut stream = source.as_bytes();

        let mut p = engram(b"hello");

        let r = p.parse(&mut stream);

        match r {
            Ok(()) => (),
            other => panic!("Expected Ok(), got: {:?}", other),
        }
    }

    #[test]
    fn engram_fails_on_the_first_wrong_byte() {
        let source = "hallo";
        let mut stream = source.as_bytes();

        let mut p = engram(b"hello");

        let r = p.parse(&mut stream);

        match r {
            Err(Error::UnexpectedByte(b'a')) => (),
            other => panic!("Expected Error::UnexpectedByte(b'a'), got: {:?}", other),
        }
    }

    #[test]
    fn engram_fails_if_called_after_completion() {
        let source = "hello";
        let mut stream = source.as_bytes();

        let mut p = engram(b"hello");

        let _ = p.parse(&mut stream);

        match p.parse(&mut stream) {
            Err(Error::ParserExhausted) => (),
            r => panic!("{:?}", r),
        }
    }

    #[test]
    #[should_panic]
    fn engram_panics_on_empty_pattern() {
        let _ = engram(&[]);
    }
}
