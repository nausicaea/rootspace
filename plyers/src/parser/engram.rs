use std::io::Read;
use std::task::Poll;
use crate::utilities::read_byte;
use crate::error::Error;
use super::Parser;

pub struct Engram<'a> {
    e: &'a [u8],
    idx: usize,
    state: Poll<Option<u8>>,
}

impl<'a> Parser for Engram<'a> {
    type Item = ();
    type Error = Error;

    fn next<R: Read>(&mut self, r: &mut R) -> Poll<Result<(), Error>> {
        match self.state {
            Poll::Ready(Some(b)) => return Poll::Ready(Err(Error::UnexpectedByte(b))),
            Poll::Ready(None) => return Poll::Ready(Ok(())),
            Poll::Pending => (),
        }

        let byte = read_byte(r)?;

        if byte == self.e[self.idx] {
            self.idx += 1;

            if self.idx >= self.e.len() {
                self.state = Poll::Ready(None);
                return Poll::Ready(Ok(()));
            }
        } else {
            self.state = Poll::Ready(Some(byte));
            return Poll::Ready(Err(Error::UnexpectedByte(byte)));
        }

        Poll::Pending
    }
}

pub fn engram(e: &[u8]) -> Engram {
    Engram { e, idx: 0, state: Poll::Pending, }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engram_parses_a_single_fixed_word() {
        let source = "hello";
        let mut stream = source.as_bytes();

        let mut p = engram(b"hello");

        let r = p.parse(&mut stream);

        assert!(r.is_ok());
    }

    #[test]
    fn engram_fails_on_the_first_wrong_byte() {
        let source = "hallo";
        let mut stream = source.as_bytes();

        let mut p = engram(b"hello");

        let r = p.parse(&mut stream);

        match r {
            Err(Error::UnexpectedByte(b'a')) => (),
            other => panic!("Expected Error::UnexpectedByte(b'a'), got: {:?}", other),
        }
    }

    #[test]
    fn engram_returns_the_same_result_if_called_after_completion() {
        let source = "hello";
        let mut stream = source.as_bytes();

        let mut p = engram(b"hello");

        let _ = p.parse(&mut stream);

        match p.next(&mut stream) {
            Poll::Ready(Ok(())) => (),
            r => panic!("{:?}", r),
        }
    }
}
