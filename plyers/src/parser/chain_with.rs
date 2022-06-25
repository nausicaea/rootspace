use super::Parser;
use std::io::Read;
use crate::error::Error;

#[derive(Debug)]
pub struct ChainWith<P, F> {
    a: Option<P>,
    b: Option<F>,
}

impl<P, Q, F> ChainWith<P, F> 
where
    P: Parser,
    Q: Parser,
    F: FnMut(&P::Item) -> Q,
{
    pub fn new(a: P, b: F) -> Self {
        ChainWith {
            a: Some(a),
            b: Some(b),
        }
    }
}

macro_rules! fuse {
    ($self:ident . $parser:ident . $($call:tt)+) => {
        match $self.$parser {
            Some(ref mut parser) => match parser.$($call)+ {
                Ok(product) => {
                    $self.$parser = None;
                    Ok(product)
                },
                Err(e) => {
                    $self.$parser = None;
                    Err(e)
                },
            },
            None => Err(Error::ParserExhausted),
        }
    };
}

impl<P, Q, F> Parser for ChainWith<P, F>
where
    P: Parser,
    Q: Parser,
    F: FnMut(&P::Item) -> Q,
{
    type Item = (P::Item, Q::Item);

    fn parse<R>(&mut self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read {
        match fuse!(self.a.parse(r)) {
            Ok(ap) => match self.b {
                Some(ref mut parser) => {
                    let mut b = (parser)(&ap);
                    Ok((ap, b.parse(r)?))
                },
                None => Err(Error::ParserExhausted),
            },
            Err(e) => Err(e),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::{one_of::one_of, engram::engram};
    use super::*;

    #[test]
    fn chain_with_chains_two_parsers() {
        let source = "hellohello";
        let mut stream = source.as_bytes();

        let r = one_of(&[b"goodbye", b"hello"])
            .chain_with(|p| engram(p))
            .parse(&mut stream);

        match r {
            Ok((b"hello", ())) => (),
            other => panic!("Expected Ok((b\"hello\", ())), got: {:?}", other),
        }
    }
}
