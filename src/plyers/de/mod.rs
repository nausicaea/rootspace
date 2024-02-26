use log::debug;
use std::num::{ParseFloatError, ParseIntError};

use nom::{
    combinator::{all_consuming, flat_map, map},
    error::{context, ContextError, FromExternalError, ParseError},
    IResult,
};

use self::{body::body_fct, header::header_fct};
use super::types::{ElementId, Ply, PropertyId};
use crate::urn::Urn;

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
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]> + 'a,
>(
    input: &'a [u8],
) -> IResult<&'a [u8], Ply, E> {
    let mut e_urn = Urn::<ElementId>::default();
    let mut p_urn = Urn::<PropertyId>::default();
    let e_urn_ref = &mut e_urn;
    let p_urn_ref = &mut p_urn;

    let r = context(
        "plyers::de::parse_ply#0",
        all_consuming(flat_map(header_fct(e_urn_ref, p_urn_ref), |descriptor| {
            debug!("Completed PLY header parsing, continuing to the body");
            context(
                "plyers::de::parse_ply#1",
                map(body_fct(descriptor.clone()), move |data| {
                    debug!("Completed PLY body parsing, assembling output data");
                    Ply {
                        descriptor: descriptor.clone(),
                        data,
                    }
                }),
            )
        })),
    )(input);

    r
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use nom::error::dbg_dmp;

    use super::super::types::{
        CommentDescriptor, DataType, ElementDescriptor, FormatType, ObjInfoDescriptor, PlyDescriptor, Primitive,
        PropertyDescriptor, Values,
    };
    use super::*;

    const EMPTY: &'static [u8] = b"";

    #[test]
    fn parse_ply_minimal_ascii_parses_correctly() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
        let expected_data: BTreeMap<PropertyId, (Primitive, Values)> =
            vec![(PropertyId(0), (Primitive::Single, Values::F32(vec![1.0])))]
                .into_iter()
                .collect();
        assert_eq!(
            parse_ply::<nom::error::Error<_>>(&input[..]),
            Ok((
                EMPTY,
                Ply {
                    descriptor: PlyDescriptor {
                        format_type: FormatType::Ascii,
                        elements: vec![(
                            ElementId(0),
                            ElementDescriptor {
                                name: String::from("vertex"),
                                count: 1usize,
                                properties: vec![(
                                    PropertyId(0),
                                    PropertyDescriptor::Scalar {
                                        data_type: DataType::F32,
                                        name: String::from("x"),
                                        comments: Vec::new(),
                                        obj_info: Vec::new()
                                    }
                                )]
                                .into_iter()
                                .collect(),
                                comments: Vec::new(),
                                obj_info: Vec::new()
                            }
                        )]
                        .into_iter()
                        .collect(),
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
        let expected_data: BTreeMap<PropertyId, (Primitive, Values)> =
            vec![(PropertyId(0), (Primitive::Single, Values::F32(vec![1.0])))]
                .into_iter()
                .collect();
        assert_eq!(
            parse_ply::<nom::error::Error<_>>(&input[..]),
            Ok((
                EMPTY,
                Ply {
                    descriptor: PlyDescriptor {
                        format_type: FormatType::Ascii,
                        elements: vec![(
                            ElementId(0),
                            ElementDescriptor {
                                name: String::from("vertex"),
                                count: 1,
                                properties: vec![(
                                    PropertyId(0),
                                    PropertyDescriptor::Scalar {
                                        data_type: DataType::F32,
                                        name: String::from("x"),
                                        comments: vec![
                                            CommentDescriptor(String::from("5. Comments are allowed here")),
                                            CommentDescriptor(String::from("6. Comments are allowed here"))
                                        ],
                                        obj_info: vec![ObjInfoDescriptor(String::from("4. ObjInfo are allowed here"))]
                                    }
                                )]
                                .into_iter()
                                .collect(),
                                comments: vec![],
                                obj_info: vec![]
                            }
                        )]
                        .into_iter()
                        .collect(),
                        comments: vec![
                            CommentDescriptor(String::from("3. Comments are allowed here")),
                            CommentDescriptor(String::from("4. Comments are allowed here"))
                        ],
                        obj_info: vec![
                            ObjInfoDescriptor(String::from("2. ObjInfo are allowed here")),
                            ObjInfoDescriptor(String::from("3. ObjInfo are allowed here"))
                        ]
                    },
                    data: expected_data,
                }
            ))
        );
    }
}
