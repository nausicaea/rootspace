use std::io::{Read, Seek};

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
pub enum TokenError {
    #[error(transparent)]
    UnexpectedByte(AddressWrapper<UnexpectedByte>),
    #[error(transparent)]
    Se(#[from] AddressWrapper<StreamError>),
}

impl TokenError {
    pub fn unexpected_byte(received: u8, expected: u8, position: u64) -> Self {
        TokenError::UnexpectedByte(AddressWrapper::new(UnexpectedByte { received, expected }, position))
    }
}

#[derive(Debug)]
pub struct Token {
    token: u8,
}

impl Parser for Token {
    type Error = TokenError;
    type Item = ();

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek,
    {
        let (byte, position) = r.read_byte()?;

        if byte != self.token {
            Err(TokenError::unexpected_byte(byte, self.token, position))
        } else {
            Ok(())
        }
    }
}

pub fn token(t: u8) -> Token {
    Token { token: t }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::to_reader;

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
