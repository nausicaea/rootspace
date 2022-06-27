use std::io::{Read, Seek};
use crate::FormatType;
use crate::parser::Parser;
use crate::parser::take_while::take_while;
use crate::parser::token::token;
use crate::types::{COUNT_TYPES, CountType, DATA_TYPES, DataType, FORMAT_TYPES, Keyword};

#[derive(Debug)]
pub enum ParserProduct {
    Format { ft: FormatType, v: String },
    Element { n: String, c: usize },
    Property { dt: DataType, n: String },
    ListProperty { ct: CountType, dt: DataType, n: String },
    Comment(String),
    ObjInfo(String),
}

fn format_parser() -> impl Parser<Item = ParserProduct> {
    token(b' ')
        .chain(one_of(FORMAT_TYPES))
        .and_then(|(_, ft)| FormatType::try_from_bytes(ft).map_err(|e| e.into()))
        .chain(token(b' '))
        .chain(take_while(|b| b != b'\n'))
        .and_then(|((ft, _), v)| {
            String::from_utf8(v)
                .map_err(|e| e.into())
                .map(|v| ParserProduct::Format { ft, v })
        })
}

fn element_parser() -> impl Parser<Item = ParserProduct> {
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
}

fn property_parser() -> impl Parser<Item = ParserProduct> {
    token(b' ')
        .chain(one_of(DATA_TYPES))
        .and_then(|(_, dt)| DataType::try_from_bytes(dt).map_err(|e| e.into()))
        .chain(token(b' '))
        .chain(take_while(|b| b != b'\n'))
        .and_then(|((dt, _), n)| {
            String::from_utf8(n)
                .map_err(|e| e.into())
                .map(|n| ParserProduct::Property { dt, n })
        })
}

fn list_property_parser() -> impl Parser<Item = ParserProduct> {
    token(b' ')
        .chain(one_of(COUNT_TYPES))
        .and_then(|(_, ct)| CountType::try_from_bytes(ct).map_err(|e| e.into()))
        .chain(token(b' '))
        .chain(one_of(DATA_TYPES))
        .and_then(|((ct, _), dt)| DataType::try_from_bytes(dt).map_err(|e| e.into()).map(|dt| (ct, dt)))
        .chain(token(b' '))
        .chain(take_while(|b| b != b'\n'))
        .and_then(|(((ct, dt), _), n)| {
            String::from_utf8(n)
                .map_err(|e| e.into())
                .map(|n| ParserProduct::ListProperty { ct, dt, n })
        })
}

fn comment_parser() -> impl Parser<Item = ParserProduct> {
    token(b' ')
        .chain(take_while(|b| b != b'\n'))
        .and_then(|(_, n)| String::from_utf8(n).map(|s| ParserProduct::Comment(s)).map_err(|e| e.into()))
}

fn obj_info_parser() -> impl Parser<Item = ParserProduct> {
    token(b' ')
        .chain(take_while(|b| b != b'\n'))
        .and_then(|(_, n)| String::from_utf8(n).map(|s| ParserProduct::ObjInfo(s)).map_err(|e| e.into()))
}

#[derive(Debug, Clone)]
pub struct PlyDirective {
    keyword: Keyword,
}

impl PlyDirective {
    pub fn new(keyword: Keyword) -> Self {
        PlyDirective {
            keyword,
        }
    }
}

impl Parser for PlyDirective {
    type Item = ParserProduct;

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, crate::error::Error> where Self:Sized, R: Read + Seek {
        match self.keyword {
            Keyword::Format => format_parser().parse(r),
            Keyword::Element => element_parser().parse(r),
            Keyword::Property => property_parser().parse(r),
            Keyword::ListProperty => list_property_parser().parse(r),
            Keyword::Comment => comment_parser().parse(r),
            Keyword::ObjInfo => obj_info_parser().parse(r),
        }
    }
}