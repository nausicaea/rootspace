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

use std::num::ParseIntError;
use std::task::Poll;
use std::{path::Path, fs::File};
use std::io::Read;

use thiserror::Error;

const PLY_HEADER_START: [u8; 4] = [0x70, 0x6c, 0x79, 0x0a];
const KWD_TBL: &[&[u8]] = &[
    // Equivalent to `Keyword::Format`
    &[0x66, 0x6f, 0x72, 0x6d, 0x61, 0x74, 0x20],
    // Equivalent to `Keyword::Element`
    &[0x65, 0x6c, 0x65, 0x6d, 0x65, 0x6e, 0x74, 0x20],
    // Equivalent to `Keyword::ListProperty`
    &[0x70, 0x72, 0x6f, 0x70, 0x65, 0x72, 0x74, 0x79, 0x20, 0x6c, 0x69, 0x73, 0x74, 0x20],
    // Equivalent to `Keyword::Property`
    &[0x70, 0x72, 0x6f, 0x70, 0x65, 0x72, 0x74, 0x79, 0x20],
    // Equivalent to `Keyword::Comment`
    &[0x63, 0x6f, 0x6d, 0x6d, 0x65, 0x6e, 0x74, 0x20],
    // Equivalent to `Keyword::ObjInfo`
    &[0x6f, 0x62, 0x6a, 0x5f, 0x69, 0x6e, 0x66, 0x6f, 0x20],
    // Equivalent to `Keyword::EndHeader`
    &[0x65, 0x6e, 0x64, 0x5f, 0x68, 0x65, 0x61, 0x64, 0x65, 0x72, 0x0a],
];
const FMT_TYP_TBL: &[&[u8]] = &[
    // Equivalent to `FormatType::Ascii`
    &[0x61, 0x73, 0x63, 0x69, 0x69, 0x20],
    // Equivalent to `FormatType::BinaryLittleEndian`
    &[0x62, 0x69, 0x6e, 0x61, 0x72, 0x79, 0x5f, 0x6c, 0x69, 0x74, 0x74, 0x6c, 0x65, 0x5f, 0x65, 0x6e, 0x64, 0x69, 0x61, 0x6e, 0x20],
    // Equivalent to `FormatType::BinaryBigEndian`
    &[0x62, 0x69, 0x6e, 0x61, 0x72, 0x79, 0x5f, 0x62, 0x69, 0x67, 0x5f, 0x65, 0x6e, 0x64, 0x69, 0x61, 0x6e, 0x20],
];
const DAT_TYP_TBL: &[&[u8]] = &[
    // Equivalent to `DataType::I8`
    &[0x63, 0x68, 0x61, 0x72, 0x20],
    // Equivalent to `DataType::U8`
    &[0x75, 0x63, 0x68, 0x61, 0x72, 0x20],
    // Equivalent to `DataType::I16`
    &[0x73, 0x68, 0x6f, 0x72, 0x74, 0x20],
    // Equivalent to `DataType::U16`
    &[0x75, 0x73, 0x68, 0x6f, 0x72, 0x74, 0x20],
    // Equivalent to `DataType::I32`
    &[0x69, 0x6e, 0x74, 0x20],
    // Equivalent to `DataType::U32`
    &[0x75, 0x69, 0x6e, 0x74, 0x20],
    // Equivalent to `DataType::F32`
    &[0x66, 0x6c, 0x6f, 0x61, 0x74, 0x20],
    // Equivalent to `DataType::F64`
    &[0x64, 0x6f, 0x75, 0x62, 0x6c, 0x65, 0x20],
    // Equivalent to `DataType::I8`
    &[0x69, 0x6e, 0x74, 0x38, 0x20],
    // Equivalent to `DataType::U8`
    &[0x75, 0x69, 0x6e, 0x74, 0x38, 0x20],
    // Equivalent to `DataType::I16`
    &[0x69, 0x6e, 0x74, 0x31, 0x36, 0x20],
    // Equivalent to `DataType::U16`
    &[0x75, 0x69, 0x6e, 0x74, 0x31, 0x36, 0x20],
    // Equivalent to `DataType::I32`
    &[0x69, 0x6e, 0x74, 0x33, 0x32, 0x20],
    // Equivalent to `DataType::U32`
    &[0x75, 0x69, 0x6e, 0x74, 0x33, 0x32, 0x20],
    // Equivalent to `DataType::F32`
    &[0x66, 0x6c, 0x6f, 0x61, 0x74, 0x33, 0x32, 0x20],
    // Equivalent to `DataType::F64`
    &[0x66, 0x6c, 0x6f, 0x61, 0x74, 0x36, 0x34, 0x20],
];
const CNT_TYP_TBL: &[&[u8]] = &[
    // Equivalent to `DataType::U8`
    &[0x75, 0x63, 0x68, 0x61, 0x72, 0x20],
    // Equivalent to `DataType::U16`
    &[0x75, 0x73, 0x68, 0x6f, 0x72, 0x74, 0x20],
    // Equivalent to `DataType::U32`
    &[0x75, 0x69, 0x6e, 0x74, 0x20],
    // Equivalent to `DataType::U8`
    &[0x75, 0x69, 0x6e, 0x74, 0x38, 0x20],
    // Equivalent to `DataType::U16`
    &[0x75, 0x69, 0x6e, 0x74, 0x31, 0x36, 0x20],
    // Equivalent to `DataType::U32`
    &[0x75, 0x69, 0x6e, 0x74, 0x33, 0x32, 0x20],
];

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CountType {
    U8,
    U16,
    U32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    F32,
    F64,
}

#[derive(Debug, Clone)]
pub struct ListPropertyDescriptor {
    pub name: String,
    pub count_type: CountType,
    pub data_type: DataType,
}

#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub name: String,
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
    pub comments: Vec<String>,
    pub obj_info: Vec<String>,
}

impl Default for PlyDescriptor {
    fn default() -> Self {
        PlyDescriptor {
            format_type: FormatType::Ascii,
            format_version: String::from("1.0"),
            elements: Vec::default(),
            comments: Vec::default(),
            obj_info: Vec::default(),
        }
    }
}

#[derive(Debug)]
pub struct Ply {
    pub descriptor: PlyDescriptor,
    pub property_data: Vec<u8>,
    pub list_property_data: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Found a list property with no parent element at offset {:#x}", .0)]
    UnexpectedListProperty(usize),
    #[error("Found a property with no parent element at offset {:#x}", .0)]
    UnexpectedProperty(usize),
    #[error("Missing format type")]
    MissingFormatType,
    #[error("Missing format version")]
    MissingFormatVersion,
    #[error("Unexpected byte {:#x} at offset {:#x}", .0, .1)]
    UnexpectedByte(u8, usize),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("The specified file ended unexpectedly at offset {:#x}", .0)]
    UnexpectedEndOfFile(usize),
    #[error("The specified file is not a PLY file")]
    NotAPlyFile,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Keyword {
    Format,
    Element,
    Property,
    ListProperty,
    Comment,
    ObjInfo,
    EndHeader,
}

fn parse_format<R>(file: &mut R, offset: &mut usize) -> Result<(FormatType, String), Error> 
where
    R: Read,
{
    let format_type = parse_from_lut(file, offset, FMT_TYP_TBL, |k| {
        match k {
            0 => {
                return Ok(FormatType::Ascii);
            },
            1 => {
                return Ok(FormatType::BinaryLittleEndian);
            },
            2 => {
                return Ok(FormatType::BinaryBigEndian);
            },
            _ => unreachable!(),
        }
    })?;

    let mut format_version = String::new();
    parse_individual(file, offset, |b, o| {
        match b {
            0x0a => Poll::Ready(Ok(())),
            b@0x30..=0x39 | b@0x2e => {
                format_version.push(char::from(b));
                Poll::Pending
            },
            b => Poll::Ready(Err(Error::UnexpectedByte(b, o))),
        }
    })?;

    Ok((format_type, format_version))
}

fn parse_element<R>(file: &mut R, offset: &mut usize) -> Result<(String, usize), Error> 
where
    R: Read,
{
    let mut name = String::new();
    parse_individual(file, offset, |b, _| {
        match b {
            0x20 => Poll::Ready(Ok(())),
            b => {
                name.push(char::from(b));
                Poll::Pending
            }
        }
    })?;

    let mut count = String::new();
    parse_individual(file, offset, |b, o| {
        match b {
            0x0a => Poll::Ready(Ok(())),
            b@0x30..=0x39 => {
                count.push(char::from(b));
                Poll::Pending
            },
            b => Poll::Ready(Err(Error::UnexpectedByte(b, o))),
        }
    })?;
    let count: usize = count.parse()?;

    Ok((name, count))
}

fn parse_property<R>(file: &mut R, offset: &mut usize) -> Result<(DataType, String), Error> 
where
    R: Read,
{
    let data_type = parse_from_lut(file, offset, DAT_TYP_TBL, |k| {
        match k {
            0 => Ok(DataType::I8),
            1 => Ok(DataType::U8),
            2 => Ok(DataType::I16),
            3 => Ok(DataType::U16),
            4 => Ok(DataType::I32),
            5 => Ok(DataType::U32),
            6 => Ok(DataType::F32),
            7 => Ok(DataType::F64),
            8 => Ok(DataType::I8),
            9 => Ok(DataType::U8),
            10 => Ok(DataType::I16),
            11 => Ok(DataType::U16),
            12 => Ok(DataType::I32),
            13 => Ok(DataType::U32),
            14 => Ok(DataType::F32),
            15 => Ok(DataType::F64),
            _ => unreachable!(),
        }
    })?;

    let mut name = String::new();
    parse_individual(file, offset, |b, _| {
        match b {
            0x0a => Poll::Ready(Ok(())),
            b => {
                name.push(char::from(b));
                Poll::Pending
            }
        }
    })?;

    Ok((data_type, name))
}

fn parse_list_property<R>(file: &mut R, offset: &mut usize) -> Result<(CountType, DataType, String), Error> 
where
    R: Read,
{
    let count_type = parse_from_lut(file, offset, CNT_TYP_TBL, |k| {
        match k {
            0 => Ok(CountType::U8),
            1 => Ok(CountType::U16),
            2 => Ok(CountType::U32),
            3 => Ok(CountType::U8),
            4 => Ok(CountType::U16),
            5 => Ok(CountType::U32),
            _ => unreachable!(),
        }
    })?;

    let data_type = parse_from_lut(file, offset, DAT_TYP_TBL, |k| {
        match k {
            0 => Ok(DataType::I8),
            1 => Ok(DataType::U8),
            2 => Ok(DataType::I16),
            3 => Ok(DataType::U16),
            4 => Ok(DataType::I32),
            5 => Ok(DataType::U32),
            6 => Ok(DataType::F32),
            7 => Ok(DataType::F64),
            8 => Ok(DataType::I8),
            9 => Ok(DataType::U8),
            10 => Ok(DataType::I16),
            11 => Ok(DataType::U16),
            12 => Ok(DataType::I32),
            13 => Ok(DataType::U32),
            14 => Ok(DataType::F32),
            15 => Ok(DataType::F64),
            _ => unreachable!(),
        }
    })?;

    let mut name = String::new();
    parse_individual(file, offset, |b, _| {
        match b {
            0x0a => Poll::Ready(Ok(())),
            b => {
                name.push(char::from(b));
                Poll::Pending
            }
        }
    })?;

    Ok((count_type, data_type, name))
}

fn parse_comment<R>(file: &mut R, offset: &mut usize) -> Result<String, Error> 
where
    R: Read,
{
    let mut comment = String::new();
    parse_individual(file, offset, |b, _| {
        match b {
            0x0a => Poll::Ready(Ok(())),
            b => {
                comment.push(char::from(b));
                Poll::Pending
            }
        }
    })?;

    Ok(comment)
}

fn parse_obj_info<R>(file: &mut R, offset: &mut usize) -> Result<String, Error> 
where
    R: Read,
{
    let mut obj_info = String::new();
    parse_individual(file, offset, |b, _| {
        match b {
            0x0a => Poll::Ready(Ok(())),
            b => {
                obj_info.push(char::from(b));
                Poll::Pending
            }
        }
    })?;

    Ok(obj_info)
}

fn read_byte<R>(file: &mut R, offset: &mut usize) -> Result<u8, Error>
where
    R: Read,
{
    let mut byte_buf = [0u8; 1];
    let n = file.read(&mut byte_buf)?;
    if n == 0 {
        return Err(Error::UnexpectedEndOfFile(*offset));
    }
    *offset = *offset + n;

    Ok(byte_buf[0])
}

fn parse_from_lut<T, F, R>(file: &mut R, offset: &mut usize, lut: &[&[u8]], mut func: F) -> Result<T, Error> 
where
    R: Read,
    F: FnMut(usize) -> Result<T, Error>,
{
    let mut indices: Vec<usize> = vec![0usize; lut.len()];
    loop {
        let byte = read_byte(file, offset)?;
        
        for k in 0..lut.len() {
            let index = indices[k];
            let expected_bytes = lut[k];
            if byte == expected_bytes[index] {
                indices[k] += 1;
            }

            if indices[k] >= lut[k].len() {
                eprintln!("Found: {:?}", std::str::from_utf8(lut[k]).unwrap());
                return func(k);
            }
        }
    }
}

fn parse_individual<T, F, R>(file: &mut R, offset: &mut usize, mut func: F) -> Result<T, Error> 
where
    R: Read,
    F: FnMut(u8, usize) -> Poll<Result<T, Error>>,
{
    loop {
        let byte = read_byte(file, offset)?;

        match func(byte, *offset) {
            Poll::Pending => (),
            Poll::Ready(r) => return r,
        }
    }
}

pub fn load_ply<P: AsRef<Path>>(p: P) -> Result<Ply, Error> {
    let mut file = File::open(p)?;

    // Try to find the header start indicator
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    if buf != PLY_HEADER_START {
        return Err(Error::NotAPlyFile);
    }

    let mut format_type: Option<FormatType> = None;
    let mut format_version: Option<String> = None;
    let mut elements: Vec<ElementDescriptor> = Vec::new();
    let mut comments: Vec<String> = Vec::new();
    let mut obj_info: Vec<String> = Vec::new();

    let mut offset = PLY_HEADER_START.len() - 1;
    loop {
        let keyword = parse_from_lut(&mut file, &mut offset, KWD_TBL, |k| {
            match k {
                0 => Ok(Keyword::Format),
                1 => Ok(Keyword::Element),
                2 => Ok(Keyword::ListProperty),
                3 => Ok(Keyword::Property),
                4 => Ok(Keyword::Comment),
                5 => Ok(Keyword::ObjInfo),
                6 => Ok(Keyword::EndHeader),
                _ => unreachable!(),
            }
        })?;

        match keyword {
            Keyword::Format => {
                let (ft, fv) = parse_format(&mut file, &mut offset)?;
                format_type = Some(ft);
                format_version = Some(fv);
            },
            Keyword::Element => {
                let (en, ec) = parse_element(&mut file, &mut offset)?;
                elements.push(ElementDescriptor {
                    name: en,
                    count: ec,
                    properties: Vec::new(),
                    list_properties: Vec::new(),
                });
            },
            Keyword::Property => {
                let (dt, pn) = parse_property(&mut file, &mut offset)?;
                elements.last_mut()
                    .map(|e| e.properties.push(PropertyDescriptor { 
                        name: pn,
                        data_type: dt,
                    }))
                    .ok_or(Error::UnexpectedProperty(offset))?;
            },
            Keyword::ListProperty => {
                let (ct, dt, pn) = parse_list_property(&mut file, &mut offset)?;
                elements.last_mut()
                    .map(|e| e.list_properties.push(ListPropertyDescriptor { 
                        name: pn,
                        count_type: ct,
                        data_type: dt,
                    }))
                    .ok_or(Error::UnexpectedListProperty(offset))?;
            },
            Keyword::Comment => {
                let cmt = parse_comment(&mut file, &mut offset)?;
                comments.push(cmt);

            },
            Keyword::ObjInfo => {
                let obj = parse_obj_info(&mut file, &mut offset)?;
                obj_info.push(obj);
            },
            Keyword::EndHeader => break,
        }
    }

    Ok(Ply {
        descriptor: PlyDescriptor {
            format_type: format_type.ok_or(Error::MissingFormatType)?,
            format_version: format_version.ok_or(Error::MissingFormatVersion)?,
            elements,
            comments,
            obj_info,
        },
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
    fn data_type_provides_these_variants() {
        let _ = DataType::I8;
        let _ = DataType::U8;
        let _ = DataType::I16;
        let _ = DataType::U16;
        let _ = DataType::I32;
        let _ = DataType::U32;
        let _ = DataType::F32;
        let _ = DataType::F64;
        
    }

    #[test]
    fn count_type_provides_these_variants() {
        let _ = CountType::U8;
        let _ = CountType::U16;
        let _ = CountType::U32;
        
    }
    #[test]
    fn ply_data_container_has_the_following_structure() {
        let _ = Ply {
            descriptor: PlyDescriptor::default(),
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
            comments: Vec::<String>::new(),
            obj_info: Vec::<String>::new(),
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
            name: String::from("x"),
            data_type: DataType::F32,
        };
    }

    #[test]
    fn list_property_descriptor_has_the_following_structure() {
        let _ = ListPropertyDescriptor {
            name: String::from("i"),
            count_type: CountType::U16,
            data_type: DataType::F32,
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
                let pd = &p.descriptor;
                assert_eq!(pd.format_type, FormatType::Ascii, "File format is not ASCII, but {:?}", &pd.format_type);
                assert_eq!(pd.format_version, "1.0", "File version is not 1.0, but {}", &pd.format_version);
                assert_eq!(pd.elements.len(), 1, "Number of element descriptors is not one, but {}", pd.elements.len());
                assert_eq!(pd.comments.len(), 0, "Number of comments is not zero, but {}", pd.comments.len());
                assert_eq!(pd.obj_info.len(), 0, "Number of obj_info is not zero, but {}", pd.obj_info.len());

                let ed = &pd.elements[0];
                assert_eq!(ed.name, "vertex", "Name of element descriptor is not vertex, but {}", &ed.name);
                assert_eq!(ed.count, 1, "Number of element instances is not one, but {}", ed.count);
                assert_eq!(ed.properties.len(), 1, "Number of element properties is not one, but {}", ed.properties.len());
                assert_eq!(ed.list_properties.len(), 0, "Number of element list properties is not zero, but {}", ed.list_properties.len());

                assert_eq!(p.property_data.len(), 3, "Property data length is not three, but {}", p.property_data.len());
                assert_eq!(p.list_property_data.len(), 0, "List property data length is not zero, but {}", p.list_property_data.len());

                let pdat = std::str::from_utf8(&p.property_data).unwrap();
                assert_eq!(pdat, "1.0");

            }
        }
    }

}
