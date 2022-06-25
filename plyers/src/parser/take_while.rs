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

    fn parse<R>(&mut self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: std::io::Read {
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
    use super::*;

    #[test]
    fn take_while_returns_everything_until_a_predicate_returns_false() {
        let source = b"Hello, World!\nBlabla";
        let expected: Vec<u8> = source.iter().take_while(|b| b != &&b'\n').copied().collect();
        let mut stream = source.as_slice();

        let mut p = take_while(|b| b != b'\n');

        let r = p.parse(&mut stream);


        match r {
            Ok(d) if d == expected => (),
            other => panic!("Expected Ok(b\"Hello, World!\" as Vec<u8>), got: {:?}", other),
        }
    }

    #[test]
    fn take_while_throws_an_error_when_the_predicate_always_returns_true() {
        let source = b"Hello, World!\nBlabla";
        let mut stream = source.as_slice();

        let mut p = take_while(|_| true);

        let r = p.parse(&mut stream);

        match r {
            Err(Error::UnexpectedEndOfFile) => (),
            other => panic!("Expected Ok(b\"Hello, World!\nBlabla\" as Vec<u8>), got: {:?}", other),
        }
    }

    #[test]
    fn take_while_returns_nothing_if_the_predicate_is_always_false() {
        let source = b"Hello, World!\nBlabla";
        let expected: Vec<u8> = Vec::new();
        let mut stream = source.as_slice();

        let mut p = take_while(|_| false);

        let r = p.parse(&mut stream);

        match r {
            Ok(d) if d == expected => (),
            other => panic!("Expected Ok(Vec::new()), got: {:?}", other),
        }
    }
}
