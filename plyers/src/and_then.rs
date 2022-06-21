use std::io::Read;
use std::task::Poll;
use crate::engram::Engram;
use crate::error::Error;

pub struct AndThen<'a> {
    first: Engram<'a>,
    second: Engram<'a>,
}

impl<'a> AndThen<'a> {
    pub fn next<R: Read>(&mut self, r: &mut R, o: &mut usize) -> Poll<Result<(), Error>> {
        match self.first.next(r, o) {
            Poll::Pending => (),
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Ready(Ok(())) => {
                match self.second.next(r, o) {
                    Poll::Pending => (),
                    Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                    Poll::Ready(Ok(())) => return Poll::Ready(Ok(())),
                }
            }
        }

        Poll::Pending
    }
}

pub fn and_then<'a>(first: Engram<'a>, second: Engram<'a>) -> AndThen<'a> {
    AndThen { first, second }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engram::engram;

    fn loop_<R: Read>(p: &mut AndThen, src: &mut R, o: &mut usize) -> Result<(), Error> {
        loop {
            match p.next(src, o) {
                Poll::Pending => continue,
                Poll::Ready(r) => break r,
            }
        }
    }

    #[test]
    fn and_then_chains_two_engrams() {
        let source = "helloworld";
        let mut stream = source.as_bytes();
        let mut offset = 0usize;

        let mut p = and_then(engram(b"hello"), engram(b"world"));

        let r = loop_(&mut p, &mut stream, &mut offset);

        assert!(r.is_ok());
    }
}
