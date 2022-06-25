use std::io::{Read, Seek};
use super::Parser;
use crate::error::Error;

#[derive(Debug)]
pub struct Chain<P, Q> {
    a: P,
    b: Q,
}

impl<P, Q> Chain<P, Q> {
    pub(crate) fn new(a: P, b: Q) -> Self {
        Chain { 
            a, b
        }
    }
}

impl<P, Q> Parser for Chain<P, Q> 
where
    P: Parser,
    Q: Parser,
{
    type Item = (P::Item, Q::Item);

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read + Seek {
        let ap = self.a.parse(r)?;

        let bp = self.b.parse(r)?;

        Ok((ap, bp))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::to_reader;

    use super::*;
    use super::super::engram::engram;

    #[test]
    fn chain_chains_two_parsers() {
        let mut stream = to_reader("helloworld");

        let r = engram(b"hello")
            .chain(engram(b"world"))
            .parse(&mut stream);

        match r {
            Ok(((), ())) => (),
            other => panic!("Expected Ok(((), ())), got: {:?}", other),
        }
    }

    #[test]
    fn chain_allows_long_chains() {
        let mut stream = to_reader("hello, world");

        let r = engram(b"hello")
            .chain(engram(b", "))
            .chain(engram(b"world"))
            .parse(&mut stream);

        match r {
            Ok((((), ()), ())) => (),
            other => panic!("Expected Ok((((), ()), ())), got: {:?}", other),
        }
    }

    #[test]
    fn chain_never_calls_first_again_once_complete_or_failed() {
        #[derive(Default)]
        struct P(bool);

        impl Parser for P {
            type Item = ();
            
             fn parse<R>(mut self, _r: &mut R) -> Result<Self::Item, Error> where R: Read {
                assert!(!self.0, "A::parse() was called more than once");
                self.0 = true;
                Ok(())
             }
        }

        let mut stream = to_reader("A, b, C");
        let r = P::default()
            .chain(P::default())
            .parse(&mut stream);

        match r {
            Ok(((), ())) => (),
            other => panic!("Expected Ok(((), ())), got: {:?}", other),
        }
    }
}
