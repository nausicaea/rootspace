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
use num_traits::NumCast;

use crate::parser::error::convert_error;
pub use crate::{
    parser::parse_ply,
    types::{
        DataType, Ply, ALPHA_PROPERTY, BLUE_PROPERTY, FACE_ELEMENT, GREEN_PROPERTY, NX_PROPERTY, NY_PROPERTY,
        NZ_PROPERTY, RED_PROPERTY, TEXTURE_U_PROPERTY, TEXTURE_V_PROPERTY, VERTEX_ELEMENT,
        VERTEX_INDICES_LIST_PROPERTY, X_PROPERTY, Y_PROPERTY, Z_PROPERTY,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum PlyError {
    #[error(transparent)]
    FileError(#[from] file_manipulation::FileError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("{}", .0)]
    NomError(String),
}

pub fn load_ply<V: NumCast, I: NumCast, P: AsRef<Path>>(path: P) -> Result<Ply<V, I>, PlyError> {
    let path = FilePathBuf::try_from(path.as_ref())?;
    let mut file = File::open(path)?;
    let mut input = Vec::new();
    file.read_to_end(&mut input)?;

    let r = parse_ply::<V, I, VerboseError<_>>(&input)
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
    use glob::glob;

    #[test]
    fn load_ply_succeeds_for_test_files() {
        for path_buf in
            glob(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/valid/*.ply")).expect("Failed to read glob pattern")
        {
            let path = path_buf.expect("Failed to read a globbed path");
            match load_ply::<f32, u32, _>(&path) {
                Err(e) => panic!("{}: {}", path.display(), e),
                _ => (),
            }
        }
    }
}
