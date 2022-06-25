//! # Stanford PLY Parser
//!
//! ## Context-Free Grammar
//!
//! S: Start symbol
//! A: Header
//! B: Body
//! C: Declaration
//! D: Format declaration
//! E: Element declaration
//! F: Comment
//! G: Property declaration
//! H: Format type
//! J: Data type
//! K: Count type
//! M: Object Info declaration
//! W: any integral or floating point number
//! X: any integral number larger than zero
//! Y: any word (non-space, non-linebreak)
//! Z: any string (non-linebreak)
//!
//! S -> A B
//! A -> "ply" C "end_header"
//! B -> W
//! B -> W B
//! C -> D E
//! D' -> "format" H Z
//! D -> D'
//! D -> F D'
//! E' -> "element" Y X
//! E -> E' G
//! E -> F E' G
//! E -> E' G E
//! E -> F E' G E
//! F' -> "comment" Z
//! M' -> "obj_info" Z
//! F -> F'
//! F -> M'
//! F -> F' F
//! F -> M' F
//! G' -> "property" J Y
//! G'' -> "property" "list" K J Y
//! G -> G'
//! G -> G' G
//! G -> G''
//! G -> G'' G
//! G -> F G'
//! G -> F G' G
//! G -> F G''
//! G -> F G'' G
//! H -> "ascii" | "binary_little_endian" | "binary_big_endian"
//! J -> "char" | "uchar" | "short" | "ushort" | "int" | "uint" | "float" | "double" | "int8" |
//! "uint8" | "int16" | "uint16" | "int32" | "uint32" | "float32" | "float64"
//! K -> "uchar" | "ushort" | "uint" | "uint8" | "uint16" | "uint32"
//!

use std::{
    path::Path,
};
use std::io::{Cursor, Read};

use self::{
    error::Error,
    types::{ElementDescriptor, FormatType, ListPropertyDescriptor, Ply, PlyDescriptor, PropertyDescriptor},
};

pub mod types;
pub mod parser;
pub mod error;

pub fn load_ply<P: AsRef<Path>>(p: P) -> Result<Ply, Error> {
    todo!()
}

pub(crate) fn to_reader(source: &str) -> Cursor<&[u8]> {
    Cursor::new(source.as_bytes())
}

pub(crate) fn read_byte<R>(file: &mut R) -> Result<u8, Error>
    where
        R: Read,
{
    let mut byte_buf = [0u8; 1];
    let n = file.read(&mut byte_buf)?;
    if n == 0 {
        return Err(Error::UnexpectedEndOfFile);
    }
    Ok(byte_buf[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_ply_fails_if_the_file_does_not_begin_with_a_magic_string() {
        let p: Result<Ply, Error> = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/garbage.ply"));
        assert!(p.is_err());
    }

    #[test]
    fn load_ply_fails_if_the_file_does_not_have_a_header_terminator() {
        let p: Result<Ply, Error> = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/incomplete_header.ply"));
        assert!(p.is_err());
    }

    #[test]
    fn load_ply_successfully_loads_minimal_ascii_ply_file() {
        let p: Result<Ply, Error> = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
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
                    ed.list_properties.len(),
                    0,
                    "Number of element list properties is not zero, but {}",
                    ed.list_properties.len()
                );

                assert_eq!(
                    p.property_data.len(),
                    3,
                    "Property data length is not three, but {}",
                    p.property_data.len()
                );
                assert_eq!(
                    p.list_property_data.len(),
                    0,
                    "List property data length is not zero, but {}",
                    p.list_property_data.len()
                );

                let pdat = std::str::from_utf8(&p.property_data).unwrap();
                assert_eq!(pdat, "1.0");
            }
        }
    }
}