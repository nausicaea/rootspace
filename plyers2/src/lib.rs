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
