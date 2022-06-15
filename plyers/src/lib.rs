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
//! I: Element type
//! J: Data type
//! K: Count type
//! L: Property type
//! M: Object Info declaration
//! W: any integral or floating point number
//! X: any integral number larger than zero
//! Y: any identifier
//! Z: any string
//!
//! S -> A B
//! A -> "ply" C "end_header"
//! B -> W
//! B -> W B
//! C -> D E
//! D' -> "format" H Z
//! D -> D'
//! D -> F D'
//! E' -> "element" I X
//! E -> E' G
//! E -> F E' G
//! E -> E' G E
//! E -> F E' G E
//! F' -> "comment" Z
//! F -> F'
//! F -> M'
//! F -> F' F
//! F -> M' F
//! G' -> "property" J L
//! G'' -> "property" "list" K J L
//! G -> G'
//! G -> G' G
//! G -> G''
//! G -> G'' G
//! G -> F G'
//! G -> F G' G
//! G -> F G''
//! G -> F G'' G
//! H -> "ascii" | "binary_little_endian" | "binary_big_endian"
//! I -> "vertex" | "face" | "edge" | Y
//! J -> "char" | "uchar" | "short" | "ushort" | "int" | "uint" | "float" | "double"
//! K -> "char" | "uchar" | "short" | "ushort" | "int" | "uint"
//! L -> "x" | "y" | "z" | "w" | Y
//! 

use std::{path::Path, fs::File, io::{SeekFrom, Seek}};

use thiserror::Error;

const PLY_HEADER_START: [u8; 4] = [0x70, 0x6c, 0x79, 0x0a];
const PLY_HEADER_END: [u8; 11] = [0x65, 0x6e, 0x64, 0x5f, 0x68, 0x65, 0x61, 0x64, 0x65, 0x72, 0x0a];
const FORMAT_KWD: [u8; 7] = [0x66, 0x6f, 0x72, 0x6d, 0x61, 0x74, 0x20];
const COMMENT_KWD: [u8; 8] = [0x63, 0x6f, 0x6d, 0x6d, 0x65, 0x6e, 0x74, 0x20];
const OBJ_INFO_KWD: [u8; 9] = [0x6f, 0x62, 0x6a, 0x5f, 0x69, 0x6e, 0x66, 0x6f, 0x20];
const ELEMENT_KWD: [u8; 8] = [0x65, 0x6c, 0x65, 0x6d, 0x65, 0x6e, 0x74, 0x20];
const PROPERTY_KWD: [u8; 9] = [0x70, 0x72, 0x6f, 0x70, 0x65, 0x72, 0x74, 0x79, 0x20];
const LIST_PROPERTY_KWD: [u8; 14] = [0x70, 0x72, 0x6f, 0x70, 0x65, 0x72, 0x74, 0x79, 0x20, 0x6c, 0x69, 0x73, 0x74, 0x20];
const FORMAT_ASCII_KWD: [u8; 5] = [0x61, 0x73, 0x63, 0x69, 0x69];
const FORMAT_BLE_KWD: [u8; 20] = [0x62, 0x69, 0x6e, 0x61, 0x72, 0x79, 0x5f, 0x6c, 0x69, 0x74, 0x74, 0x6c, 0x65, 0x5f, 0x65, 0x6e, 0x64, 0x69, 0x61, 0x6e];
const FORMAT_BBE_KWD: [u8; 17] = [0x62, 0x69, 0x6e, 0x61, 0x72, 0x79, 0x5f, 0x62, 0x69, 0x67, 0x5f, 0x65, 0x6e, 0x64, 0x69, 0x61, 0x6e];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FormatType {
    Ascii,
    BinaryLittleEndian,
    BinaryBigEndian,
}

impl Default for FormatType {
    fn default() -> Self {
        FormatType::Ascii
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElementType {
    Vertex,
    Face,
    Edge,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CountType {
    Char,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    Char,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Float,
    Double,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyType {
    X,
    Y,
    Z,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct ListPropertyDescriptor {
    pub name: PropertyType,
    pub count_type: CountType,
    pub data_type: DataType,
}

#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub name: PropertyType,
    pub data_type: DataType,
}

#[derive(Debug, Clone)]
pub struct ElementDescriptor {
    pub name: String,
    pub count: usize,
    pub properties: Vec<PropertyDescriptor>,
    pub list_properties: Vec<ListPropertyDescriptor>,
}

#[derive(Debug, Clone)]
pub struct PlyDescriptor {
    pub format_type: FormatType,
    pub format_version: String,
    pub elements: Vec<ElementDescriptor>,
}

impl Default for PlyDescriptor {
    fn default() -> Self {
        PlyDescriptor {
            format_type: FormatType::Ascii,
            format_version: String::from("1.0"),
            elements: Vec::default(),
        }
    }
}

#[derive(Debug)]
pub struct Ply {
    pub descriptor: PlyDescriptor,
    pub comments: Vec<String>,
    pub obj_info: Vec<String>,
    pub property_data: Vec<u8>,
    pub list_property_data: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("The specified file contains an incomplete format statement at offset {}", .0)]
    IncompleteFormatStatement(usize),
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("The specified file does not have a complete header")]
    IncompleteHeader,
    #[error("The specified file ended unexpectedly at offset {}", .0)]
    UnexpectedEndOfFile(usize),
    #[error("The specified file is not a PLY file")]
    NotAPlyFile,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Reading,
    EndOfHeader,
    Format,
    Comment,
    ObjInfo,
    Element,
    Property,
    ListProperty,
}

pub fn load_ply<P: AsRef<Path>>(p: P) -> Result<Ply, Error> {
    use std::io::Read;

    let mut file = File::open(p)?;

    // Try to find the header start indicator
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    if buf != PLY_HEADER_START {
        return Err(Error::NotAPlyFile);
    }

    let mut format_type: Option<FormatType> = None;
    let mut format_version: Option<String> = None;
    let mut comments: Vec<String> = Vec::new();
    let mut obj_info: Vec<String> = Vec::new();

    let mut offset: usize = 4;
    let mut state = State::Reading;
    let mut statement_buf: Vec<u8> = Vec::with_capacity(256);
    loop {
        let mut byte_buf = [0u8; 1];
        let n = file.read(&mut byte_buf)?;
        if n == 0 {
            return Err(Error::UnexpectedEndOfFile(offset));
        }
        offset += n;

        match byte_buf[0] {
            0x0a => {
                if statement_buf.starts_with(&FORMAT_KWD) {
                    state = State::Format;
                } else if statement_buf.starts_with(&COMMENT_KWD) {
                    state = State::Comment;
                } else if statement_buf.starts_with(&OBJ_INFO_KWD) {
                    state = State::ObjInfo;
                } else if statement_buf.starts_with(&ELEMENT_KWD) {
                    state = State::Element;
                } else if statement_buf.starts_with(&PROPERTY_KWD) {
                    state = State::Property;
                } else if statement_buf.starts_with(&LIST_PROPERTY_KWD) {
                    state = State::ListProperty;
                } else if statement_buf.starts_with(&PLY_HEADER_END) {
                    state = State::EndOfHeader;
                }
            },
            b => statement_buf.push(b),
        }

        match state {
            State::Comment => {
                let s = std::str::from_utf8(&statement_buf[COMMENT_KWD.len()..])?;
                comments.push(s.to_owned());
            },
            State::ObjInfo => {
                let s = std::str::from_utf8(&statement_buf[OBJ_INFO_KWD.len()..])?;
                obj_info.push(s.to_owned());
            },
            State::Format => {
                let f: Vec<_> = statement_buf.split(|b| b == &0x20).skip(1).take(2).collect();
                if f.len() != 2 {
                    return Err(Error::IncompleteFormatStatement(offset));
                }

                format_version = Some(std::str::from_utf8(f[1])?.to_owned());
                format_type = if f[0] == FORMAT_ASCII_KWD {
                    Some(FormatType::Ascii)
                } else if f[0] == FORMAT_BLE_KWD {
                    Some(FormatType::BinaryLittleEndian)
                } else if f[0] == FORMAT_BBE_KWD {
                    Some(FormatType::BinaryBigEndian)
                } else {
                    None
                };

            },
            State::Element => {
                let e: Vec<_> = statement_buf.split(|b| b == &0x20).skip(1).take(2).collect();
                if e.len() != 2 {
                    return Err(Error::IncompleteFormatStatement(offset));
                }
            },
            State::Property => todo!(),
            State::ListProperty => todo!(),
            State::EndOfHeader => break,
            State::Reading => continue,
        }

        statement_buf.clear();
    }

    Ok(Ply {
        descriptor: PlyDescriptor::default(),
        comments: Vec::default(),
        obj_info: Vec::default(),
        property_data: Vec::default(),
        list_property_data: Vec::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_type_provides_these_variants() {
        let _ = FormatType::Ascii;
        let _ = FormatType::BinaryLittleEndian;
        let _ = FormatType::BinaryBigEndian;
    }

    #[test]
    fn element_type_provides_these_variants() {
        let _ = ElementType::Vertex;
        let _ = ElementType::Face;
        let _ = ElementType::Edge;
        let _ = ElementType::Custom(String::from("identifier"));
    }

    #[test]
    fn property_type_provides_these_variants() {
        let _ = PropertyType::X;
        let _ = PropertyType::Y;
        let _ = PropertyType::Z;
        let _ = PropertyType::Custom(String::from("identifier"));
    }

    #[test]
    fn data_type_provides_these_variants() {
        let _ = DataType::Char;
        let _ = DataType::UChar;
        let _ = DataType::Short;
        let _ = DataType::UShort;
        let _ = DataType::Int;
        let _ = DataType::UInt;
        let _ = DataType::Float;
        let _ = DataType::Double;
        
    }

    #[test]
    fn count_type_provides_these_variants() {
        let _ = CountType::Char;
        let _ = CountType::UChar;
        let _ = CountType::Short;
        let _ = CountType::UShort;
        let _ = CountType::Int;
        let _ = CountType::UInt;
        
    }
    #[test]
    fn ply_data_container_has_the_following_structure() {
        let _ = Ply {
            descriptor: PlyDescriptor::default(),
            comments: Vec::<String>::new(),
            obj_info: Vec::<String>::new(),
            property_data: Vec::<u8>::new(),
            list_property_data: Vec::<u8>::new(),
        };
    }

    #[test]
    fn ply_descriptor_has_the_following_structure() {
        let _ = PlyDescriptor {
            format_type: FormatType::Ascii,
            format_version: String::from("1.0"),
            elements: Vec::<ElementDescriptor>::new(),
        };
    }

    #[test]
    fn element_descriptor_has_the_following_structure() {
        let _ = ElementDescriptor {
            name: String::from("vertex"),
            count: 0usize,
            properties: Vec::<PropertyDescriptor>::new(),
            list_properties: Vec::<ListPropertyDescriptor>::new(),
        };
    }

    #[test]
    fn property_descriptor_has_the_following_structure() {
        let _ = PropertyDescriptor {
            name: PropertyType::X,
            data_type: DataType::Float,
        };
    }

    #[test]
    fn list_property_descriptor_has_the_following_structure() {
        let _ = ListPropertyDescriptor {
            name: PropertyType::X,
            count_type: CountType::UShort,
            data_type: DataType::Float,
        };
    }

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
            Err(e) => panic!("{}", e),
            Ok(p) => {
                assert_eq!(p.comments.len(), 0, "Number of comments is not zero, but {}", p.comments.len());
                assert_eq!(p.property_data.len(), 0, "Property data length is not zero, but {}", p.property_data.len());
                assert_eq!(p.list_property_data.len(), 0, "List property data length is not zero, but {}", p.list_property_data.len());

                let pd = &p.descriptor;
                assert_eq!(pd.format_type, FormatType::Ascii, "File format is not ASCII, but {:?}", &pd.format_type);
                assert_eq!(pd.format_version, "1.0", "File version is not 1.0, but {}", &pd.format_version);
                assert_eq!(pd.elements.len(), 1, "Number of element descriptors is not one, but {}", pd.elements.len());

                let ed = &pd.elements[0];
                assert_eq!(ed.name, "vertex", "Name of element descriptor is not vertex, but {}", &ed.name);
                assert_eq!(ed.count, 1, "Number of element instances is not one, but {}", ed.count);
                assert_eq!(ed.properties.len(), 1, "Number of element properties is not one, but {}", ed.properties.len());
                assert_eq!(ed.list_properties.len(), 0, "Number of element list properties is not zero, but {}", ed.list_properties.len());
            }
        }
    }

}
