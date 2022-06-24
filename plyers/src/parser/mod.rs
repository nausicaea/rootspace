use std::io::Read;
use std::task::Poll;
use crate::error::Error;

mod chain;
mod chain_with;
mod take_while;
mod engram;
mod one_of;

pub trait Parser {
    type Item;

    fn next<R>(&mut self, r: &mut R) -> Poll<Result<Self::Item, Error>> where R: Read;

    fn chain<P>(self, second: P) -> self::chain::Chain<Self, P> 
    where
        Self: Sized,
        P: Parser,
    {
        self::chain::Chain::new(self, second)
    }

    fn chain_with<Q, F>(self, func: F) -> self::chain_with::ChainWith<Self, Q, F>
    where
        Self: Sized,
        Q: Parser,
        F: Fn(Self::Item) -> Q,
    {
        self::chain_with::ChainWith::new(self, func)
    }

    fn parse<R>(mut self, reader: &mut R) -> Result<Self::Item, Error>
    where
        Self: Sized,
        R: Read,
    {
        loop {
            match self.next(reader) {
                Poll::Pending => continue,
                Poll::Ready(r) => break r,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use super::{engram::engram, one_of::one_of};

    #[test]
    fn can_i_get_a_ply_parser_out_of_this() {
        let path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
        let mut file = std::fs::File::open(path);

        let p = engram(b"ply\n")
            .chain(one_of(&[b"format ", b"element ", b"property list", b"property ", b"comment ", b"obj_info "]))
            .chain_with(|p| {
            });
    }
}
