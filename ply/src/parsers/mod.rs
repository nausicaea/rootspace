mod base;
mod body;
mod header;

use self::header::header;
use self::body::body;
use super::types::Ply;
use combine::{
    error::ParseError,
    parser::{
        item::value,
        Parser,
    },
    stream::RangeStream,
};

pub fn ply<'a, I>() -> impl Parser<Input = I, Output = Ply> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    header()
        .then(|h| (value(h.clone()), body(h)))
        .map(|(h, b)| Ply { header: h, body: b })
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::stream::state::State;

    #[test]
    fn ply_ascii() {
        let serialized_data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube-ascii.ply"));
        let r = ply().easy_parse(State::new(&serialized_data[..]));
        assert_ok!(r);
    }

    #[test]
    #[ignore]
    fn ply_be() {
        let serialized_data = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube-be.ply"));
        let r = ply().easy_parse(&serialized_data[..]);
        assert_ok!(r);

    }
}
