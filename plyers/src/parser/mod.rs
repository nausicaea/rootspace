use std::io::Read;
use crate::error::Error;

mod chain;
mod chain_with;
mod take_while;
mod engram;
mod one_of;
mod token;
mod map;
mod and_then;
mod chain_repeat;

pub trait Parser {
    type Item;

    fn parse<R>(&mut self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read;

    fn chain<P>(self, second: P) -> self::chain::Chain<Self, P> 
    where
        Self: Sized,
        P: Parser,
    {
        self::chain::Chain::new(self, second)
    }

    fn chain_with<Q, F>(self, func: F) -> self::chain_with::ChainWith<Self, F>
    where
        Self: Sized,
        Q: Parser,
        F: Fn(&Self::Item) -> Q,
    {
        self::chain_with::ChainWith::new(self, func)
    }

    fn map<J, F>(self, func: F) -> self::map::Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> J,
    {
        self::map::Map::new(self, func)
    }

    fn and_then<J, F>(self, func: F) -> self::and_then::AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> Result<J, crate::error::Error>,
    {
        self::and_then::AndThen::new(self, func)
    }

    fn chain_repeat<Q, R>(self, repeat: Q, until: R) -> self::chain_repeat::ChainRepeat<Self, Q, R>
    where
        Self: Sized,
        Q: Parser,
        R: Parser,
    {
        self::chain_repeat::ChainRepeat::new(self, repeat, until)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::take_while::take_while;
    use crate::parser::token::token;
    use crate::types::FormatType;

    use super::Parser;
    use super::{engram::engram, one_of::one_of};

    #[test]
    fn can_i_get_a_ply_parser_out_of_this() {
        let path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
        let mut file = std::fs::File::open(path).unwrap();

        #[derive(Debug)]
        enum ParserProduct {
            Format { t: FormatType, v: String },
            Element { n: String, c: usize},
        }

        #[derive(Debug, Clone)]
        struct PlyDirectiveParser<'a> {
            keyword: &'a [u8],
        }

        impl<'a> PlyDirectiveParser<'a> {
            fn new(keyword: &'a [u8]) -> Self {
                PlyDirectiveParser {
                    keyword,
                }
            }
        }

        impl<'a> Parser for PlyDirectiveParser<'a> {
            type Item = ParserProduct;

            fn parse<R>(&mut self, r: &mut R) -> Result<Self::Item, crate::error::Error> where Self:Sized, R: std::io::Read {
                match self.keyword {
                    b"format" => {
                        token(b' ')
                            .chain(one_of(&[b"ascii", b"binary_little_endian", b"binary_big_endian"]))
                            .map(|(_, p)| match p {
                                b"ascii" => FormatType::Ascii,
                                b"binary_little_endian" => FormatType::BinaryLittleEndian,
                                b"binary_big_endian" => FormatType::BinaryBigEndian,
                                _ => unreachable!(),
                            })
                            .chain(take_while(|b| b != b'\n'))
                            .and_then(|(t, p)| {
                                String::from_utf8(p)
                                    .map(|v| ParserProduct::Format { t, v })
                                    .map_err(|e| e.into())
                            })
                            .parse(r)
                    },
                    b"element" => {
                        token(b' ')
                            .chain(take_while(|b| b != b' '))
                            .and_then(|(_, n)| String::from_utf8(n).map_err(|e| e.into()))
                            .chain(take_while(|b| b != b'\n'))
                            .and_then(|(n, c)| {
                                String::from_utf8(c)?
                                    .parse::<usize>()
                                    .map_err(|e| e.into())
                                    .map(|c| ParserProduct::Element { n, c })
                            })
                            .parse(r)
                    }
                    _ => todo!(),
                }
            }
        }

        engram(b"ply\n")
            .chain_repeat(
                one_of(&[b"format", b"element", b"property list", b"property", b"comment", b"obj_info"])
                    .chain_with(|p| PlyDirectiveParser::new(p)),
                engram(b"end_header\n")
            )
            .parse(&mut file)
            .unwrap();
    }
}
