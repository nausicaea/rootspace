use std::io::{Read, Seek};

use crate::utilities::read_byte;
use crate::error::Error;

use super::Parser;

#[derive(Debug)]
pub struct Token {
    token: u8,
    exhausted: bool,
}

impl Parser for Token {
    type Item = ();

    fn parse<R>(mut self, r: &mut R) -> Result<Self::Item, crate::error::Error> where Self:Sized, R: Read + Seek {
        if !self.exhausted {
            let byte = read_byte(r)?;

            if byte != self.token {
                self.exhausted = true;
                return Err(Error::UnexpectedByte(byte));
            } else {
                self.exhausted = true;
                return Ok(());
            }
        }

        Err(Error::ParserExhausted)
    }
}

pub fn token(t: u8) -> Token {
    Token {
        token: t,
        exhausted: false,
    }
}
 
#[cfg(test)]
mod tests {
    use crate::parser::to_reader;
    use super::*;

    #[test]
    fn token_parses_a_single_fixed_byte() {
        let mut stream = to_reader("hello");

        let mut p = token(b'h');

        let r = p.parse(&mut stream);

        match r {
            Ok(()) => (),
            other => panic!("Expected Ok(), got: {:?}", other),
        }
    }
}
