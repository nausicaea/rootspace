use std::io::{Read, Seek};

use crate::{error::Error, utilities::read_byte};

use super::Parser;

#[derive(Debug)]
pub struct TakeWhile<F> {
    func: Option<F>,
    buffer: Vec<u8>,
}

pub fn take_while<F>(predicate: F) -> TakeWhile<F>
where
    F: FnMut(u8) -> bool,
{
    TakeWhile {
        func: Some(predicate),
        buffer: Vec::new(),
    }
}


impl<F> Parser for TakeWhile<F> 
where
    F: FnMut(u8) -> bool,
{
    type Item = Vec<u8>;

    fn parse<R>(mut self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read + Seek {
        if let Some(ref mut func) = self.func {
            loop {
                let byte = read_byte(r)?;

                if !(func)(byte) {
                    self.func = None;
                    return Ok(self.buffer.clone());
                } else {
                    self.buffer.push(byte);
                }
            }
        }

        Err(Error::ParserExhausted)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::to_reader;

    use super::*;

    #[test]
    fn take_while_returns_everything_until_a_predicate_returns_false() {
        let expected: Vec<u8> = b"Hello, World!".iter().copied().collect();
        let mut stream = to_reader("Hello, World!\nBlabla");

        let r = take_while(|b| b != b'\n').parse(&mut stream);

        match r {
            Ok(d) if d == expected => (),
            other => panic!("Expected Ok(b\"Hello, World!\" as Vec<u8>), got: {:?}", other),
        }
    }

    #[test]
    fn take_while_throws_an_error_when_the_predicate_always_returns_true() {
        let mut stream = to_reader("Hello, World!\nBlabla");

        let r = take_while(|_| true).parse(&mut stream);

        match r {
            Err(Error::UnexpectedEndOfFile) => (),
            other => panic!("Expected Err(, got: {:?}", other),
        }
    }

    #[test]
    fn take_while_returns_nothing_if_the_predicate_is_always_false() {
        let expected: Vec<u8> = Vec::new();
        let mut stream = to_reader("Hello, World!\nBlabla");

        let r = take_while(|_| false).parse(&mut stream);

        match r {
            Ok(d) if d == expected => (),
            other => panic!("Expected Ok(Vec::new()), got: {:?}", other),
        }
    }
}
