use std::io::Read;
use std::task::Poll;

mod and;
mod engram;
mod one_of;

pub trait Parser {
    type Item;
    type Error;

    fn next<R>(&mut self, r: &mut R, o: &mut usize) -> Poll<Result<Self::Item, Self::Error>> where R: Read;

    fn and<P>(self, second: P) -> self::and::And<Self, P> 
    where
        Self: Sized,
        P: Parser,
    {
        self::and::And::new(self, second)
    }
}

fn loop_<P: Parser, R: Read>(p: &mut P, src: &mut R, o: &mut usize) -> Result<P::Item, P::Error> {
    loop {
        match p.next(src, o) {
            Poll::Pending => continue,
            Poll::Ready(r) => break r,
        }
    }
}
