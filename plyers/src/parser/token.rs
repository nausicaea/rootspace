use std::io::{Read, Seek};

use crate::read_byte;
use crate::error::Error;

use super::Parser;

#[derive(Debug)]
pub struct Token {
    token: u8,
}

impl Parser for Token {
    type Item = ();

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, crate::error::Error> where Self:Sized, R: Read + Seek {
        let (byte, position) = read_byte(r)?;

        if byte != self.token {
            return Err(Error::UnexpectedByte(byte, position));
        } else {
            return Ok(());
        }
    }
}

pub fn token(t: u8) -> Token {
    Token {
        token: t,
    }
}
 
#[cfg(test)]
mod tests {
    use crate::to_reader;
    use super::*;

    #[test]
    fn token_parses_a_single_fixed_byte() {
        let mut stream = to_reader("hello");

        let r = token(b'h').parse(&mut stream);

        match r {
            Ok(()) => (),
            other => panic!("Expected Ok(), got: {:?}", other),
        }
    }
}
