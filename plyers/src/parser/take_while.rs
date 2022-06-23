use crate::error::Error;

use super::Parser;

pub struct TakeWhile {

}

impl Parser for TakeWhile {
    type Item = Vec<u8>;
    type Error = Error;

    fn next<R>(&mut self, _r: &mut R) -> std::task::Poll<Result<Self::Item, Self::Error>> where R: std::io::Read {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
