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

pub mod types;

use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use combine::{Parser, parser, ParseError, parser::byte::{bytes}, stream::read::Stream};
use thiserror::Error as ThisError;
use self::types::Ply;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("received byte sequence {:?}, but expected one of {:?}", .received, .expected)]
pub struct OneOfManyError {
    received: Vec<u8>,
    expected: &'static [&'static [u8]],
}

impl OneOfManyError {
    pub fn new(received: Vec<u8>, expected: &'static [&'static [u8]]) -> Self {
        OneOfManyError { received, expected }
    }
}

parser! {
    pub fn ply_parser[Input]()(Input) -> Ply 
    where [
        Input: combine::Stream<Token = u8>,
        <Input as StreamOnce>::Range: &[u8],
    ]
    {
        bytes(b"ply")
    }
}

pub fn load_ply<P: AsRef<Path>>(p: P) -> Result<Ply, Error> {
    let p = p.as_ref().to_path_buf();
    let file = File::open(&p)?;
    // let mut reader = BufReader::new(file);
    
    let mut parser = bytes(b"ply");

    let ply = parser.parse(Stream::new(file))?;

    Ok(ply)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_ply_fails_if_the_file_does_not_begin_with_a_magic_string() {
        let p = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/garbage.ply"));
        assert!(p.is_err());
    }

    #[test]
    fn load_ply_fails_if_the_file_does_not_have_a_header_terminator() {
        let p = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/incomplete_header.ply"));
        assert!(p.is_err());
    }

    // #[test]
    // fn load_ply_successfully_loads_minimal_ascii_ply_file() {
    //     let p = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
    //     match p {
    //         Err(e) => panic!("{:?}", e),
    //         Ok(p) => {
    //             let pd = &p.descriptor;
    //             assert_eq!(
    //                 pd.format_type,
    //                 FormatType::Ascii,
    //                 "File format is not ASCII, but {:?}",
    //                 &pd.format_type
    //             );
    //             assert_eq!(
    //                 pd.format_version, "1.0",
    //                 "File version is not 1.0, but {}",
    //                 &pd.format_version
    //             );
    //             assert_eq!(
    //                 pd.elements.len(),
    //                 1,
    //                 "Number of element descriptors is not one, but {}",
    //                 pd.elements.len()
    //             );
    //             assert_eq!(
    //                 pd.comments.len(),
    //                 0,
    //                 "Number of comments is not zero, but {}",
    //                 pd.comments.len()
    //             );
    //             assert_eq!(
    //                 pd.obj_info.len(),
    //                 0,
    //                 "Number of obj_info is not zero, but {}",
    //                 pd.obj_info.len()
    //             );

    //             let ed = &pd.elements[0];
    //             assert_eq!(
    //                 ed.name, "vertex",
    //                 "Name of element descriptor is not vertex, but {}",
    //                 &ed.name
    //             );
    //             assert_eq!(ed.count, 1, "Number of element instances is not one, but {}", ed.count);
    //             assert_eq!(
    //                 ed.properties.len(),
    //                 1,
    //                 "Number of element properties is not one, but {}",
    //                 ed.properties.len()
    //             );

    //             assert_eq!(
    //                 p.property_data.len(),
    //                 4,
    //                 "Property data length is not three, but {}",
    //                 p.property_data.len()
    //             );
    //             assert_eq!(
    //                 p.list_property_data.len(),
    //                 0,
    //                 "List property data length is not zero, but {}",
    //                 p.list_property_data.len()
    //             );

    //             let pdat = f32::from_ne_bytes(p.property_data.try_into().unwrap());
    //             assert_eq!(pdat, 1.0f32);
    //         }
    //     }
    // }

    // #[test]
    // fn load_ply_accepts_comments_and_obj_info_almost_anywhere_in_the_header() {
    //     let p: Ply = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/heavy_comments_ascii.ply")).unwrap();
    //     dbg!(&p.descriptor.comments);
    //     assert_eq!(p.descriptor.comments.len(), 6);
    //     dbg!(&p.descriptor.obj_info);
    //     assert_eq!(p.descriptor.obj_info.len(), 4);
    // }

    // #[test]
    // fn load_ply_succeeds_in_loading_a_large_ascii_file() {
    //     let p: Ply = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/bun_zipper.ply")).unwrap();
    // }

    // #[test]
    // fn load_ply_succeeds_in_loading_a_large_binary_file() {
    //     let p: Ply = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/surfaceAB.ply")).unwrap();
    // }
}
