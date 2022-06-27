use super::Parser;
use std::io::{Read, Seek};
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct ChainWith<P, F> {
    a: P,
    b: F,
}

impl<P, F> ChainWith<P, F> {
    pub fn new(a: P, b: F) -> Self {
        ChainWith {
            a, b
        }
    }
}

impl<P, Q, F> Parser for ChainWith<P, F>
where
    P: Parser,
    Q: Parser,
    F: FnMut(&P::Item) -> Q,
{
    type Item = (P::Item, Q::Item);

    fn parse<R>(mut self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read + Seek {
        let ap = self.a.parse(r)?;
        let b = (self.b)(&ap);
        let bp = b.parse(r)?;

        Ok((ap, bp))
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::{engram::engram};
    use crate::to_reader;
    use super::*;

    #[test]
    fn chain_with_chains_two_parsers() {
        let mut stream = to_reader("hellohello");

        let r = engram(b"hello")
            .chain_with(|p| engram(p))
            .parse(&mut stream);

        match r {
            Ok((b"hello", b"hello")) => (),
            other => panic!("Expected Ok((b\"hello\", b\"hello\")), got: {:?}", other),
        }
    }
}
