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

pub mod parser;
