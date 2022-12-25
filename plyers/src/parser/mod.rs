use std::num::{ParseFloatError, ParseIntError};

use nom::{
    combinator::{all_consuming, flat_map, map},
    error::{FromExternalError, ParseError},
    IResult,
};

use self::{body::body_fct, header::header};
use crate::types::Ply;

mod body;
mod common;
pub mod error;
mod header;

#[derive(Debug, thiserror::Error)]
pub enum ParseNumError {
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
}

pub fn parse_ply<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + 'a>(
    input: &'a [u8],
) -> IResult<&'a [u8], Ply, E> {
    all_consuming(flat_map(header, |descriptor| {
        map(
            body_fct(descriptor.clone()),
            move |data| Ply {
                descriptor: descriptor.clone(),
                data,
            },
        )
    }))(input)
}

#[cfg(test)]
mod tests {
    use nom::error::dbg_dmp;

    use super::*;
    use crate::types::{
        CommentDescriptor, DataType, ElementDescriptor, FormatType, ObjInfoDescriptor, PlyDescriptor,
        PropertyDescriptor,
    };

    const EMPTY: &'static [u8] = b"";

    #[test]
    fn parse_ply_minimal_ascii_parses_correctly() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
        assert_eq!(
            parse_ply::<nom::error::Error<_>>(&input[..]),
            Ok((
                EMPTY,
                Ply {
                    descriptor: PlyDescriptor {
                        format_type: FormatType::Ascii,
                        elements: vec![ElementDescriptor {
                            name: String::from("vertex"),
                            count: 1usize,
                            properties: vec![PropertyDescriptor {
                                data_type: DataType::F32,
                                name: String::from("x"),
                                comments: Vec::new(),
                                obj_info: Vec::new()
                            }],
                            list_properties: Vec::new(),
                            comments: Vec::new(),
                            obj_info: Vec::new()
                        }],
                        comments: Vec::new(),
                        obj_info: Vec::new()
                    },
                    data: vec![vec![0x00, 0x00, 0x80, 0x3f]],
                }
            ))
        );
    }

    #[test]
    fn parse_ply_fails_with_garbage() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/garbage.ply"));
        let r = dbg_dmp(parse_ply::<nom::error::Error<_>>, "GARBAGE")(&input[..]);
        assert!(r.is_err(), "{:?}", r.unwrap());
        print!("{:?}", r.unwrap_err())
    }

    #[test]
    fn parse_ply_fails_with_incomplete_header() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/incomplete_header.ply"));
        let r = nom::error::dbg_dmp(parse_ply::<nom::error::Error<_>>, "INCOMPLETE HEADER")(&input[..]);
        assert!(r.is_err(), "{:?}", r.unwrap());
        print!("{:?}", r.unwrap_err())
    }

    #[test]
    fn parse_ply_has_no_errors_for_ascii_with_heavy_comments() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/heavy_comments_ascii.ply"));
        assert_eq!(
            parse_ply::<nom::error::Error<_>>(&input[..]),
            Ok((
                EMPTY,
                Ply {
                    descriptor: PlyDescriptor {
                        format_type: FormatType::Ascii,
                        elements: vec![ElementDescriptor {
                            name: String::from("vertex"),
                            count: 1,
                            properties: vec![PropertyDescriptor {
                                data_type: DataType::F32,
                                name: String::from("x"),
                                comments: vec![
                                    CommentDescriptor(String::from("5. Comments are allowed here")),
                                    CommentDescriptor(String::from("6. Comments are allowed here"))
                                ],
                                obj_info: vec![ObjInfoDescriptor(String::from("4. ObjInfo are allowed here"))]
                            }],
                            list_properties: Vec::new(),
                            comments: vec![
                                CommentDescriptor(String::from("3. Comments are allowed here")),
                                CommentDescriptor(String::from("4. Comments are allowed here"))
                            ],
                            obj_info: vec![
                                ObjInfoDescriptor(String::from("2. ObjInfo are allowed here")),
                                ObjInfoDescriptor(String::from("3. ObjInfo are allowed here"))
                            ]
                        }],
                        comments: vec![
                            CommentDescriptor(String::from("1. Comments are allowed here")),
                            CommentDescriptor(String::from("2. Comments are allowed here"))
                        ],
                        obj_info: vec![ObjInfoDescriptor(String::from("1. ObjInfo are allowed here"))]
                    },
                    data: vec![vec![0, 0, 128, 63]],
                }
            ))
        );
    }
}
