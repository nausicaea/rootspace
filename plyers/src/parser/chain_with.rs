use super::Parser;
use std::task::Poll;
use std::io::Read;
use crate::error::Error;

pub struct ChainWith<P, Q, F> {
    a: Option<P>,
    b: Option<Q>,
    func: F,
}

impl<P, Q, F> ChainWith<P, Q, F> 
where
    P: Parser,
    Q: Parser,
    F: Fn(P::Item) -> Q,
{
    pub fn new(a: P, func: F) -> Self {
        ChainWith {
            a: Some(a),
            b: None,
            func,
        }
    }
}

impl<P, Q, F> Parser for ChainWith<P, Q, F>
where
    P: Parser,
    Q: Parser,
    F: Fn(P::Item) -> Q,
{
    type Item = Q::Item;

    fn next<R>(&mut self, r: &mut R) -> Poll<Result<Self::Item, Error>> where R: Read {
        todo!()
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
            Ok(()) => (),
            other => panic!("Expected Ok(()), got: {:?}", other),
        }
    }
}
