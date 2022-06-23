use std::task::Poll;
use super::Parser;
use crate::utilities::read_byte;
use crate::error::Error;

pub struct OneOf<'a> {
    es: &'a [&'a [u8]],
    idx: Vec<usize>,
    state: Poll<Option<usize>>,
}

impl<'a> Parser for OneOf<'a> {
    type Item = &'a [u8];
    type Error = Error;

    fn next<R>(&mut self, r: &mut R) -> std::task::Poll<Result<Self::Item, Self::Error>> where R: std::io::Read {
        let byte = read_byte(r)?;

        let mut found_match = false;
        for i in 0..self.es.len() {
            if byte == self.es[i][self.idx[i]] {
                found_match = true;
                self.idx[i] += 1;

                if self.idx[i] >= self.es[i].len() {
                    self.state = Poll::Ready(Some(i));
                    return Poll::Ready(Ok(self.es[i]));
                }
            }
        }

        if !found_match {
            self.state = Poll::Ready(None);
            return Poll::Ready(Err(Error::UnexpectedByte(byte)));
        }

        Poll::Pending
    }
}

pub fn one_of<'a>(engrams: &'a [&'a [u8]]) -> OneOf<'a> {
    OneOf {
        es: engrams,
        idx: vec![0; engrams.len()],
        state: Poll::Pending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_of_succeeds_on_the_first_engram_that_matches() {
        let source = "hello";
        let mut stream = source.as_bytes();

        let mut p = one_of(&[b"bye bye", b"hello"]);

        let r = p.parse(&mut stream);

        match r {
            Ok(b"hello") => (),
            other => panic!("Expected Ok(b\"hello\"), got: {:?}", other),
        }
    }

    #[test]
    fn one_of_fails_on_the_first_byte_that_does_not_match_any_engram() {
        let source = "bald eagle";
        let mut stream = source.as_bytes();

        let mut p = one_of(&[b"bye bye", b"hello"]);

        let r = p.parse(&mut stream);

        match r {
            Err(Error::UnexpectedByte(b'a')) => (),
            other => panic!("Expected Error::UnexpectedByte(b'a'), got: {:?}", other),
        }
    }
}
