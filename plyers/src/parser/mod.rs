use std::io::{Read, Seek};
use crate::error::Error;

pub mod chain;
pub mod chain_with;
pub mod take_while;
pub mod engram;
pub mod token;
pub mod map;
pub mod and_then;
pub mod chain_repeat;
pub mod empty;
pub mod chain_exact;
pub mod lookahead;
pub mod chain_either;
pub mod chain_optional;

pub trait Parser {
    type Item;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read + Seek;

    fn chain<P>(self, second: P) -> chain::Chain<Self, P>
    where
        Self: Sized,
    {
        chain::Chain::new(self, second)
    }

    fn chain_with<Q, F>(self, func: F) -> chain_with::ChainWith<Self, F>
    where
        Self: Sized,
        Q: Parser,
        F: Fn(&Self::Item) -> Q,
    {
        chain_with::ChainWith::new(self, func)
    }

    fn chain_exact<Q>(self, n: usize, repeated: Q) -> chain_exact::ChainExact<Self, Q>
    where
        Self: Sized,
        Q: Parser + Clone,
    {
        chain_exact::ChainExact::new(self, repeated, n)
    }

    fn chain_either<Q, R>(self, a: Q, b: R) -> chain_either::ChainEither<Self, Q, R>
    where
        Self: Sized,
        Q: Parser,
        R: Parser,
    {
        chain_either::ChainEither::new(self, a, b)
    }

    fn chain_optional<Q>(self, a: Q) -> chain_optional::ChainOptional<Self, Q>
    where
        Self: Sized,
        Q: Parser,
    {
        chain_optional::ChainOptional::new(self, a)
    }

    fn map<J, F>(self, func: F) -> map::Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> J,
    {
        map::Map::new(self, func)
    }

    fn and_then<J, F>(self, func: F) -> and_then::AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> Result<J, Error>,
    {
        and_then::AndThen::new(self, func)
    }

    fn chain_repeat<Q, R>(self, at_least_once: Q, until: R) -> chain_repeat::ChainRepeat<Self, Q, R>
    where
        Self: Sized,
        Q: Parser + Clone,
        R: Parser + Clone,
    {
        chain_repeat::ChainRepeat::new(self, at_least_once, until)
    }
}
