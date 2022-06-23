use std::io::Read;
use std::task::Poll;

mod and;
mod take_while;
mod engram;
mod one_of;

pub trait Parser {
    type Item;
    type Error;

    fn next<R>(&mut self, r: &mut R) -> Poll<Result<Self::Item, Self::Error>> where R: Read;

    fn and<P>(self, second: P) -> self::and::And<Self, P> 
    where
        Self: Sized,
        P: Parser,
    {
        self::and::And::new(self, second)
    }

    fn parse<R>(&mut self, reader: &mut R) -> Result<Self::Item, Self::Error>
    where
        R: Read,
    {
        loop {
            match self.next(reader) {
                Poll::Pending => continue,
                Poll::Ready(r) => break r,
            }
        }
    }
}
