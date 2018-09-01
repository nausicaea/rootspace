use super::base::{eol, lex, identity, ascii_unsigned_integral};
use types::{DataType, CountType, Element, Format, FormatType, Header, Property};
use combine::{
    error::ParseError,
    parser::{
        byte::{byte, bytes},
        choice::{choice, optional},
        combinator::{look_ahead, try},
        range::range,
        repeat::{many, many1, sep_by, take_until},
        sequence::between,
        Parser,
    },
    stream::RangeStream,
};

/// Parses the beginning of the PLY header.
fn begin_header<'a, I>() -> impl Parser<Input = I, Output = &'a [u8]> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(range(&b"ply"[..])).expected("the beginning of the PLY header")
}

/// Parses the end of the PLY header.
fn end_header<'a, I>() -> impl Parser<Input = I, Output = &'a [u8]> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(range(&b"end_header"[..])).expected("the end of the PLY header")
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
    ]).map(|r: &[u8]| match r {
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
    }).expected("a property data type")
}

fn count_type<'a, I>() -> impl Parser<Input = I, Output = CountType> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice([
        try(bytes(&b"uint8"[..])),
        try(bytes(&b"uchar"[..])),
        try(bytes(&b"uint16"[..])),
        try(bytes(&b"ushort"[..])),
        try(bytes(&b"uint32"[..])),
        try(bytes(&b"uint"[..])),
    ]).map(|r: &[u8]| match r {
        b"uint8" => CountType::Uint8,
        b"uchar" => CountType::Uint8,
        b"uint16" => CountType::Uint16,
        b"ushort" => CountType::Uint16,
        b"uint32" => CountType::Uint32,
        b"uint" => CountType::Uint32,
        _ => unreachable!(),
    }).expected("a count data type (an unsigned integral type)")
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
    ]).map(|r: &[u8]| match r {
        b"ascii" => FormatType::Ascii,
        b"binary_big_endian" => FormatType::BinaryBigEndian,
        b"binary_little_endian" => FormatType::BinaryLittleEndian,
        _ => unreachable!(),
    }).expected("a PLY format type")
}

/// Parses the PLY format version.
fn format_version<'a, I>() -> impl Parser<Input = I, Output = Vec<usize>> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    look_ahead(bytes(&b"1.0"[..]))
        .with(sep_by::<Vec<_>, _, _>(
            ascii_unsigned_integral::<_, usize>(),
            byte(b'.'),
        )).expected("a PLY format version with value '1.0'")
}

/// Parses the PLY format statement.
fn format_stmt<'a, I>() -> impl Parser<Input = I, Output = Format> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (lex(bytes(&b"format"[..])), lex(format_type()), lex(format_version()))
        .map(|(_, format, version)| Format { format, version })
        .expected("a format statement")
}

/// Parses comment statements.
fn comment_stmt<'a, I>() -> impl Parser<Input = I, Output = String> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (lex(bytes(&b"comment"[..])), lex(take_until::<Vec<_>, _>(eol())))
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
        optional((lex(bytes(&b"list"[..])), lex(count_type()))),
        lex(data_type()),
        lex(identity()),
    )
        .map(|(_, list, data_type, name)| Property {
            name,
            count_data_type: list.map(|l| l.1),
            data_type,
        }).expected("a property statement")
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
        lex(ascii_unsigned_integral::<_, usize>()),
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
        many1::<Vec<_>, _>((many::<Vec<_>, _>(comment_stmt()), property_stmt())),
    )
        .map(|(_, (_, name, count), properties)| Element {
            name,
            count,
            properties: properties.into_iter().map(|p| p.1).collect(),
        }).expected("an element and at least one property")
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
    between(begin_header(), end_header(), spec_group())
        .map(|(format, elements)| Header { format, elements })
        .expected("a PLY header")
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn ply_unknown_version() {
        let stream = b"format ascii 1.1\n";

        let r = format_stmt().easy_parse(&stream[..]);
        assert_err!(r);
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
            count_data_type: Some(CountType::Uint8),
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
                version: vec![1, 0],
            },
            elements: Vec::new(),
        };

        let r = header().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_header_mixed_newlines() {
        let stream =
            b"ply\nformat ascii 1.0\r\nelement face 3\rproperty list uint8 uint32 vertex_indices\nend_header\r\n";

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
            elements: vec![Element {
                name: "vertex".into(),
                count: 3,
                properties: vec![Property {
                    name: "x".into(),
                    count_data_type: None,
                    data_type: DataType::Float32,
                }],
            }],
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
            elements: vec![Element {
                name: "face".into(),
                count: 3,
                properties: vec![Property {
                    name: "vertex_indices".into(),
                    count_data_type: Some(CountType::Uint8),
                    data_type: DataType::Uint32,
                }],
            }],
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
            elements: vec![Element {
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
                        count_data_type: Some(CountType::Uint8),
                        data_type: DataType::Uint32,
                    },
                ],
            }],
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
            elements: vec![Element {
                name: "face".into(),
                count: 3,
                properties: vec![
                    Property {
                        name: "vertex_indices".into(),
                        count_data_type: Some(CountType::Uint8),
                        data_type: DataType::Uint32,
                    },
                    Property {
                        name: "x".into(),
                        count_data_type: None,
                        data_type: DataType::Float32,
                    },
                ],
            }],
        };

        let r = header().easy_parse(&stream[..]);
        assert_ok!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_header_comments() {
        let stream_a =
            b"ply\ncomment Hello, World!\nformat ascii 1.0\nelement vertex 3\nproperty float x\nend_header\n";
        let stream_b = b"ply\nformat ascii 1.0\ncomment How are you?\nelement vertex 3\nproperty float x\nend_header\n";
        let stream_c = b"ply\nformat ascii 1.0\nelement vertex 3\ncomment This is a special property\nproperty float x\nend_header\n";
        let stream_d =
            b"ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\ncomment I am done now\nend_header\n";

        assert_ok!(header().easy_parse(&stream_a[..]));
        assert_ok!(header().easy_parse(&stream_b[..]));
        assert_ok!(header().easy_parse(&stream_c[..]));
        assert_err!(header().easy_parse(&stream_d[..]));
    }
}
