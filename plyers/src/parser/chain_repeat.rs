use super::Parser;
use crate::error::Error;

#[derive(Debug)]
pub struct ChainRepeat<P, Q, R> {
    tail: Option<P>,
    repeat: Q,
    until: R,
}

impl<P, Q, R> ChainRepeat<P, Q, R>
where
    P: Parser,
    Q: Parser,
    R: Parser,
{
    pub fn new(tail: P, repeat: Q, until: R) -> Self {
        ChainRepeat { 
            tail: Some(tail),
            repeat, 
            until,
        }
    }
}

macro_rules! fuse {
    ($self:ident . $parser:ident . $($call:tt)+) => {
        match $self.$parser {
            Some(ref mut parser) => match parser.$($call)+ {
                Ok(product) => {
                    $self.$parser = None;
                    Ok(product)
                },
                Err(e) => {
                    $self.$parser = None;
                    Err(e)
                },
            },
            None => Err(Error::ParserExhausted),
        }
    };
}

impl<P, Q, R> Parser for ChainRepeat<P, Q, R>
where
    P: Parser,
    Q: Parser,
    R: Parser,
{
    type Item = (P::Item, Vec<Q::Item>, R::Item);

    fn parse<S>(&mut self, r: &mut S) -> Result<Self::Item, Error> where Self:Sized, S: std::io::Read {
        let tail_p = fuse!(self.tail.parse(r))?;

        todo!()
    }
}
