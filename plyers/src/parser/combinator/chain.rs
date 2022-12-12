use std::io::{Read, Seek};

use crate::Parser;

#[derive(Debug, Clone)]
pub struct Chain<P, Q> {
    a: P,
    b: Q,
}

impl<P, Q> Chain<P, Q> {
    pub(crate) fn new(a: P, b: Q) -> Self {
        Chain { a, b }
    }
}

impl<P, Q> Parser for Chain<P, Q>
where
    P: Parser,
    P::Error: std::error::Error + 'static,
    Q: Parser,
    Q::Error: std::error::Error + 'static,
{
    type Error = Box<dyn std::error::Error + 'static>;
    type Item = (P::Item, Q::Item);

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek,
    {
        let ap = self.a.parse(r)?;

        let bp = self.b.parse(r)?;

        Ok((ap, bp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser::base::engram::engram, to_reader};

    #[test]
    fn chain_chains_two_parsers() {
        let mut stream = to_reader("helloworld");

        let r = engram(b"hello").chain(engram(b"world")).parse(&mut stream);

        match r {
            Ok((b"hello", b"world")) => (),
            other => panic!("Expected Ok((b\"hello\", b\"world\")), got: {:?}", other),
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
            Ok(((b"hello", b", "), b"world")) => (),
            other => panic!("Expected Ok(((b\"hello\", b\", \"), b\"world\")), got: {:?}", other),
        }
    }

    #[test]
    fn chain_never_calls_first_again_once_complete_or_failed() {
        #[derive(Default)]
        struct P(bool);

        #[derive(Debug, thiserror::Error)]
        #[error("error in tests")]
        struct PError;

        impl Parser for P {
            type Error = PError;
            type Item = ();

            fn parse<R>(mut self, r: &mut R) -> Result<Self::Item, Self::Error>
            where
                R: Read,
            {
                assert!(!self.0, "A::parse() was called more than once");
                self.0 = true;
                Ok(())
            }
        }

        let mut stream = to_reader("A, b, C");
        let r = P::default().chain(P::default()).parse(&mut stream);

        match r {
            Ok(((), ())) => (),
            other => panic!("Expected Ok(((), ())), got: {:?}", other),
        }
    }
}
