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
    fs::File,
    path::Path,
};

use generic_parsers::parse_whitespace;
use ply_parsers::parse_begin_header;

use self::{
    error::Error,
    generic_parsers::parse_from_lut,
    ply_parsers::{
        parse_comment, parse_data_point, parse_element, parse_format, parse_length, parse_list_property,
        parse_obj_info, parse_property,
    },
    tables::KWD_TBL,
    types::{ElementDescriptor, FormatType, ListPropertyDescriptor, Ply, PlyDescriptor, PropertyDescriptor},
};

mod generic_parsers;
mod ply_parsers;
mod tables;
pub mod types;
mod utilities;
pub mod parser;
pub mod error;

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

pub fn load_ply<P: AsRef<Path>>(p: P) -> Result<Ply, Error> {
    let mut file = File::open(p)?;
    let mut format_type: Option<FormatType> = None;
    let mut format_version: Option<String> = None;
    let mut elements: Vec<ElementDescriptor> = Vec::new();
    let mut comments: Vec<String> = Vec::new();
    let mut obj_info: Vec<String> = Vec::new();

    // Try to find the header start indicator
    parse_begin_header(&mut file)?;

    loop {
        let keyword = parse_from_lut(&mut file, KWD_TBL, |k| match k {
            0 => Ok(Keyword::Format),
            1 => Ok(Keyword::Element),
            2 => Ok(Keyword::ListProperty),
            3 => Ok(Keyword::Property),
            4 => Ok(Keyword::Comment),
            5 => Ok(Keyword::ObjInfo),
            6 => Ok(Keyword::EndHeader),
            _ => unreachable!(),
        })?;

        parse_whitespace(&mut file)?;

        match keyword {
            Keyword::Format => {
                let (ft, fv) = parse_format(&mut file)?;
                format_type = Some(ft);
                format_version = Some(fv);
            }
            Keyword::Element => {
                let (en, ec) = parse_element(&mut file)?;
                elements.push(ElementDescriptor {
                    name: en,
                    count: ec,
                    properties: Vec::new(),
                    list_properties: Vec::new(),
                });
            }
            Keyword::Property => {
                let (dt, pn) = parse_property(&mut file)?;
                elements
                    .last_mut()
                    .map(|e| {
                        e.properties.push(PropertyDescriptor {
                            name: pn,
                            data_type: dt,
                        })
                    })
                    .ok_or(Error::UnexpectedProperty)?;
            }
            Keyword::ListProperty => {
                let (ct, dt, pn) = parse_list_property(&mut file)?;
                elements
                    .last_mut()
                    .map(|e| {
                        e.list_properties.push(ListPropertyDescriptor {
                            name: pn,
                            count_type: ct,
                            data_type: dt,
                        })
                    })
                    .ok_or(Error::UnexpectedListProperty)?;
            }
            Keyword::Comment => {
                let cmt = parse_comment(&mut file)?;
                comments.push(cmt);
            }
            Keyword::ObjInfo => {
                let obj = parse_obj_info(&mut file)?;
                obj_info.push(obj);
            }
            Keyword::EndHeader => break,
        }
    }

    let descriptor = PlyDescriptor {
        format_type: format_type.ok_or(Error::MissingFormatType)?,
        format_version: format_version.ok_or(Error::MissingFormatVersion)?,
        elements,
        comments,
        obj_info,
    };

    let mut property_data = Vec::new();
    let mut list_property_data = Vec::new();
    for e in &descriptor.elements {
        for _ in 0..e.count {
            for p in &e.properties {
                let d = parse_data_point(&mut file, p.data_type, descriptor.format_type)?;
                property_data.extend(d);
            }

            for l in &e.list_properties {
                let c = parse_length(&mut file, l.count_type, descriptor.format_type)?;
                for _ in 0..c {
                    let d = parse_data_point(&mut file, l.data_type, descriptor.format_type)?;
                    list_property_data.extend(d);
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
            Err(e) => panic!("{}", e),
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
