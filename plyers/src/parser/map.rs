use super::Parser;

#[derive(Debug)]
pub struct Map<P, F> {
    a: P,
    func: F,
}

impl<P, J, F> Map<P, F> 
where
    P: Parser,
    F: FnMut(P::Item) -> J,
{
    pub fn new(a: P, func: F) -> Self {
        Map {
            a,
            func,
        }
    }
}

impl<P, J, F> Parser for Map<P, F>
where
    P: Parser,
    F: FnMut(P::Item) -> J,
{
    type Item = J;

    fn parse<R>(&mut self, r: &mut R) -> Result<Self::Item, crate::error::Error> where Self:Sized, R: std::io::Read {
        let product = self.a.parse(r)?;
        Ok((self.func)(product))
    }
}
