use std::num::{ParseFloatError, ParseIntError};

use nom::{
    combinator::{all_consuming, flat_map, map},
    error::{context, ContextError, FromExternalError, ParseError},
    IResult,
};
use num_traits::NumCast;

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
    #[error("Unable to cast from source to output numeric type")]
    NumCastError,
}

pub fn parse_ply<
    'a,
    V: NumCast + 'a,
    I: NumCast + 'a,
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]> + 'a,
>(
    input: &'a [u8],
) -> IResult<&'a [u8], Ply<V, I>, E> {
    context(
        "plyers::parser::parse_ply",
        all_consuming(flat_map(header, |descriptor| {
            map(body_fct(descriptor.clone()), move |data| Ply {
                descriptor: descriptor.clone(),
                data: data
                    .into_iter()
                    .zip(descriptor.elements.iter().map(|e| &e.name))
                    .map(|(d, n)| (n.clone(), d))
                    .collect(),
            })
        })),
    )(input)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use either::Either::{self, Left};
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
        let expected_data: BTreeMap<String, Vec<Either<f32, Vec<u16>>>> =
            vec![("vertex".to_string(), vec![Left(1.0f32)])].into_iter().collect();
        assert_eq!(
            parse_ply::<f32, u16, nom::error::Error<_>>(&input[..]),
            Ok((
                EMPTY,
                Ply {
                    descriptor: PlyDescriptor {
                        format_type: FormatType::Ascii,
                        elements: vec![ElementDescriptor {
                            name: String::from("vertex"),
                            count: 1usize,
                            properties: vec![PropertyDescriptor::Scalar {
                                data_type: DataType::F32,
                                name: String::from("x"),
                                comments: Vec::new(),
                                obj_info: Vec::new()
                            }],
                            comments: Vec::new(),
                            obj_info: Vec::new()
                        }],
                        comments: Vec::new(),
                        obj_info: Vec::new()
                    },
                    data: expected_data,
                }
            ))
        );
    }

    #[test]
    fn parse_ply_fails_with_garbage() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/garbage.ply"));
        let r = dbg_dmp(parse_ply::<f32, u16, nom::error::Error<_>>, "GARBAGE")(&input[..]);
        assert!(r.is_err(), "{:?}", r.unwrap());
        print!("{:?}", r.unwrap_err())
    }

    #[test]
    fn parse_ply_fails_with_incomplete_header() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/incomplete_header.ply"));
        let r = nom::error::dbg_dmp(parse_ply::<f32, u16, nom::error::Error<_>>, "INCOMPLETE HEADER")(&input[..]);
        assert!(r.is_err(), "{:?}", r.unwrap());
        print!("{:?}", r.unwrap_err())
    }

    #[test]
    fn parse_ply_has_no_errors_for_ascii_with_heavy_comments() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/heavy_comments_ascii.ply"));
        let expected_data: BTreeMap<String, Vec<Either<f32, Vec<u16>>>> =
            vec![("vertex".to_string(), vec![Left(1.0f32)])].into_iter().collect();
        assert_eq!(
            parse_ply::<f32, u16, nom::error::Error<_>>(&input[..]),
            Ok((
                EMPTY,
                Ply {
                    descriptor: PlyDescriptor {
                        format_type: FormatType::Ascii,
                        elements: vec![ElementDescriptor {
                            name: String::from("vertex"),
                            count: 1,
                            properties: vec![PropertyDescriptor::Scalar {
                                data_type: DataType::F32,
                                name: String::from("x"),
                                comments: vec![
                                    CommentDescriptor(String::from("5. Comments are allowed here")),
                                    CommentDescriptor(String::from("6. Comments are allowed here"))
                                ],
                                obj_info: vec![ObjInfoDescriptor(String::from("4. ObjInfo are allowed here"))]
                            }],
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
                    data: expected_data,
                }
            ))
        );
    }
}
