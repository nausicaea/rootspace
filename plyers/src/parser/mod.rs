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
pub mod bytes;
pub mod le_count;
pub mod be_count;
pub mod le_number;
pub mod be_number;

pub trait Parser {
    type Item;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read + Seek;

    fn repeated(self) -> repeat::Repeat<Self>
    where
        Self: Sized,
    {
        repeat::repeat(self)
    }

    fn optional(self) -> optional::Optional<Self>
    where
        Self: Sized,
    {
        optional::optional(self)
    }

    fn chain<P>(self, second: P) -> chain::Chain<Self, P>
    where
        Self: Sized,
        P: Parser,
    {
        chain::Chain::new(self, second)
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
}
