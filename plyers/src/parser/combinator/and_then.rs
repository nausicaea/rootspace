use std::io::{Read, Seek};

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

impl<P, J, K, F> Parser for AndThen<P, F>
where
    P: Parser,
    P::Error: std::error::Error + 'static,
    K: std::error::Error + 'static,
    F: FnMut(P::Item) -> Result<J, K>,
{
    type Error = Box<dyn std::error::Error + 'static>;
    type Item = J;

    fn parse<R>(mut self, r: &mut R) -> Result<Self::Item, Self::Error>
    where
        Self: Sized,
        R: Read + Seek,
    {
        let product = self.a.parse(r)?;
        let product = (self.func)(product)?;
        Ok(product)
    }
}
