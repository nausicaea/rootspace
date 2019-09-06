mod base;
mod body;
mod header;

use self::{body::body, header::header};
use super::types::Ply;
use combine::{
    error::ParseError,
    parser::{item::value, Parser},
    stream::Stream,
};

pub fn ply<'a, I>() -> impl Parser<Input = I, Output = Ply> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    header()
        .then(|h| (value(h.clone()), body(h)))
        .map(|(h, b)| Ply { header: h, body: b })
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::stream::{buffered::BufferedStream, state::State, ReadStream};
    use std::fs::File;

    #[test]
    fn ply_ascii() {
        let data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube.ply");
        let file = File::open(data_path).unwrap();
        let stream = BufferedStream::new(State::new(ReadStream::new(file)), 32);
        let r = ply().parse(stream);
        assert!(r.is_ok());
    }

    #[test]
    #[ignore]
    fn ply_be() {
        let data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube-be.ply");
        let file = File::open(data_path).unwrap();
        let stream = BufferedStream::new(State::new(ReadStream::new(file)), 32);
        let r = ply().parse(stream);
        assert!(r.is_ok());
    }

    #[test]
    #[ignore]
    fn ply_le() {
        let data_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube-le.ply");
        let file = File::open(data_path).unwrap();
        let stream = BufferedStream::new(State::new(ReadStream::new(file)), 32);
        let r = ply().parse(stream);
        assert!(r.is_ok());
    }
}
