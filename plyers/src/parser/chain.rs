use std::io::Read;
use std::task::Poll;
use super::Parser;
use crate::error::Error;

pub struct Chain<P, Q> {
    a: Option<P>,
    b: Option<Q>,
}

impl<P, Q> Chain<P, Q> {
    pub(crate) fn new(a: P, b: Q) -> Self {
        Chain { 
            a: Some(a), 
            b: Some(b), 
        }
    }
}

macro_rules! fuse {
    ($self:ident . $parser:ident . $($call:tt)+) => {
        match $self.$parser {
            Some(ref mut parser) => match parser.$($call)+ {
                std::task::Poll::Ready(Ok(_)) => {
                    $self.$parser = None;
                    std::task::Poll::Pending
                },
                std::task::Poll::Ready(Err(e)) => {
                    $self.$parser = None;
                    std::task::Poll::Ready(Err(e))
                }
                pending => pending,
            },
            None => std::task::Poll::Pending,
        }
    };
}

macro_rules! maybe {
    ($self:ident . $parser:ident . $($call:tt)+) => {
        match $self.$parser {
            Some(ref mut parser) => parser.$($call)+,
            None => std::task::Poll::Pending,
        }
    };
}

impl<P, Q> Parser for Chain<P, Q> 
where
    P: Parser,
    Q: Parser,
{
    type Item = Q::Item;

    fn next<R: Read>(&mut self, r: &mut R) -> Poll<Result<Q::Item, Error>> {
        match fuse!(self.a.next(r)) {
            Poll::Ready(Ok(_)) => maybe!(self.b.next(r)),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::engram::engram;

    #[test]
    fn chain_chains_two_parsers() {
        let source = "helloworld";
        let mut stream = source.as_bytes();

        let mut p = engram(b"hello")
            .chain(engram(b"world"));

        let r = p.parse(&mut stream);

        match r {
            Ok(()) => (),
            other => panic!("Expected Ok(()), got: {:?}", other),
        }
    }

    #[test]
    fn chain_allows_long_chains() {
        let source = "hello, world";
        let mut stream = source.as_bytes();

        let mut p = engram(b"hello")
            .chain(engram(b", "))
            .chain(engram(b"world"));

        let r = p.parse(&mut stream);

        match r {
            Ok(()) => (),
            other => panic!("Expected Ok(()), got: {:?}", other),
        }
    }

    #[test]
    fn chain_never_calls_first_again_once_complete_or_failed() {
        #[derive(Default)]
        struct P(bool);

        impl Parser for P {
            type Item = ();
            
             fn next<R>(&mut self, r: &mut R) -> Poll<Result<Self::Item, Error>> where R: Read {
                assert!(!self.0, "A::next() was called more than once");
                self.0 = true;
                Poll::Ready(Ok(()))
             }
        }

        let data = "A, b, C";
        let mut stream = data.as_bytes();
        let r = P::default()
            .chain(P::default())
            .parse(&mut stream);

        match r {
            Ok(()) => (),
            other => panic!("Expected Ok(()), got: {:?}", other),
        }
    }
}
