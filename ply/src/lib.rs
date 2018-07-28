#[cfg(test)]
#[macro_use]
extern crate assertions;
#[macro_use]
extern crate nom;

use std::num;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatType {
    Ascii,
    BinaryBigEndian,
    BinaryLittleEndian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float32,
    Float64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatSpec {
    pub format: FormatType,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScalarProperty {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorProperty {
    pub name: String,
    pub count_data_type: DataType,
    pub data_type: DataType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub count: usize,
    pub scalars: Vec<ScalarProperty>,
    pub vectors: Vec<VectorProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    format: FormatSpec,
    elements: Vec<Element>,
}

fn is_ident_char(c: u8) -> bool {
    match c {
        b'a' ..= b'z' | b'A' ..= b'Z' | b'0' ..= b'9' | b'-' | b'_' | b'.' => true,
        _ => false,
    }
}

fn is_num_char(c: u8) -> bool {
    match c {
        b'0' ..= b'9' => true,
        _ => false,
    }
}

fn is_eol_char(c: u8) -> bool {
    c == b'\r' || c == b'\n'
}

fn to_string(input: &[u8]) -> String {
    String::from_utf8_lossy(input).to_string()
}

fn try_usize_from_bytes(input: &[u8]) -> Result<usize, num::ParseIntError> {
    let buf = String::from_utf8_lossy(input);
    usize::from_str_radix(&buf, 10)
}

named!(format_type<&[u8], FormatType>,
    alt!(
        value!(FormatType::Ascii, tag!(b"ascii")) |
        value!(FormatType::BinaryBigEndian, tag!(b"binary_big_endian")) |
        value!(FormatType::BinaryLittleEndian, tag!(b"binary_little_endian"))
    )
);

named!(data_type<&[u8], DataType>,
    alt!(
        value!(DataType::Int8, tag!(b"int8")) |
        value!(DataType::Int8, tag!(b"char")) |
        value!(DataType::Uint8, tag!(b"uint8")) |
        value!(DataType::Uint8, tag!(b"uchar")) |
        value!(DataType::Int16, tag!(b"int16")) |
        value!(DataType::Int16, tag!(b"short")) |
        value!(DataType::Uint16, tag!(b"uint16")) |
        value!(DataType::Uint16, tag!(b"ushort")) |
        value!(DataType::Int32, tag!(b"int32")) |
        value!(DataType::Int32, tag!(b"int")) |
        value!(DataType::Uint32, tag!(b"uint32")) |
        value!(DataType::Uint32, tag!(b"uint")) |
        value!(DataType::Float32, tag!(b"float32")) |
        value!(DataType::Float32, tag!(b"float")) |
        value!(DataType::Float64, tag!(b"float64")) |
        value!(DataType::Float64, tag!(b"double"))
    )
);

named!(ident<&[u8], String>,
    map!(take_while!(is_ident_char), to_string)
);

named!(format_statement<&[u8], FormatSpec>,
    do_parse!(
        tag!(b"format") >>
        take_while!(nom::is_space) >>
        format: format_type >>
        take_while!(nom::is_space) >>
        version: map!(take!(3), to_string) >>
        take_while!(is_eol_char) >>
        (FormatSpec { format, version })
    )
);

named!(comment<&[u8], String>,
    do_parse!(
        tag!(b"comment") >>
        take_while!(nom::is_space) >>
        comment: map!(take_till!(is_eol_char), to_string) >>
        take_while!(is_eol_char) >>
        (comment)
    )
);

named!(scalar_property<&[u8], ScalarProperty>,
    do_parse!(
        tag!(b"property") >>
        take_while!(nom::is_space) >>
        data_type: data_type >>
        take_while!(nom::is_space) >>
        name: ident >>
        take_while!(is_eol_char) >>
        (ScalarProperty { name, data_type })
    )
);

named!(vector_property<&[u8], VectorProperty>,
    do_parse!(
        tag!(b"property") >>
        take_while!(nom::is_space) >>
        tag!(b"list") >>
        take_while!(nom::is_space) >>
        count_data_type: data_type >>
        take_while!(nom::is_space) >>
        data_type: data_type >>
        take_while!(nom::is_space) >>
        name: ident >>
        take_while!(is_eol_char) >>
        (VectorProperty { name, count_data_type, data_type })
    )
);

named!(element_group<&[u8], Element>,
    dbg_dmp!(do_parse!(
        tag!(b"element") >>
        take_while!(nom::is_space) >>
        name: ident >>
        take_while!(nom::is_space) >>
        count: map_res!(take_while!(is_num_char), try_usize_from_bytes) >>
        take_while!(is_eol_char) >>
        vectors: many_till!(vector_property, end_header) >>
        scalars: many_till!(scalar_property, end_header) >>
        (Element { name, count, scalars: scalars.0, vectors: vectors.0 })
    ))
);

named!(begin_header<&[u8], ()>,
    dbg_dmp!(do_parse!(
        tag!(b"ply") >>
        take_while!(is_eol_char) >>
        (())
    ))
);

named!(end_header<&[u8], ()>,
    dbg_dmp!(do_parse!(
        tag!(b"end_header") >>
        take_while!(is_eol_char) >>
        (())
    ))
);

pub fn header(input: &[u8]) -> nom::IResult<&[u8], Header> {
    do_parse!(input,
        begin_header >>
        format: format_statement >>
        elements: many0!(element_group) >>
        end_header >>
        (Header { format, elements })
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ply_header_minimal() {
        let expected = Header {
            format: FormatSpec {
                format: FormatType::Ascii,
                version: "1.0".into(),
            },
            elements: Vec::new(),
        };

        let r = header(b"ply\nformat ascii 1.0\nend_header\n");
        assert_ok!(r);

        let r = r.unwrap();
        assert!(r.0.is_empty());
        assert_eq!(r.1, expected);
    }
}
