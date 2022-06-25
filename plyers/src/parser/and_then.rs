use std::io::{Read, Seek};

use super::Parser;

#[derive(Debug)]
pub struct AndThen<P, F> {
    a: P,
    func: F,
}

impl<P, F> AndThen<P, F> {
    pub fn new(a: P, func: F) -> Self {
        AndThen {
            a,
            func,
        }
    }
}

impl<P, J, F> Parser for AndThen<P, F>
where
    P: Parser,
    F: FnMut(P::Item) -> Result<J, crate::error::Error>,
{
    type Item = J;

    fn parse<R>(mut self, r: &mut R) -> Result<Self::Item, crate::error::Error> where Self:Sized, R: Read + Seek {
        let product = self.a.parse(r)?;
        (self.func)(product)
    }
}
