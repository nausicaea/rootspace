use std::num::{ParseIntError, ParseFloatError};
use nom::{
    combinator::{all_consuming, flat_map, map},
    IResult,
};

use self::{header::header, body::body_fct};
use crate::types::Ply;

mod common;
mod header;
mod body;

#[derive(Debug, thiserror::Error)]
pub enum ParseNumError {
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
}

pub fn parse_ply(input: &[u8]) -> IResult<&[u8], Ply> {
    all_consuming(flat_map(header, |descriptor| {
        map(
            body_fct(descriptor.clone()),
            move |(property_data, list_property_data)| Ply {
                descriptor: descriptor.clone(),
                property_data,
                list_property_data,
            },
        )
    }))(input)
}

#[cfg(test)]
mod tests {
    use crate::types::{PlyDescriptor, FormatType, ElementDescriptor, PropertyDescriptor, DataType};

    use super::*;

    #[test]
    fn parse_ply_minimal_ascii_parses_correctly() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
        assert_eq!(
            parse_ply(&input[..]),
            Ok((
                &b""[..],
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
                    property_data: vec![0x00, 0x00, 0x80, 0x3f],
                    list_property_data: Vec::new(),
                }
            ))
        );
    }
}
