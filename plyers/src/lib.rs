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
//! ;: one or more newline characters (0x0a)
//! -: one or more space or tab characters (0x09 or 0x20)
//! W: any integral or floating point number
//! X: any integral number larger than zero
//! Y: any word (non-space, non-linebreak)
//! Z: any string (non-linebreak)
//!
//! Data Types:
//! H -> "ascii" | "binary_little_endian" | "binary_big_endian"
//! J -> "char" | "uchar" | "short" | "ushort" | "int" | "uint" | "float" | "double" | "int8" |
//! "uint8" | "int16" | "uint16" | "int32" | "uint32" | "float32" | "float64"
//! K -> "uchar" | "ushort" | "uint" | "uint8" | "uint16" | "uint32"
//!
//! Declarations:
//! A' -> "ply" ;
//! A'' -> "end_header" ;
//! D' -> "format" - H - "1.0" ;
//! E' -> "element" - Y - X ;
//! F' -> "comment" - Z ;
//! M' -> "obj_info" - Z ;
//! G' -> "property" - J - Y ;
//! G'' -> "property" - "list" - K - J - Y ;
//!
//! Grammar:
//! S -> A B
//! A -> A' D E A''
//! B -> W | W B
//! D -> D' | F D'
//! E -> E' G | F E' G | E' G E | F E' G E
//! F -> F' | M' | F' F | M' F
//! Ga -> G' | F G' | G' Ga | F G' Ga
//! Gb -> G'' | F G'' | G'' Gb | F G'' Gb
//! G -> Ga | Gb

pub mod parser;
pub mod types;

use std::{fs::File, io::Read, path::Path};

use file_manipulation::FilePathBuf;
use nom::error::VerboseError;

use crate::parser::error::convert_error;
pub use crate::{parser::parse_ply, types::Ply};

#[derive(Debug, thiserror::Error)]
pub enum PlyError {
    #[error(transparent)]
    FileError(#[from] file_manipulation::FileError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("{}", .0)]
    NomError(String),
}

pub fn load_ply<P: AsRef<Path>>(path: P) -> Result<Ply, PlyError> {
    let path = FilePathBuf::try_from(path.as_ref())?;
    let mut file = File::open(path)?;
    let mut input = Vec::new();
    file.read_to_end(&mut input)?;

    let r = parse_ply::<VerboseError<_>>(&input)
        .map(|(_, p)| p)
        .map_err(|e| match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => PlyError::NomError(convert_error(&input, e)),
            e @ nom::Err::Incomplete(_) => PlyError::NomError(format!("{}", e)),
        })?;

    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FILES: &'static [&'static str] = &[
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/cube.ply"), // Ascii Cube
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/bun_zipper.ply"), // Large Ascii File
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/surfaceAB.ply"), // Large Little Endian File
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/Baby_Kinect.ply"), // Large Big Endian File
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/db_tall_obstacle_0.ply"), // VTK generated
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/saved_terrain.ply"), // VCGLIB generated
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/pasillo_1.ply"), // Large Ascii File
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/changing_dtypes.ply"), // Large Ascii File
    ];

    #[test]
    fn load_ply_succeeds_for_test_files() {
        for &p in TEST_FILES {
            let r = load_ply(p);
            if let Err(e) = r {
                panic!("{}", e)
            }
        }
    }
}
