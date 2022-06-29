use std::io::{Read, Seek};

use crate::Parser;

#[derive(Debug, Clone)]
pub struct Map<P, F> {
    a: P,
    func: F,
}

impl<P, F> Map<P, F> {
    pub fn new(a: P, func: F) -> Self {
        Map { a, func }
    }
}

impl<P, J, F> Parser for Map<P, F>
where
    P: Parser,
    F: FnMut(P::Item) -> J,
{
    type Item = J;

    fn parse<R>(mut self, r: &mut R) -> anyhow::Result<Self::Item>
    where
        Self: Sized,
        R: Read + Seek,
    {
        let product = self.a.parse(r)?;
        Ok((self.func)(product))
    }
}
