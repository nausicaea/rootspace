#[cfg(test)]
#[macro_use]
extern crate assertions;
extern crate combine;

use combine::parser::Parser;
use combine::parser::byte::{byte, bytes, spaces, digit, crlf, newline};
use combine::parser::choice::{choice, optional};
use combine::parser::combinator::{try, lazy};
use combine::parser::item::value;
use combine::parser::range::{range, take_while1};
use combine::parser::repeat::{many, many1, sep_by, take_until, count_min_max};
use combine::parser::sequence::between;
use combine::stream::{Stream, StreamOnce, RangeStream};
use combine::error::ParseError;

/// Describes the recognized formats of a PLY file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatType {
    Ascii,
    BinaryBigEndian,
    BinaryLittleEndian,
}

/// Describes the recognized data types for property values.
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

/// Describes the PLY format and version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Format {
    pub format: FormatType,
    pub version: Vec<usize>,
}

/// Describes a PLY property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub name: String,
    pub count_data_type: Option<DataType>,
    pub data_type: DataType,
}

/// Describes a PLY element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub count: usize,
    pub properties: Vec<Property>,
}

/// Describes the PLY header..
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub format: Format,
    pub elements: Vec<Element>,
}

pub enum PropertyData {
    Int8(i8),
    Uint8(u8),
    Int16(i16),
    Uint16(u16),
    Int32(i32),
    Uint32(u32),
    Float32(f32),
    Float64(f64),
    VInt8(Vec<i8>),
    VUint8(Vec<u8>),
    VInt16(Vec<i16>),
    VUint16(Vec<u16>),
    VInt32(Vec<i32>),
    VUint32(Vec<u32>),
    VFloat32(Vec<f32>),
    VFloat64(Vec<f64>),
}

pub struct ElementData {
    pub properties: Vec<PropertyData>,
}

pub struct Body {
    pub elements: Vec<ElementData>,
}

pub struct Ply {
    pub header: Header,
    pub body: Body,
}

/// Returns true if the supplied byteacter is ASCII alphabetic.
fn is_alphabetic(b: u8) -> bool {
    match b {
        b'a' ..= b'z' | b'A' ..= b'Z' => true,
        _ => false,
    }
}

/// Returns true if the supplied byteacter is ASCII numeric.
fn is_numeric(b: u8) -> bool {
    match b {
        b'0' ..= b'9' => true,
        _ => false,
    }
}

/// Returns true if the supplied byteacter is ASCII alphanumeric or a limited set of special
/// characters.
fn is_identity(b: u8) -> bool {
    is_alphabetic(b) || is_numeric(b) || b".-_".iter().any(|s| s == &b)
}

/// Skips any whitespace after the supplied parser.
fn lex<'a, P>(parser: P) -> impl Parser<Input = P::Input, Output = P::Output>
where
    P: Parser,
    P::Input: Stream<Item = u8, Range = &'a [u8]> + 'a,
    <P::Input as StreamOnce>::Error: ParseError<
        <P::Input as StreamOnce>::Item,
        <P::Input as StreamOnce>::Range,
        <P::Input as StreamOnce>::Position,
    >,
{
    parser.skip(spaces())
}

/// Parses an unsigned integer from a stream of numeric ASCII characters.
fn unsigned_integer<'a, I>() -> impl Parser<Input = I, Output = usize> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1::<Vec<_>, _>(digit())
        .map(|v| {
            let mut n: usize = 0;
            for byte in v {
                n = n * 10 + (byte as usize - '0' as usize);
            }
            n
        })
        .expected("an unsigned ASCII integer")
}

/// Parses a set of characters if it can be interpreted as a name or identity.
fn identity<'a, I>() -> impl Parser<Input = I, Output = String> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    take_while1(is_identity)
        .map(|s| String::from_utf8_lossy(s).to_string())
        .expected("a name or identity")
}

/// Parses end-of-line sequences and maps them to the LF ASCII character.
fn eol<'a, I>() -> impl Parser<Input = I, Output = u8> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    crlf()
        .or(newline())
        .expected("a line termination byte sequence")
}

/// Parses the beginning of the PLY header.
fn begin_header<'a, I>() -> impl Parser<Input = I, Output = &'a [u8]> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(range(&b"ply"[..]))
        .expected("the beginning of the PLY header")
}

/// Parses the end of the PLY header.
fn end_header<'a, I>() -> impl Parser<Input = I, Output = &'a [u8]> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(range(&b"end_header"[..]))
        .expected("the end of the PLY header")
}

/// Parses a data type.
fn data_type<'a, I>() -> impl Parser<Input = I, Output = DataType> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice([
        try(bytes(&b"int8"[..])),
        try(bytes(&b"char"[..])),
        try(bytes(&b"uint8"[..])),
        try(bytes(&b"uchar"[..])),
        try(bytes(&b"int16"[..])),
        try(bytes(&b"short"[..])),
        try(bytes(&b"uint16"[..])),
        try(bytes(&b"ushort"[..])),
        try(bytes(&b"int32"[..])),
        try(bytes(&b"int"[..])),
        try(bytes(&b"uint32"[..])),
        try(bytes(&b"uint"[..])),
        try(bytes(&b"float32"[..])),
        try(bytes(&b"float"[..])),
        try(bytes(&b"float64"[..])),
        try(bytes(&b"double"[..])),
    ])
        .map(|r: &[u8]| {
            match r {
                b"int8" => DataType::Int8,
                b"char" => DataType::Int8,
                b"uint8" => DataType::Uint8,
                b"uchar" => DataType::Uint8,
                b"int16" => DataType::Int16,
                b"short" => DataType::Int16,
                b"uint16" => DataType::Uint16,
                b"ushort" => DataType::Uint16,
                b"int32" => DataType::Int32,
                b"int" => DataType::Uint32,
                b"uint32" => DataType::Uint32,
                b"uint" => DataType::Uint32,
                b"float32" => DataType::Float32,
                b"float" => DataType::Float32,
                b"float64" => DataType::Float64,
                b"double" => DataType::Float64,
                _ => unreachable!(),
            }
        })
        .expected("a property data type")
}

/// Parses the PLY format type.
fn format_type<'a, I>() -> impl Parser<Input = I, Output = FormatType> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice([
       try(bytes(&b"ascii"[..])),
       try(bytes(&b"binary_big_endian"[..])),
       try(bytes(&b"binary_little_endian"[..])),
    ])
        .map(|r: &[u8]| {
            match r {
                b"ascii" => FormatType::Ascii,
                b"binary_big_endian" => FormatType::BinaryBigEndian,
                b"binary_little_endian" => FormatType::BinaryLittleEndian,
                _ => unreachable!(),
            }
        })
        .expected("a PLY format type")
}

/// Parses the PLY format version.
fn format_version<'a, I>() -> impl Parser<Input = I, Output = Vec<usize>> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    sep_by::<Vec<_>, _, _>(unsigned_integer(), byte(b'.'))
        .expected("a PLY format version")
}

/// Parses the PLY format statement.
fn format_stmt<'a, I>() -> impl Parser<Input = I, Output = Format> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        lex(bytes(&b"format"[..])),
        lex(format_type()),
        lex(format_version()),
    )
        .map(|(_, format, version)| Format { format, version })
        .expected("a format statement")
}

/// Parses comment statements.
fn comment_stmt<'a, I>() -> impl Parser<Input = I, Output = String> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        lex(bytes(&b"comment"[..])),
        lex(take_until::<Vec<_>, _>(eol())),
    )
        .map(|(_, c)| String::from_utf8_lossy(&c).to_string())
        .expected("a comment statement")
}

/// Parses property statements (scalars and vectors).
fn property_stmt<'a, I>() -> impl Parser<Input = I, Output = Property> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        lex(bytes(&b"property"[..])),
        optional((lex(bytes(&b"list"[..])), lex(data_type()))),
        lex(data_type()),
        lex(identity()),
    )
        .map(|(_, list, data_type, name)| Property { name, count_data_type: list.map(|l| l.1), data_type })
        .expected("a property statement")
}

/// Parses element statements.
fn element_stmt<'a, I>() -> impl Parser<Input = I, Output = (&'a [u8], String, usize)> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        lex(range(&b"element"[..])),
        lex(identity()),
        lex(unsigned_integer()),
    )
        .expected("an element statement")
}

/// Parses an element and its properties.
fn element_group<'a, I>() -> impl Parser<Input = I, Output = Element> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        many::<Vec<_>, _>(comment_stmt()),
        element_stmt(),
        many1::<Vec<_>, _>((
            many::<Vec<_>, _>(comment_stmt()),
            property_stmt(),
        )),
    )
        .map(|(_, (_, name, count), properties)| Element { name, count, properties: properties.into_iter().map(|p| p.1).collect() })
        .expected("an element and at least one property")
}

/// Parses the PLY format statement followed by zero or more elements and their properties.
fn spec_group<'a, I>() -> impl Parser<Input = I, Output = (Format, Vec<Element>)> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        many::<Vec<_>, _>(comment_stmt()),
        format_stmt(),
        many::<Vec<_>, _>(element_group()),
    )
        .map(|(_, format, elements)| (format, elements))
        .expected("a format statement followed by zero or more elements and their properties")
}

/// Parses the entire PLY header. Any comments are ignored.
pub fn header<'a, I>() -> impl Parser<Input = I, Output = Header> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    between(
        begin_header(),
        end_header(),
        spec_group(),
    )
        .map(|(format, elements)| Header { format, elements })
        .expected("a PLY header")
}

fn property_data<'a, I>(format: &'a Format, property: &'a Property) -> impl Parser<Input = I, Output = PropertyData> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
}

fn element_data<'a, I>(format: &'a Format, element: &'a Element) -> impl Parser<Input = I, Output = ElementData> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let count = element.properties.len();
    let mut parsers = element.properties
        .iter()
        .map(|p| property_data(format, p));

    count_min_max(count, count, lazy(move || parsers.next().unwrap()))
        .map(|properties| ElementData { properties })
}

pub fn body<'a, I>(header: &'a Header) -> impl Parser<Input = I, Output = Body> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    let count = header.elements.len();
    let mut parsers = header.elements
        .iter()
        .map(|e| element_data(&header.format, e));

    count_min_max(count, count, lazy(move || parsers.next().unwrap()))
        .map(|elements| Body { elements })
}

pub fn ply<'a, I>() -> impl Parser<Input = I, Output = Ply> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    header()
        .then(|h| (value(h), body(&h)))
        .map(|(h, b)| Ply { header: h, body: b })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_identity_char() {
        let expected = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._";
        assert!(expected.iter().all(|b| is_identity(*b)));
    }

    #[test]
    fn ply_format_statement() {
        let stream = b"format ascii 1.0\n";
        let expected = Format {
            format: FormatType::Ascii,
            version: vec![1, 0],
        };

        let r = format_stmt().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_comment_statement() {
        let stream = b"comment Hello, World!\n";
        let expected = String::from("Hello, World!");

        let r = comment_stmt().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_element_statement() {
        let stream = b"element some_name 10";
        let expected = (&b"element"[..], String::from("some_name"), 10usize);

        let r = element_stmt().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_property_statement() {
        let stream = b"property uchar property_a";
        let expected = Property {
            name: "property_a".into(),
            count_data_type: None,
            data_type: DataType::Uint8,
        };

        let r = property_stmt().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);

        let stream = b"property list uchar uchar property_a";
        let expected = Property {
            name: "property_a".into(),
            count_data_type: Some(DataType::Uint8),
            data_type: DataType::Uint8,
        };

        let r = property_stmt().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_header_minimal() {
        let stream = b"ply\nformat ascii 1.0\nend_header\n";
        let expected = Header {
            format: Format {
                format: FormatType::Ascii,
                version: vec![1, 0]
            },
            elements: Vec::new()
        };

        let r = header().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_header_mixed_newlines() {
        let stream = b"ply\nformat ascii 1.0\r\nelement face 3\rproperty list uint8 uint32 vertex_indices\nend_header\r\n";

        assert_ok!(header().easy_parse(&stream[..]));
    }

    #[test]
    fn ply_header_element_scalar() {
        let stream = b"ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nend_header\n";
        let expected = Header {
            format: Format {
                format: FormatType::Ascii,
                version: vec![1, 0],
            },
            elements: vec![
                Element {
                    name: "vertex".into(),
                    count: 3,
                    properties: vec![
                        Property {
                            name: "x".into(),
                            count_data_type: None,
                            data_type: DataType::Float32,
                        }
                    ],
                }
            ],
        };

        let r = header().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_header_element_vector() {
        let stream = b"ply\nformat ascii 1.0\nelement face 3\nproperty list uint8 uint32 vertex_indices\nend_header\n";
        let expected = Header {
            format: Format {
                format: FormatType::Ascii,
                version: vec![1, 0],
            },
            elements: vec![
                Element {
                    name: "face".into(),
                    count: 3,
                    properties: vec![
                        Property {
                            name: "vertex_indices".into(),
                            count_data_type: Some(DataType::Uint8),
                            data_type: DataType::Uint32,
                        }
                    ],
                }
            ],
        };

        let r = header().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_header_element_scalar_vector() {
        let stream = b"ply\nformat ascii 1.0\nelement face 3\nproperty float x\nproperty list uint8 uint32 vertex_indices\nend_header\n";
        let expected = Header {
            format: Format {
                format: FormatType::Ascii,
                version: vec![1, 0],
            },
            elements: vec![
                Element {
                    name: "face".into(),
                    count: 3,
                    properties: vec![
                        Property {
                            name: "x".into(),
                            count_data_type: None,
                            data_type: DataType::Float32,
                        },
                        Property {
                            name: "vertex_indices".into(),
                            count_data_type: Some(DataType::Uint8),
                            data_type: DataType::Uint32,
                        },
                    ],
                }
            ],
        };

        let r = header().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_header_element_vector_scalar() {
        let stream = b"ply\nformat ascii 1.0\nelement face 3\nproperty list uint8 uint32 vertex_indices\nproperty float x\nend_header\n";
        let expected = Header {
            format: Format {
                format: FormatType::Ascii,
                version: vec![1, 0],
            },
            elements: vec![
                Element {
                    name: "face".into(),
                    count: 3,
                    properties: vec![
                        Property {
                            name: "vertex_indices".into(),
                            count_data_type: Some(DataType::Uint8),
                            data_type: DataType::Uint32,
                        },
                        Property {
                            name: "x".into(),
                            count_data_type: None,
                            data_type: DataType::Float32,
                        },
                    ],
                }
            ],
        };

        let r = header().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_header_comments() {
        let stream_a = b"ply\ncomment Hello, World!\nformat ascii 1.0\nelement vertex 3\nproperty float x\nend_header\n";
        let stream_b = b"ply\nformat ascii 1.0\ncomment How are you?\nelement vertex 3\nproperty float x\nend_header\n";
        let stream_c = b"ply\nformat ascii 1.0\nelement vertex 3\ncomment This is a special property\nproperty float x\nend_header\n";
        let stream_d = b"ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\ncomment I am done now\nend_header\n";

        assert_ok!(header().easy_parse(&stream_a[..]));
        assert_ok!(header().easy_parse(&stream_b[..]));
        assert_ok!(header().easy_parse(&stream_c[..]));
        assert_err!(header().easy_parse(&stream_d[..]));
    }
}
