use std::io::{Read, Seek};

use anyhow::Context;

use crate::Parser;

#[derive(Debug, Clone)]
pub struct AndThen<P, F> {
    a: P,
    func: F,
}

impl<P, F> AndThen<P, F> {
    pub fn new(a: P, func: F) -> Self {
        AndThen { a, func }
    }
}

impl<P, J, F> Parser for AndThen<P, F>
where
    P: Parser,
    F: FnMut(P::Item) -> anyhow::Result<J>,
{
    type Item = J;

    fn parse<R>(mut self, r: &mut R) -> anyhow::Result<Self::Item>
    where
        Self: Sized,
        R: Read + Seek,
    {
        let product = self.a.parse(r).context("before applying the AndThen mapping closure")?;
        (self.func)(product).context("when applying the AndThen mapping closure")
    }
}
