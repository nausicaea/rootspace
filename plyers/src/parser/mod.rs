use std::io::{Read, Seek};
use crate::error::Error;

pub mod chain;
pub mod take_while;
pub mod engram;
pub mod token;
pub mod map;
pub mod and_then;
pub mod repeat_until;
pub mod empty;
pub mod repeat_exact;
pub mod lookahead;
pub mod chain_either;
pub mod optional;
pub mod repeat;

pub trait Parser {
    type Item;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read + Seek;

    fn chain<P>(self, second: P) -> chain::Chain<Self, P>
    where
        Self: Sized,
    {
        chain::Chain::new(self, second)
    }

    fn repeat_exact<Q>(self, n: usize, repeated: Q) -> repeat_exact::RepeatExact<Self, Q>
    where
        Self: Sized,
        Q: Parser + Clone,
    {
        repeat_exact::RepeatExact::new(self, repeated, n)
    }

    fn chain_either<Q, R>(self, a: Q, b: R) -> chain_either::ChainEither<Self, Q, R>
    where
        Self: Sized,
        Q: Parser,
        R: Parser,
    {
        chain_either::ChainEither::new(self, a, b)
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

    fn repeat_until<Q, R>(self, at_least_once: Q, until: R) -> repeat_until::RepeatUntil<Self, Q, R>
    where
        Self: Sized,
        Q: Parser + Clone,
        R: Parser + Clone,
    {
        repeat_until::RepeatUntil::new(self, at_least_once, until)
    }
}
