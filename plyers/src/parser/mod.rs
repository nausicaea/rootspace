use std::io::{Read, Seek};
use crate::error::Error;

pub mod chain;
pub mod chain_with;
pub mod take_while;
pub mod engram;
pub mod one_of;
pub mod token;
pub mod map;
pub mod and_then;
pub mod chain_repeat;

pub trait Parser {
    type Item;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read + Seek;

    fn chain<P>(self, second: P) -> chain::Chain<Self, P>
    where
        Self: Sized,
    {
        chain::Chain::new(self, second)
    }

    fn chain_with<Q, F>(self, func: F) -> chain_with::ChainWith<Self, F>
    where
        Self: Sized,
        Q: Parser,
        F: Fn(&Self::Item) -> Q,
    {
        chain_with::ChainWith::new(self, func)
    }

    fn map<J, F>(self, func: F) -> map::Map<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> J,
    {
        map::Map::new(self, func)
    }

    fn and_then<J, F>(self, func: F) -> and_then::AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Item) -> Result<J, Error>,
    {
        and_then::AndThen::new(self, func)
    }

    fn chain_repeat<Q, R>(self, at_least_once: Q, until: R) -> chain_repeat::ChainRepeat<Self, Q, R>
    where
        Self: Sized,
        Q: Parser,
        R: Parser,
    {
        chain_repeat::ChainRepeat::new(self, at_least_once, until)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Seek};
    use crate::parser::take_while::take_while;
    use crate::parser::token::token;
    use crate::types::{COUNT_TYPES, CountType, DATA_TYPES, DataType, FORMAT_TYPES, FormatType, KEYWORDS};

    use super::Parser;
    use super::{engram::engram, one_of::one_of};

    #[allow(dead_code)]
    #[derive(Debug)]
    enum ParserProduct {
        Format { ft: FormatType, v: String },
        Element { n: String, c: usize },
        Property { dt: DataType, n: String },
        ListProperty { ct: CountType, dt: DataType, n: String },
        Comment(String),
        ObjInfo(String),
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

        fn parse<R>(self, r: &mut R) -> Result<Self::Item, crate::error::Error> where Self:Sized, R: Read + Seek {
            match self.keyword {
                b"format" => {
                    token(b' ')
                        .chain(one_of(FORMAT_TYPES))
                        .map(|(_, p)| match p {
                            b"ascii" => FormatType::Ascii,
                            b"binary_little_endian" => FormatType::BinaryLittleEndian,
                            b"binary_big_endian" => FormatType::BinaryBigEndian,
                            _ => unreachable!(),
                        })
                        .chain(token(b' '))
                        .chain(take_while(|b| b != b'\n'))
                        .and_then(|((ft, _), v)| {
                            String::from_utf8(v)
                                .map_err(|e| e.into())
                                .map(|v| ParserProduct::Format { ft, v })
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
                },
                b"property" => {
                    token(b' ')
                        .chain(one_of(DATA_TYPES))
                        .map(|(_, dt)| {
                            match dt {
                                b"uint8" | b"uchar" => DataType::U8,
                                b"int8" | b"char" => DataType::I8,
                                b"uint16" | b"ushort" => DataType::U16,
                                b"int16" | b"short" => DataType::I16,
                                b"uint32" | b"uint" => DataType::U32,
                                b"int32" | b"int" => DataType::I32,
                                b"float32" | b"float" => DataType::F32,
                                b"float64" | b"double" => DataType::F64,
                                _ => unreachable!(),
                            }
                        })
                        .chain(token(b' '))
                        .chain(take_while(|b| b != b'\n'))
                        .and_then(|((dt, _), n)| {
                            String::from_utf8(n)
                                .map_err(|e| e.into())
                                .map(|n| ParserProduct::Property { dt, n })
                        })
                        .parse(r)
                },
                b"property list" => {
                    token(b' ')
                        .chain(one_of(COUNT_TYPES))
                        .map(|(_, ct)| {
                            match ct {
                                b"uint8" | b"uchar" => CountType::U8,
                                b"uint16" | b"ushort" => CountType::U16,
                                b"uint32" | b"uint" => CountType::U32,
                                _ => unreachable!(),
                            }
                        })
                        .chain(token(b' '))
                        .chain(one_of(DATA_TYPES))
                        .map(|((ct, _), dt)| {
                            let dt = match dt {
                                b"uint8" | b"uchar" => DataType::U8,
                                b"int8" | b"char" => DataType::I8,
                                b"uint16" | b"ushort" => DataType::U16,
                                b"int16" | b"short" => DataType::I16,
                                b"uint32" | b"uint" => DataType::U32,
                                b"int32" | b"int" => DataType::I32,
                                b"float32" | b"float" => DataType::F32,
                                b"float64" | b"double" => DataType::F64,
                                _ => unreachable!(),
                            };
                            (ct, dt)
                        })
                        .chain(token(b' '))
                        .chain(take_while(|b| b != b'\n'))
                        .and_then(|(((ct, dt), _), n)| {
                            String::from_utf8(n)
                                .map_err(|e| e.into())
                                .map(|n| ParserProduct::ListProperty { ct, dt, n })
                        })
                        .parse(r)
                },
                b"comment" => {
                    token(b' ')
                        .chain(take_while(|b| b != b'\n'))
                        .and_then(|(_, n)| String::from_utf8(n).map(|s| ParserProduct::Comment(s)).map_err(|e| e.into()))
                        .parse(r)
                },
                b"obj_info" => {
                    token(b' ')
                        .chain(take_while(|b| b != b'\n'))
                        .and_then(|(_, n)| String::from_utf8(n).map(|s| ParserProduct::ObjInfo(s)).map_err(|e| e.into()))
                        .parse(r)
                },
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn can_i_get_a_ply_parser_out_of_this() {
        let path = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
        let file = std::fs::File::open(path).unwrap();
        let mut stream = std::io::BufReader::new(file);

        let r = engram(b"ply\n")
            .chain_repeat(
                one_of(KEYWORDS)
                    .chain_with(|p| PlyDirectiveParser::new(p))
                    .map(|(_, pd)| pd),
                engram(b"end_header\n")
            )
            .map(|(_, pds, _)| pds)
            .parse(&mut stream)
            .unwrap();

        dbg!(r);
    }
}
