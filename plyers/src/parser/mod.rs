use std::io::{Read, Seek};

use combinator::{and_then, chain, map, optional, repeat};

pub mod base;
pub mod be_count;
pub mod be_number;
pub mod combinator;
pub mod error;
pub mod le_count;
pub mod le_number;
pub mod read_byte;

pub trait Parser {
    type Error;
    type Item;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek;

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
        F: Fn(Self::Item) -> anyhow::Result<J>,
    {
        and_then::AndThen::new(self, func)
    }
}
