use std::io::Read;
use std::task::Poll;
use crate::error::Error;
use super::Parser;

pub struct And<F, S> {
    first: F,
    second: S,
}

impl<F, S> And<F, S> {
    pub(crate) fn new(first: F, second: S) -> Self {
        And { first, second }
    }
}

impl<F, S> Parser for And<F, S> 
where
    F: Parser<Error = Error>,
    S: Parser<Error = Error>,
{
    type Item = ();
    type Error = Error;

    fn next<R: Read>(&mut self, r: &mut R, o: &mut usize) -> Poll<Result<(), Error>> {
        match self.first.next(r, o) {
            Poll::Pending => (),
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Ready(Ok(_)) => {
                match self.second.next(r, o) {
                    Poll::Pending => (),
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Ready(Ok(_)) => return Poll::Ready(Ok(())),
                }
            }
        }

        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::engram::engram;
    use super::super::loop_;

    #[test]
    fn and_chains_two_engrams() {
        let source = "helloworld";
        let mut stream = source.as_bytes();
        let mut offset = 0usize;

        let mut p = engram(b"hello")
            .and(engram(b"world"));

        let r = loop_(&mut p, &mut stream, &mut offset);

        assert!(r.is_ok());
    }

    #[test]
    fn and_allows_long_chains() {
        let source = "hello, world";
        let mut stream = source.as_bytes();
        let mut offset = 0usize;

        let mut p = engram(b"hello")
            .and(engram(b", "))
            .and(engram(b"world"));

        let r = loop_(&mut p, &mut stream, &mut offset);

        assert!(r.is_ok());
    }
}
