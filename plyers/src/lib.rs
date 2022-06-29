//! # Stanford PLY Parser
//!
//! ## Context-Free Grammar
//!
//! S: Start symbol
//! A: Header
//! B: Body
//! D: Format declaration
//! E: Element declaration
//! F: Comment or Object Info declaration
//! G: Property declaration
//! H: Format type
//! J: Data type
//! K: Count type
//! W: any integral or floating point number
//! X: any integral number larger than zero
//! Y: any word (non-space, non-linebreak)
//! Z: any string (non-linebreak)
//!
//! S -> A B
//! A -> "ply" D E "end_header"
//! B -> W | W B
//! D' -> "format" H Z
//! D -> D' | F D'
//! E' -> "element" Y X
//! E -> E' G | F E' G | E' G E | F E' G E
//! F' -> "comment" Z
//! M' -> "obj_info" Z
//! F -> F' | M' | F' F | M' F
//! G' -> "property" J Y
//! G'' -> "property" "list" K J Y
//! Ga -> G' | F G' | G' Ga | F G' Ga
//! Gb -> G'' | F G'' | G'' Gb | F G'' Gb
//! G -> Ga | Gb
//! H -> "ascii" | "binary_little_endian" | "binary_big_endian"
//! J -> "char" | "uchar" | "short" | "ushort" | "int" | "uint" | "float" | "double" | "int8" |
//! "uint8" | "int16" | "uint16" | "int32" | "uint32" | "float32" | "float64"
//! K -> "uchar" | "ushort" | "uint" | "uint8" | "uint16" | "uint32"
//!

use std::{
    fs::File,
    io::{BufReader, Read, Seek},
    path::Path,
};

use parser::{be_count, be_number, le_count, le_number};
use parser::base::bytes::Bytes;
use parser::base::empty::empty;
use parser::base::engram::engram;
use parser::base::lookahead::lookahead;
use parser::base::take_while::take_while;
use parser::combinator::repeat_exact::repeat_exact;
use parser::combinator::repeat_until::repeat_until;

use self::types::{FormatType, Ply};
use crate::{
    error::Error,
    parser::Parser,
    types::{
        CommentDescriptor, CountType, DataType, ElementDescriptor, ListPropertyDescriptor, ObjInfoDescriptor,
        PlyDescriptor, PropertyDescriptor,
    },
};

pub mod error;
pub mod parser;
pub mod ply;
pub mod types;

pub fn parse_ply<S: Read + Seek>(stream: &mut S) -> anyhow::Result<Ply> {
    let descriptor = ply::header().parse(stream)?;
    let mut property_data: Vec<u8> = Vec::new();
    let mut list_property_data: Vec<u8> = Vec::new();

    match descriptor.format_type {
        FormatType::Ascii => {
            for element in &descriptor.elements {
                for property in &element.properties {
                    let property_value = ply::ascii_number_parser(property.data_type).parse(stream)?;
                    property_data.extend(property_value);
                }

                for list_property in &element.list_properties {
                    let count = ply::ascii_usize_parser().parse(stream)?;
                    let property_values =
                        repeat_exact(ply::ascii_number_parser(list_property.data_type), count).parse(stream)?;
                    for property_value in property_values {
                        list_property_data.extend(property_value);
                    }
                }
            }
        }
        FormatType::BinaryLittleEndian => {
            for element in &descriptor.elements {
                for property in &element.properties {
                    let property_value = le_number::le_number(property.data_type).parse(stream)?;
                    property_data.extend(property_value);
                }

                for list_property in &element.list_properties {
                    let count = le_count::le_count(list_property.count_type).parse(stream)?;
                    let property_values =
                        repeat_exact(le_number::le_number(list_property.data_type), count).parse(stream)?;
                    for property_value in property_values {
                        list_property_data.extend(property_value);
                    }
                }
            }
        }
        FormatType::BinaryBigEndian => {
            for element in &descriptor.elements {
                for property in &element.properties {
                    let property_value = be_number::be_number(property.data_type).parse(stream)?;
                    property_data.extend(property_value);
                }

                for list_property in &element.list_properties {
                    let count = be_count::be_count(list_property.count_type).parse(stream)?;
                    let property_values =
                        repeat_exact(be_number::be_number(list_property.data_type), count).parse(stream)?;
                    for property_value in property_values {
                        list_property_data.extend(property_value);
                    }
                }
            }
        }
    }

    Ok(Ply {
        descriptor,
        property_data,
        list_property_data,
    })
}

pub fn load_ply<P: AsRef<Path>>(p: P) -> anyhow::Result<Ply> {
    let file = File::open(p)?;
    let mut reader = BufReader::new(file);

    parse_ply(&mut reader)
}

#[cfg(test)]
pub(crate) fn to_reader(source: &str) -> std::io::Cursor<&[u8]> {
    std::io::Cursor::new(source.as_bytes())
}

pub(crate) fn read_byte<R>(file: &mut R) -> Result<(u8, u64), Error>
where
    R: Read + Seek,
{
    let mut byte_buf = [0u8; 1];
    let position = file.stream_position()?;
    let n = file.read(&mut byte_buf)?;
    if n == 0 {
        return Err(Error::UnexpectedEndOfFile(position));
    }
    Ok((byte_buf[0], position))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_ply_fails_if_the_file_does_not_begin_with_a_magic_string() {
        let p: anyhow::Result<Ply> = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/garbage.ply"));
        assert!(p.is_err());
    }

    #[test]
    fn load_ply_fails_if_the_file_does_not_have_a_header_terminator() {
        let p: anyhow::Result<Ply> = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/incomplete_header.ply"));
        assert!(p.is_err());
    }

    #[test]
    fn load_ply_successfully_loads_minimal_ascii_ply_file() {
        let p: anyhow::Result<Ply> = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
        match p {
            Err(e) => panic!("{:?}", e),
            Ok(p) => {
                let pd = &p.descriptor;
                assert_eq!(
                    pd.format_type,
                    FormatType::Ascii,
                    "File format is not ASCII, but {:?}",
                    &pd.format_type
                );
                assert_eq!(
                    pd.format_version, "1.0",
                    "File version is not 1.0, but {}",
                    &pd.format_version
                );
                assert_eq!(
                    pd.elements.len(),
                    1,
                    "Number of element descriptors is not one, but {}",
                    pd.elements.len()
                );
                assert_eq!(
                    pd.comments.len(),
                    0,
                    "Number of comments is not zero, but {}",
                    pd.comments.len()
                );
                assert_eq!(
                    pd.obj_info.len(),
                    0,
                    "Number of obj_info is not zero, but {}",
                    pd.obj_info.len()
                );

                let ed = &pd.elements[0];
                assert_eq!(
                    ed.name, "vertex",
                    "Name of element descriptor is not vertex, but {}",
                    &ed.name
                );
                assert_eq!(ed.count, 1, "Number of element instances is not one, but {}", ed.count);
                assert_eq!(
                    ed.properties.len(),
                    1,
                    "Number of element properties is not one, but {}",
                    ed.properties.len()
                );

                assert_eq!(
                    p.property_data.len(),
                    4,
                    "Property data length is not three, but {}",
                    p.property_data.len()
                );
                assert_eq!(
                    p.list_property_data.len(),
                    0,
                    "List property data length is not zero, but {}",
                    p.list_property_data.len()
                );

                let pdat = f32::from_ne_bytes(p.property_data.try_into().unwrap());
                assert_eq!(pdat, 1.0f32);
            }
        }
    }

    #[test]
    fn load_ply_accepts_comments_and_obj_info_almost_anywhere_in_the_header() {
        let p: Ply = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/heavy_comments_ascii.ply")).unwrap();
        dbg!(&p.descriptor.comments);
        assert_eq!(p.descriptor.comments.len(), 6);
        dbg!(&p.descriptor.obj_info);
        assert_eq!(p.descriptor.obj_info.len(), 4);
    }

    #[test]
    fn load_ply_succeeds_in_loading_a_large_ascii_file() {
        let p: Ply = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/bun_zipper.ply")).unwrap();
    }

    #[test]
    fn load_ply_succeeds_in_loading_a_large_binary_file() {
        let p: Ply = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/surfaceAB.ply")).unwrap();
    }
}
