use super::base::{eol, lex, identity, keyword, ascii_unsigned_integral};
use types::{DataType, CountType, Element, Format, FormatType, Header, Property};
use combine::{
    error::ParseError,
    parser::{
        byte::{byte, spaces},
        choice::{choice, optional},
        combinator::{look_ahead, attempt},
        item::{tokens, tokens2},
        repeat::{many, many1, sep_by, take_until},
        sequence::between,
        Parser,
    },
    stream::Stream,
};

/// Parses the beginning of the ply header.
fn begin_header<'a, I>() -> impl Parser<Input = I, Output = ()> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    keyword(&b"ply"[..])
        .skip(spaces())
        .map(|_| ())
        .expected("the beginning of the ply header")
}

/// Parses the end of the ply header.
fn end_header<'a, I>() -> impl Parser<Input = I, Output = ()> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    keyword(&b"end_header"[..])
        .skip(spaces())
        .map(|_| ())
        .expected("the end of the ply header")
}

/// Parses a data type.
fn data_type<'a, I>() -> impl Parser<Input = I, Output = DataType> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice([
        attempt(keyword(&b"int8"[..])),
        attempt(keyword(&b"char"[..])),
        attempt(keyword(&b"uint8"[..])),
        attempt(keyword(&b"uchar"[..])),
        attempt(keyword(&b"int16"[..])),
        attempt(keyword(&b"short"[..])),
        attempt(keyword(&b"uint16"[..])),
        attempt(keyword(&b"ushort"[..])),
        attempt(keyword(&b"int32"[..])),
        attempt(keyword(&b"int"[..])),
        attempt(keyword(&b"uint32"[..])),
        attempt(keyword(&b"uint"[..])),
        attempt(keyword(&b"float32"[..])),
        attempt(keyword(&b"float"[..])),
        attempt(keyword(&b"float64"[..])),
        attempt(keyword(&b"double"[..])),
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
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice([
        attempt(keyword(&b"uint8"[..])),
        attempt(keyword(&b"uchar"[..])),
        attempt(keyword(&b"uint16"[..])),
        attempt(keyword(&b"ushort"[..])),
        attempt(keyword(&b"uint32"[..])),
        attempt(keyword(&b"uint"[..])),
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

/// Parses the ply format type.
fn format_type<'a, I>() -> impl Parser<Input = I, Output = FormatType> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    choice([
        attempt(keyword(&b"ascii"[..])),
        attempt(keyword(&b"binary_big_endian"[..])),
        attempt(keyword(&b"binary_little_endian"[..])),
    ]).map(|r: &[u8]| match r {
        b"ascii" => FormatType::Ascii,
        b"binary_big_endian" => FormatType::BinaryBigEndian,
        b"binary_little_endian" => FormatType::BinaryLittleEndian,
        _ => unreachable!(),
    }).expected("a ply format type")
}

/// Parses the ply format version.
fn format_version<'a, I>() -> impl Parser<Input = I, Output = Vec<usize>> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    look_ahead(keyword(&b"1.0"[..]))
        .with(sep_by::<Vec<_>, _, _>(
            ascii_unsigned_integral::<_, usize>(),
            byte(b'.'),
        )).expected("a ply format version with value '1.0'")
}

/// Parses the ply format statement.
fn format_stmt<'a, I>() -> impl Parser<Input = I, Output = Format> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (lex(keyword(&b"format"[..])), lex(format_type()), lex(format_version()))
        .map(|(_, format, version)| Format { format, version })
        .expected("a format statement")
}

/// Parses comment statements.
fn comment_stmt<'a, I>() -> impl Parser<Input = I, Output = String> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (lex(keyword(&b"comment"[..])), lex(take_until::<Vec<_>, _>(eol())))
        .map(|(_, c)| String::from_utf8_lossy(&c).to_string())
        .expected("a comment statement")
}

/// Parses property statements (scalars and vectors).
fn property_stmt<'a, I>() -> impl Parser<Input = I, Output = Property> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        lex(keyword(&b"property"[..])),
        optional((lex(keyword(&b"list"[..])), lex(count_type()))),
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
fn element_stmt<'a, I>() -> impl Parser<Input = I, Output = (String, usize)> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    lex(keyword(&b"element"[..]))
        .with((lex(identity()), lex(ascii_unsigned_integral::<_, usize>())))
        .expected("an element statement")
}

/// Parses an element and its properties.
fn element_group<'a, I>() -> impl Parser<Input = I, Output = Element> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        many::<Vec<_>, _>(attempt(comment_stmt())),
        element_stmt(),
        many1::<Vec<_>, _>((many::<Vec<_>, _>(comment_stmt()), property_stmt())),
    )
        .map(|(_, (name, count), properties)| Element {
            name,
            count,
            properties: properties.into_iter().map(|p| p.1).collect(),
        }).expected("an element and at least one property")
}

/// Parses the ply format statement followed by zero or more elements and their properties.
fn spec_group<'a, I>() -> impl Parser<Input = I, Output = (Format, Vec<Element>)> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    (
        many::<Vec<_>, _>(attempt(comment_stmt())),
        format_stmt(),
        many::<Vec<_>, _>(attempt(element_group())),
    )
        .map(|(_, format, elements)| (format, elements))
        .expected("a format statement followed by zero or more elements and their properties")
}

/// Parses the entire ply header. Any comments are ignored.
pub fn header<'a, I>() -> impl Parser<Input = I, Output = Header> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    between(begin_header(), end_header(), spec_group())
        .map(|(format, elements)| Header { format, elements })
        .expected("a ply header")
}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::stream::{ReadStream, buffered::BufferedStream, state::State};

    const BUFFER_SIZE: usize = 32;

    #[test]
    fn ply_format_statement() {
        let stream = b"format ascii 1.0\n";
        let expected = Format {
            format: FormatType::Ascii,
            version: vec![1, 0],
        };

        let r = format_stmt().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE));
        assert_ok2!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_unknown_version() {
        let stream = b"format ascii 1.1\n";

        let r = format_stmt().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE));
        assert_err2!(r);
    }

    #[test]
    fn ply_comment_statement() {
        let stream = b"comment Hello, World!\n";
        let expected = String::from("Hello, World!");

        let r = comment_stmt().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE));
        assert_ok2!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_element_statement() {
        let stream = b"element some_name 10\n";
        let expected = (String::from("some_name"), 10usize);

        let r = element_stmt().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE));
        assert_ok2!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_property_statement() {
        let stream = b"property uchar property_a\n";
        let expected = Property {
            name: "property_a".into(),
            count_data_type: None,
            data_type: DataType::Uint8,
        };

        let r = property_stmt().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE));
        assert_ok2!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);

        let stream = b"property list uchar uchar property_a\n";
        let expected = Property {
            name: "property_a".into(),
            count_data_type: Some(CountType::Uint8),
            data_type: DataType::Uint8,
        };

        let r = property_stmt().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE));
        assert_ok2!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_header_minimal() {
        let stream = b"ply format ascii 1.0 end_header ";
        let expected = Header {
            format: Format {
                format: FormatType::Ascii,
                version: vec![1, 0],
            },
            elements: Vec::new(),
        };

        let r = header().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE));
        assert_ok2!(r);
        let r = r.unwrap();
        assert_eq!(r.0, expected);
    }

    #[test]
    fn ply_header_mixed_newlines() {
        let stream =
            b"ply\nformat ascii 1.0\r\nelement face 3\rproperty list uint8 uint32 vertex_indices\nend_header\r\n";

        assert_ok2!(header().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE)));
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

        let r = header().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE));
        assert_ok2!(r);
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

        let r = header().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE));
        assert_ok2!(r);
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

        let r = header().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE));
        assert_ok2!(r);
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

        let r = header().parse(BufferedStream::new(State::new(ReadStream::new(&stream[..])), BUFFER_SIZE));
        assert_ok2!(r);
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

        assert_ok2!(header().parse(BufferedStream::new(State::new(ReadStream::new(&stream_a[..])), BUFFER_SIZE)));
        assert_ok2!(header().parse(BufferedStream::new(State::new(ReadStream::new(&stream_b[..])), BUFFER_SIZE)));
        assert_ok2!(header().parse(BufferedStream::new(State::new(ReadStream::new(&stream_c[..])), BUFFER_SIZE)));
        assert_err2!(header().parse(BufferedStream::new(State::new(ReadStream::new(&stream_d[..])), BUFFER_SIZE)));
    }
}
