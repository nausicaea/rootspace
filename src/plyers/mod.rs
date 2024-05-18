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

mod de;
mod ser;
pub mod types;

use std::{
    fs::File,
    io::{BufWriter, Read},
    path::Path,
};

use nom::error::VerboseError;

use crate::{
    file_manipulation,
    file_manipulation::{FilePathBuf, NewOrExFilePathBuf},
    plyers::{
        de::error::convert_error,
        ser::{write_ascii_values, write_be_values, write_header, write_le_values},
        types::{AmbiguousMixedPrimitive, FormatType, Ply, Values},
    },
};

#[derive(Debug, thiserror::Error)]
pub enum PlyError {
    #[error(transparent)]
    File(#[from] file_manipulation::FileError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{}", .0)]
    Nom(String),
    #[error("{}", .0)]
    Primitive(#[from] AmbiguousMixedPrimitive),
}

pub fn parse_ply(input: &[u8]) -> Result<Ply, PlyError> {
    de::parse_ply::<VerboseError<_>>(input)
        .map(|(_, p)| p)
        .map_err(|e| match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => PlyError::Nom(convert_error(&input, e)),
            e @ nom::Err::Incomplete(_) => PlyError::Nom(format!("{}", e)),
        })
}

pub fn load_ply<P: AsRef<Path>>(path: P) -> Result<Ply, PlyError> {
    let path = FilePathBuf::try_from(path.as_ref())?;
    tracing::debug!("Opening PLY file at {}", path.display());
    let mut file = File::open(path.clone())?;
    let mut input = Vec::new();
    tracing::debug!("Reading entire contents of file at {}", path.display());
    file.read_to_end(&mut input)?;

    parse_ply(&input)
}

pub fn save_ply<P: AsRef<Path>>(ply: &Ply, path: P) -> Result<(), PlyError> {
    let path = NewOrExFilePathBuf::try_from(path.as_ref())?;
    tracing::debug!("Creating PLY file at {}", path.display());
    let file = File::create(path.clone())?;
    let mut f = BufWriter::new(file);

    tracing::debug!("Writing PLY header to file at {}", path.display());
    write_header(&mut f, &ply.descriptor)?;

    match ply.descriptor.format_type {
        FormatType::BinaryLittleEndian => {
            for e_desc in ply.descriptor.elements.values() {
                let e_count = e_desc.count;
                for e_idx in 0..e_count {
                    for (p_id, p_desc) in &e_desc.properties {
                        let (p_prim, p_values) = &ply.data[p_id];
                        match p_values {
                            Values::I8(values) => write_le_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::U8(values) => write_le_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::I16(values) => write_le_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::U16(values) => write_le_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::I32(values) => write_le_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::U32(values) => write_le_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::I64(values) => write_le_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::U64(values) => write_le_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::F32(values) => write_le_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::F64(values) => write_le_values(&mut f, p_prim, p_desc, values, e_idx)?,
                        }
                    }
                }
            }
        }
        FormatType::BinaryBigEndian => {
            for e_desc in ply.descriptor.elements.values() {
                let e_count = e_desc.count;
                for e_idx in 0..e_count {
                    for (p_id, p_desc) in &e_desc.properties {
                        let (p_prim, p_values) = &ply.data[p_id];
                        match p_values {
                            Values::I8(values) => write_be_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::U8(values) => write_be_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::I16(values) => write_be_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::U16(values) => write_be_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::I32(values) => write_be_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::U32(values) => write_be_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::I64(values) => write_be_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::U64(values) => write_be_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::F32(values) => write_be_values(&mut f, p_prim, p_desc, values, e_idx)?,
                            Values::F64(values) => write_be_values(&mut f, p_prim, p_desc, values, e_idx)?,
                        }
                    }
                }
            }
        }
        FormatType::Ascii => {
            for e_desc in ply.descriptor.elements.values() {
                let e_count = e_desc.count;
                for e_idx in 0..e_count {
                    for (p_idx, (p_id, p_desc)) in e_desc.properties.iter().enumerate() {
                        let is_last_property = p_idx == e_desc.properties.len() - 1;
                        let (p_prim, p_values) = &ply.data[p_id];
                        match p_values {
                            Values::I8(values) => {
                                write_ascii_values(&mut f, p_prim, p_desc, values, e_idx, is_last_property)?
                            }
                            Values::U8(values) => {
                                write_ascii_values(&mut f, p_prim, p_desc, values, e_idx, is_last_property)?
                            }
                            Values::I16(values) => {
                                write_ascii_values(&mut f, p_prim, p_desc, values, e_idx, is_last_property)?
                            }
                            Values::U16(values) => {
                                write_ascii_values(&mut f, p_prim, p_desc, values, e_idx, is_last_property)?
                            }
                            Values::I32(values) => {
                                write_ascii_values(&mut f, p_prim, p_desc, values, e_idx, is_last_property)?
                            }
                            Values::U32(values) => {
                                write_ascii_values(&mut f, p_prim, p_desc, values, e_idx, is_last_property)?
                            }
                            Values::I64(values) => {
                                write_ascii_values(&mut f, p_prim, p_desc, values, e_idx, is_last_property)?
                            }
                            Values::U64(values) => {
                                write_ascii_values(&mut f, p_prim, p_desc, values, e_idx, is_last_property)?
                            }
                            Values::F32(values) => {
                                write_ascii_values(&mut f, p_prim, p_desc, values, e_idx, is_last_property)?
                            }
                            Values::F64(values) => {
                                write_ascii_values(&mut f, p_prim, p_desc, values, e_idx, is_last_property)?
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use super::*;
    use crate::plyers::types::{
        CountType, DataType, ElementDescriptor, ElementId, FormatType, ObjInfoDescriptor, Ply, PlyDescriptor,
        Primitive, PropertyDescriptor, PropertyId, Values,
    };

    #[test]
    fn load_ply_parses_jasmin6_correctly() {
        let ply = load_ply(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/valid/jasmin6.ply")).unwrap();
        let expected = Ply {
            descriptor: PlyDescriptor {
                format_type: FormatType::Ascii,
                comments: Vec::new(),
                obj_info: vec![
                    ObjInfoDescriptor("jasmin6-00000.jpg".into()),
                    ObjInfoDescriptor("jasmin6-00001.jpg".into()),
                    ObjInfoDescriptor("jasmin6-00002.jpg".into()),
                    ObjInfoDescriptor("jasmin6-00003.jpg".into()),
                    ObjInfoDescriptor("jasmin6-00004.jpg".into()),
                    ObjInfoDescriptor("jasmin6-00005.jpg".into()),
                ],
                elements: vec![
                    (
                        ElementId(0),
                        ElementDescriptor {
                            name: "vertex".into(),
                            count: 24,
                            comments: Vec::new(),
                            obj_info: Vec::new(),
                            properties: vec![
                                (
                                    PropertyId(0),
                                    PropertyDescriptor::Scalar {
                                        name: "x".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(1),
                                    PropertyDescriptor::Scalar {
                                        name: "y".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(2),
                                    PropertyDescriptor::Scalar {
                                        name: "z".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(3),
                                    PropertyDescriptor::Scalar {
                                        name: "nx".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(4),
                                    PropertyDescriptor::Scalar {
                                        name: "ny".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(5),
                                    PropertyDescriptor::Scalar {
                                        name: "nz".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(6),
                                    PropertyDescriptor::Scalar {
                                        name: "s".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(7),
                                    PropertyDescriptor::Scalar {
                                        name: "t".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                            ]
                            .into_iter()
                            .collect::<BTreeMap<_, _>>(),
                        },
                    ),
                    (
                        ElementId(1),
                        ElementDescriptor {
                            name: "face".into(),
                            count: 6,
                            comments: Vec::new(),
                            obj_info: Vec::new(),
                            properties: vec![(
                                PropertyId(8),
                                PropertyDescriptor::List {
                                    name: "vertex_indices".into(),
                                    count_type: CountType::U8,
                                    data_type: DataType::U32,
                                    comments: Vec::new(),
                                    obj_info: Vec::new(),
                                },
                            )]
                            .into_iter()
                            .collect::<BTreeMap<_, _>>(),
                        },
                    ),
                    (
                        ElementId(2),
                        ElementDescriptor {
                            name: "pass".into(),
                            count: 6,
                            comments: Vec::new(),
                            obj_info: Vec::new(),
                            properties: vec![
                                (
                                    PropertyId(9),
                                    PropertyDescriptor::List {
                                        name: "face_indices".into(),
                                        count_type: CountType::U8,
                                        data_type: DataType::U32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(10),
                                    PropertyDescriptor::Scalar {
                                        name: "m0".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(11),
                                    PropertyDescriptor::Scalar {
                                        name: "m1".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(12),
                                    PropertyDescriptor::Scalar {
                                        name: "m2".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(13),
                                    PropertyDescriptor::Scalar {
                                        name: "m3".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(14),
                                    PropertyDescriptor::Scalar {
                                        name: "m4".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(15),
                                    PropertyDescriptor::Scalar {
                                        name: "m5".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(16),
                                    PropertyDescriptor::Scalar {
                                        name: "m6".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(17),
                                    PropertyDescriptor::Scalar {
                                        name: "m7".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(18),
                                    PropertyDescriptor::Scalar {
                                        name: "m8".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(19),
                                    PropertyDescriptor::Scalar {
                                        name: "m9".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(20),
                                    PropertyDescriptor::Scalar {
                                        name: "m10".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(21),
                                    PropertyDescriptor::Scalar {
                                        name: "m11".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(22),
                                    PropertyDescriptor::Scalar {
                                        name: "m12".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(23),
                                    PropertyDescriptor::Scalar {
                                        name: "m13".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(24),
                                    PropertyDescriptor::Scalar {
                                        name: "m14".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(25),
                                    PropertyDescriptor::Scalar {
                                        name: "m15".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(26),
                                    PropertyDescriptor::Scalar {
                                        name: "x".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(27),
                                    PropertyDescriptor::Scalar {
                                        name: "y".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(28),
                                    PropertyDescriptor::Scalar {
                                        name: "w".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(29),
                                    PropertyDescriptor::Scalar {
                                        name: "h".into(),
                                        data_type: DataType::F32,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                                (
                                    PropertyId(30),
                                    PropertyDescriptor::Scalar {
                                        name: "tex_index".into(),
                                        data_type: DataType::U8,
                                        comments: Vec::new(),
                                        obj_info: Vec::new(),
                                    },
                                ),
                            ]
                            .into_iter()
                            .collect::<BTreeMap<_, _>>(),
                        },
                    ),
                ]
                .into_iter()
                .collect::<BTreeMap<_, _>>(),
            },
            data: vec![
                (
                    PropertyId(0),
                    (
                        Primitive::Single,
                        Values::F32(vec![
                            0.037002, 0.036996, -0.036999, -0.036993, 0.036999, -0.036996, -0.037002, 0.036993,
                            0.037002, 0.036999, 0.036993, 0.036996, 0.036996, 0.036993, -0.037002, -0.036999,
                            -0.036999, -0.037002, -0.036996, -0.036993, 0.036999, 0.037002, -0.036993, -0.036996,
                        ]),
                    ),
                ),
                (
                    PropertyId(1),
                    (
                        Primitive::Single,
                        Values::F32(vec![
                            0.032001, -0.032002, -0.031999, 0.032004, 0.031999, 0.032002, -0.032001, -0.032004,
                            0.032001, 0.031999, -0.032004, -0.032002, -0.032002, -0.032004, -0.032001, -0.031999,
                            -0.031999, -0.032001, 0.032002, 0.032004, 0.031999, 0.032001, 0.032004, 0.032002,
                        ]),
                    ),
                ),
                (
                    PropertyId(2),
                    (
                        Primitive::Single,
                        Values::F32(vec![
                            -0.060002, -0.059996, -0.060001, -0.060006, 0.060001, 0.059996, 0.060002, 0.060006,
                            -0.060002, 0.060001, 0.060006, -0.059996, -0.059996, 0.060006, 0.060002, -0.060001,
                            -0.060001, 0.060002, 0.059996, -0.060006, 0.060001, -0.060002, -0.060006, 0.059996,
                        ]),
                    ),
                ),
                (
                    PropertyId(3),
                    (
                        Primitive::Single,
                        Values::F32(vec![
                            0.000062, 0.000062, 0.000062, 0.000062, -0.000062, -0.000062, -0.000062, -0.000062,
                            1.000000, 1.000000, 1.000000, 1.000000, -0.000034, -0.000034, -0.000034, -0.000034,
                            -1.000000, -1.000000, -1.000000, -1.000000, 0.000034, 0.000034, 0.000034, 0.000034,
                        ]),
                    ),
                ),
                (
                    PropertyId(4),
                    (
                        Primitive::Single,
                        Values::F32(vec![
                            -0.000085, -0.000085, -0.000085, -0.000085, 0.000085, 0.000085, 0.000085, 0.000085,
                            -0.000095, -0.000095, -0.000095, -0.000095, -1.000000, -1.000000, -1.000000, -1.000000,
                            0.000095, 0.000095, 0.000095, 0.000095, 1.000000, 1.000000, 1.000000, 1.000000,
                        ]),
                    ),
                ),
                (
                    PropertyId(5),
                    (
                        Primitive::Single,
                        Values::F32(vec![
                            -1.000000, -1.000000, -1.000000, -1.000000, 1.000000, 1.000000, 1.000000, 1.000000,
                            0.000025, 0.000025, 0.000025, 0.000025, -0.000013, -0.000013, -0.000013, -0.000013,
                            -0.000026, -0.000026, -0.000026, -0.000026, 0.000013, 0.000013, 0.000013, 0.000013,
                        ]),
                    ),
                ),
                (
                    PropertyId(6),
                    (
                        Primitive::Single,
                        Values::F32(vec![
                            1.000000, 0.000000, 0.046274, 0.995699, 0.787469, 0.000000, 0.169696, 1.000000, 1.000000,
                            0.924001, 0.000000, 0.152738, 1.000000, 0.964649, 0.000000, 0.122957, 1.000000, 0.906571,
                            0.000000, 0.163019, 0.033887, 0.000000, 0.861597, 1.000000,
                        ]),
                    ),
                ),
                (
                    PropertyId(7),
                    (
                        Primitive::Single,
                        Values::F32(vec![
                            1.000000, 0.916903, 0.000000, 0.107536, 1.000000, 0.834112, 0.000000, 0.202412, 0.095504,
                            1.000000, 0.925341, 0.000000, 0.109484, 1.000000, 0.916022, 0.000000, 0.099584, 1.000000,
                            0.924449, 0.000000, 1.000000, 0.129055, 0.000000, 0.897787,
                        ]),
                    ),
                ),
                (
                    PropertyId(8),
                    (
                        Primitive::Quads,
                        Values::U32(vec![
                            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                        ]),
                    ),
                ),
                (PropertyId(9), (Primitive::Single, Values::U32(vec![3, 1, 4, 5, 2, 0]))),
                (
                    PropertyId(10),
                    (
                        Primitive::Single,
                        Values::F32(vec![1.606138, 1.625754, -0.480422, -1.597932, 0.441293, -0.049902]),
                    ),
                ),
                (
                    PropertyId(11),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.377680, 0.408797, 0.632663, 0.428811, -0.706215, 2.092186]),
                    ),
                ),
                (
                    PropertyId(12),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.253921, 0.188426, 0.977627, 0.285758, -0.974627, -0.334516]),
                    ),
                ),
                (
                    PropertyId(13),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.241534, 0.179235, 0.929938, 0.271818, -0.927084, -0.318198]),
                    ),
                ),
                (
                    PropertyId(14),
                    (
                        Primitive::Single,
                        Values::F32(vec![-0.496447, -0.405338, -1.606825, -0.521590, 1.619993, 1.621019]),
                    ),
                ),
                (
                    PropertyId(15),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.811847, 2.015269, -0.389332, -0.912783, 0.359951, 0.272766]),
                    ),
                ),
                (
                    PropertyId(16),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.947651, 0.421430, -0.248646, -0.914065, 0.224057, 0.244900]),
                    ),
                ),
                (
                    PropertyId(17),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.901424, 0.400873, -0.236517, -0.869477, 0.213127, 0.232953]),
                    ),
                ),
                (
                    PropertyId(18),
                    (
                        Primitive::Single,
                        Values::F32(vec![-0.100194, 0.169712, -0.153255, 0.103609, -0.130623, -0.453854]),
                    ),
                ),
                (
                    PropertyId(19),
                    (
                        Primitive::Single,
                        Values::F32(vec![2.051921, 0.885265, 2.111936, 1.998766, 2.093756, 0.748648]),
                    ),
                ),
                (
                    PropertyId(20),
                    (
                        Primitive::Single,
                        Values::F32(vec![-0.377725, -0.944503, -0.295997, -0.433614, -0.324182, 0.966083]),
                    ),
                ),
                (
                    PropertyId(21),
                    (
                        Primitive::Single,
                        Values::F32(vec![-0.359299, -0.898430, -0.281558, -0.412462, -0.308369, 0.918957]),
                    ),
                ),
                (
                    PropertyId(22),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.006884, 0.028156, 0.030603, 0.001764, 0.014361, 0.003482]),
                    ),
                ),
                (
                    PropertyId(23),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.050059, 0.017858, 0.030462, 0.025788, 0.008944, 0.072094]),
                    ),
                ),
                (
                    PropertyId(24),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.318657, 0.298807, 0.305354, 0.296511, 0.301228, 0.336362]),
                    ),
                ),
                (
                    PropertyId(25),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.498234, 0.479353, 0.485581, 0.477169, 0.481656, 0.515076]),
                    ),
                ),
                (
                    PropertyId(26),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.451387, 0.459713, 0.485658, 0.417102, 0.465644, 0.473894]),
                    ),
                ),
                (
                    PropertyId(27),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.382772, 0.489358, 0.356537, 0.349984, 0.331910, 0.436498]),
                    ),
                ),
                (
                    PropertyId(28),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.136837, 0.173210, 0.128915, 0.143695, 0.129949, 0.114713]),
                    ),
                ),
                (
                    PropertyId(29),
                    (
                        Primitive::Single,
                        Values::F32(vec![0.293662, 0.179877, 0.308941, 0.302289, 0.305762, 0.187858]),
                    ),
                ),
                (PropertyId(30), (Primitive::Single, Values::U8(vec![0, 1, 2, 3, 4, 5]))),
            ]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        };

        eprintln!("{:#?} \n {:#?}", &ply, &expected);

        assert_eq!(ply, expected);
    }

    #[rstest::rstest]
    fn load_ply_succeeds_for_test_files(#[files("tests/valid/*.ply")] path: PathBuf) {
        if let Err(e) = load_ply(&path) {
            panic!("{}: {}", path.display(), e);
        }
    }

    #[rstest::rstest]
    fn roundtrip_save_ply_succeeds_for_test_files(#[files("tests/valid/*.ply")] path: PathBuf) {
        let ply = load_ply(&path).unwrap();
        let tmp = tempfile::Builder::new()
            .prefix(path.file_stem().unwrap())
            .suffix(path.extension().unwrap())
            .tempfile()
            .unwrap();
        if let Err(e) = save_ply(&ply, tmp.path()) {
            panic!("{}: {}", path.display(), e);
        }
        match load_ply(tmp.path()) {
            Ok(ply2) => {
                if ply != ply2 {
                    persist_failures(&path, tmp);
                    assert_eq!(&ply.descriptor, &ply2.descriptor, "differing headers");
                    assert_eq!(&ply.data, &ply2.data, "differing data");
                }
            }
            Err(e) => {
                persist_failures(&path, tmp);
                panic!("{}", e);
            }
        }
    }

    #[rstest::rstest]
    fn parse_ply_crashes(#[files("tests/crashes/*.afl")] path: PathBuf) {
        let mut file = File::open(&path).unwrap();
        let mut input = Vec::new();
        file.read_to_end(&mut input).unwrap();
        let _ = parse_ply(&input);
    }

    fn persist_failures(source: &Path, tmp: tempfile::NamedTempFile) {
        let persist_path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/tests/save_ply_succeeds_for_test_files"
        ));
        if !persist_path.is_dir() {
            std::fs::create_dir_all(persist_path).unwrap();
        }
        let persist_path = persist_path.join(source.file_name().unwrap());
        tmp.persist(&persist_path).unwrap();
        eprintln!("diff {} {}", source.display(), persist_path.display());
    }
}
