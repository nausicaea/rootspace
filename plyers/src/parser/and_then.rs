use super::Parser;

#[derive(Debug)]
pub struct AndThen<P, F> {
    a: P,
    func: F,
}

impl<P, J, F> AndThen<P, F> 
where
    P: Parser,
    F: FnMut(P::Item) -> Result<J, crate::error::Error>,
{
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

    fn parse<R>(&mut self, r: &mut R) -> Result<Self::Item, crate::error::Error> where Self:Sized, R: std::io::Read {
        let product = self.a.parse(r)?;
        (self.func)(product)
    }
}
