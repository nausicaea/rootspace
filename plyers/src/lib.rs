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
//!

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use either::{Either, Left, Right};
use crate::error::Error;
use crate::parser::empty::empty;
use crate::parser::engram::engram;
use crate::parser::lookahead::lookahead;
use crate::parser::Parser;
use crate::parser::take_while::take_while;
use crate::types::{CommentDescriptor, CountType, DataType, ElementDescriptor, ListPropertyDescriptor, ObjInfoDescriptor, PlyDescriptor, PropertyDescriptor};

use self::{
    types::{FormatType, Ply},
};

pub mod types;
pub mod parser;
pub mod error;

#[derive(Debug, Clone, Copy)]
enum DataValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
    F64(f64),
}

impl<'a> Into<&'a [u8]> for &'a DataValue {
    fn into(self) -> &'a [u8] {
        match self {
            DataValue::U8(dv) => bytemuck::bytes_of(dv),
            DataValue::I8(dv) => bytemuck::bytes_of(dv),
            DataValue::U16(dv) => bytemuck::bytes_of(dv),
            DataValue::I16(dv) => bytemuck::bytes_of(dv),
            DataValue::U32(dv) => bytemuck::bytes_of(dv),
            DataValue::I32(dv) => bytemuck::bytes_of(dv),
            DataValue::F32(dv) => bytemuck::bytes_of(dv),
            DataValue::F64(dv) => bytemuck::bytes_of(dv),
        }
    }
}

fn ascii_usize_parser() -> impl Parser<Item = usize> {
    take_while(|b| b != b' ')
        .and_then(|cd| {
            String::from_utf8(cd)
                .map_err(|e| e.into())
                .and_then(|cd| cd.parse::<usize>().map_err(|e| e.into()))
        })
}

fn ascii_number_parser(data_type: DataType) -> impl Parser<Item = DataValue> + Clone {
    take_while(|b| b != b' ' && b != b'\n')
        .and_then(move |pd| {
            let pd = String::from_utf8(pd)?;
            let pd = match data_type {
                DataType::U8 => pd.parse::<u8>().map(|pd| DataValue::U8(pd))?,
                DataType::I8 => pd.parse::<i8>().map(|pd| DataValue::I8(pd))?,
                DataType::U16 => pd.parse::<u16>().map(|pd| DataValue::U16(pd))?,
                DataType::I16 => pd.parse::<i16>().map(|pd| DataValue::I16(pd))?,
                DataType::U32 => pd.parse::<u32>().map(|pd| DataValue::U32(pd))?,
                DataType::I32 => pd.parse::<i32>().map(|pd| DataValue::I32(pd))?,
                DataType::F32 => pd.parse::<f32>().map(|pd| DataValue::F32(pd))?,
                DataType::F64 => pd.parse::<f64>().map(|pd| DataValue::F64(pd))?,
            };

            Ok(pd)
        })
}

pub fn parse_ply<S: Read + Seek>(stream: &mut S) -> Result<Ply, Error> {
    let comment = engram(b"comment ")
        .chain(take_while(|b| b != b'\n'))
        .and_then(|(_, c)| {
            let c = String::from_utf8(c)?;
            Ok(CommentDescriptor(c))
        });
    let obj_info = engram(b"obj_info ")
        .chain(take_while(|b| b != b'\n'))
        .and_then(|(_, o)| {
            let o = String::from_utf8(o)?;
            Ok(ObjInfoDescriptor(o))
        });
    let comment_or_obj_info = empty()
        .chain_either(comment, obj_info);
    let format = engram(b"format ")
        .chain(take_while(|b| b != b' '))
        .and_then(|(_, ft)| {
            let ft = FormatType::try_from_bytes(&ft)?;
            Ok(ft)
        })
        .chain(take_while(|b| b != b'\n'))
        .and_then(|(ft, fv)| {
            let fv = String::from_utf8(fv)?;
            Ok((ft, fv))
        });

    let normal_property = empty()
        .chain_optional(comment_or_obj_info.clone())
        .chain(engram(b"property "))
        .chain(take_while(|b| b != b' '))
        .and_then(|(_, dt)| {
            let dt = DataType::try_from_bytes(&dt)?;
            Ok(dt)
        })
        .chain(take_while(|b| b != b'\n'))
        .and_then(|(dt, n)| {
            let n = String::from_utf8(n)?;
            Ok(PropertyDescriptor {
                data_type: dt,
                name: n,
            })
        });
    let list_property = empty()
        .chain_optional(comment_or_obj_info.clone())
        .chain(engram(b"property list "))
        .chain(take_while(|b| b != b' '))
        .and_then(|(_, ct)| {
            let ct = CountType::try_from_bytes(&ct)?;
            Ok(ct)
        })
        .chain(take_while(|b| b != b' '))
        .and_then(|(ct, dt)| {
            let dt = DataType::try_from_bytes(&dt)?;
            Ok((ct, dt))
        })
        .chain(take_while(|b| b != b'\n'))
        .and_then(|((ct, dt), n)| {
            let n = String::from_utf8(n)?;
            Ok(ListPropertyDescriptor {
                count_type: ct,
                data_type: dt,
                name: n,
            })
        });
    let normal_properties = empty()
        .chain_repeat(normal_property, lookahead(b'e'))
        .map(|(_, np, _)| np);
    let list_properties = empty()
        .chain_repeat(list_property, lookahead(b'e'))
        .map(|(_, lp, _)| lp);
    let properties = empty()
        .chain_either(normal_properties, list_properties)
        .map(|(_, p)| p);

    let element = empty()
        .chain_optional(comment_or_obj_info.clone())
        .chain(engram(b"element "))
        .chain(take_while(|b| b != b' '))
        .and_then(|(_, en)| {
            let en = String::from_utf8(en)?;
            Ok(en)
        })
        .chain(take_while(|b| b != b'\n'))
        .and_then(|(en, ec)| {
            let ec = String::from_utf8(ec)?;
            let ec = ec.parse::<usize>()?;
            Ok((en, ec))
        })
        .chain(properties)
        .map(|((en, ec), p)| match p {
            Left(p) => ElementDescriptor {
                name: en,
                count: ec,
                properties: p,
                list_properties: vec![]
            },
            Right(lp) => ElementDescriptor {
                name: en,
                count: ec,
                properties: vec![],
                list_properties: lp,
            },
        });

    let header = engram(b"ply\n")
        .chain_optional(comment_or_obj_info.clone())
        .chain(format)
        .map(|(_, (ft, fv))| (ft, fv))
        .chain_repeat(element, engram(b"end_header\n"))
        .map(|((ft, fv), e, _)| PlyDescriptor {
            format_type: ft,
            format_version: fv,
            elements: e,
            comments: vec![],
            obj_info: vec![]
        });

    let r = header.parse(stream)?;

    todo!()
}

pub fn load_ply<P: AsRef<Path>>(p: P) -> Result<Ply, Error> {
    let file = File::open(p)?;
    let mut reader = BufReader::new(file);

    parse_ply(&mut reader)
}

#[cfg(test)]
pub(crate) fn to_reader(source: &str) -> std::io::Cursor<&[u8]> {
    std::io::Cursor::new(source.as_bytes())
}

pub(crate) fn read_byte<R>(file: &mut R) -> Result<(u8, u64), Error>
    where
        R: Read + Seek,
{
    let mut byte_buf = [0u8; 1];
    let position = file.stream_position()?;
    let n = file.read(&mut byte_buf)?;
    if n == 0 {
        return Err(Error::UnexpectedEndOfFile(position));
    }
    Ok((byte_buf[0], position))
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
