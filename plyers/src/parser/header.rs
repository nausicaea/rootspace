use either::{Either, Left, Right};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, map_res, opt, value},
    error::{FromExternalError, ParseError},
    multi::{many0, many1},
    sequence::tuple,
    IResult,
};

use super::{
    common::{identifier, newline, single_line_text, space, split_vecs_of_either, whitespace},
    ParseNumError,
};
use crate::types::{
    CommentDescriptor, CountType, DataType, ElementDescriptor, FormatType, ListPropertyDescriptor, ObjInfoDescriptor,
    PlyDescriptor, PropertyDescriptor,
};

const PLY: &'static [u8] = b"ply";
const END_HEADER: &'static [u8] = b"end_header";
const FORMAT: &'static [u8] = b"format";
const ELEMENT: &'static [u8] = b"element";
const COMMENT: &'static [u8] = b"comment";
const OBJ_INFO: &'static [u8] = b"obj_info";
const PROPERTY: &'static [u8] = b"property";
const PROPERTY_LIST: &'static [u8] = b"property list";
const ASCII: &'static [u8] = b"ascii";
const BINARY_LITTLE_ENDIAN: &'static [u8] = b"binary_little_endian";
const BINARY_BIG_ENDIAN: &'static [u8] = b"binary_big_endian";

const FLOAT64: &'static [u8] = b"float64";
const FLOAT32: &'static [u8] = b"float32";

const USHORT: &'static [u8] = b"ushort";
const UINT32: &'static [u8] = b"uint32";
const UINT16: &'static [u8] = b"uint16";
const DOUBLE: &'static [u8] = b"double";

const UINT8: &'static [u8] = b"uint8";
const UCHAR: &'static [u8] = b"uchar";
const SHORT: &'static [u8] = b"short";
const INT32: &'static [u8] = b"int32";
const INT16: &'static [u8] = b"int16";
const FLOAT: &'static [u8] = b"float";

const UINT: &'static [u8] = b"uint";
const INT8: &'static [u8] = b"int8";
const CHAR: &'static [u8] = b"char";

const INT: &'static [u8] = b"int";

fn format_type<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], FormatType, E> {
    alt((
        value(FormatType::Ascii, tag(ASCII)),
        value(FormatType::BinaryBigEndian, tag(BINARY_BIG_ENDIAN)),
        value(FormatType::BinaryLittleEndian, tag(BINARY_LITTLE_ENDIAN)),
    ))(input)
}

fn format_version<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E> {
    tag(b"1.0")(input)
}

fn format_decl<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], FormatType, E> {
    map(
        tuple((tag(FORMAT), space, format_type, space, format_version, newline)),
        |(_, _, ft, _, _, _)| ft,
    )(input)
}

fn comment_decl<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], CommentDescriptor, E> {
    map(
        tuple((tag(COMMENT), space, single_line_text, newline)),
        |(_, _, c, _)| CommentDescriptor(String::from_utf8_lossy(c).to_string()),
    )(input)
}

fn obj_info_decl<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], ObjInfoDescriptor, E> {
    map(
        tuple((tag(OBJ_INFO), space, single_line_text, newline)),
        |(_, _, c, _)| ObjInfoDescriptor(String::from_utf8_lossy(c).to_string()),
    )(input)
}

fn element_decl<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (String, usize), E> {
    map_res(
        tuple((tag(ELEMENT), space, identifier, space, digit1, newline)),
        |(_, _, nm, _, cnt, _)| {
            let nm = std::str::from_utf8(nm)?.to_string();
            let cnt = std::str::from_utf8(cnt)?;
            let cnt: usize = usize::from_str_radix(cnt, 10)?;

            Result::<_, ParseNumError>::Ok((nm, cnt))
        },
    )(input)
}

fn data_type<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], DataType, E> {
    alt((
        value(DataType::F64, tag(FLOAT64)),
        value(DataType::F32, tag(FLOAT32)),
        value(DataType::U16, tag(USHORT)),
        value(DataType::U32, tag(UINT32)),
        value(DataType::U16, tag(UINT16)),
        value(DataType::F64, tag(DOUBLE)),
        value(DataType::U8, tag(UINT8)),
        value(DataType::U8, tag(UCHAR)),
        value(DataType::I16, tag(SHORT)),
        value(DataType::I32, tag(INT32)),
        value(DataType::I16, tag(INT16)),
        value(DataType::F32, tag(FLOAT)),
        value(DataType::U32, tag(UINT)),
        value(DataType::I8, tag(INT8)),
        value(DataType::I8, tag(CHAR)),
        value(DataType::I32, tag(INT)),
    ))(input)
}

fn count_type<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], CountType, E> {
    alt((
        value(CountType::U16, tag(USHORT)),
        value(CountType::U32, tag(UINT32)),
        value(CountType::U16, tag(UINT16)),
        value(CountType::U8, tag(UINT8)),
        value(CountType::U8, tag(UCHAR)),
        value(CountType::U32, tag(UINT)),
    ))(input)
}

fn property_scalar_decl<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], (DataType, String), E> {
    map(
        tuple((tag(PROPERTY), space, data_type, space, identifier, newline)),
        |(_, _, dt, _, nm, _)| (dt, String::from_utf8_lossy(nm).to_string()),
    )(input)
}

fn property_list_decl<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (CountType, DataType, String), E> {
    map(
        tuple((
            tag(PROPERTY_LIST),
            space,
            count_type,
            space,
            data_type,
            space,
            identifier,
            newline,
        )),
        |(_, _, ct, _, dt, _, nm, _)| (ct, dt, String::from_utf8_lossy(nm).to_string()),
    )(input)
}

fn comment_blk<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Vec<Either<CommentDescriptor, ObjInfoDescriptor>>, E> {
    many0(alt((map(comment_decl, Left), map(obj_info_decl, Right))))(input)
}

fn format_blk<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], (Vec<Either<CommentDescriptor, ObjInfoDescriptor>>, FormatType), E> {
    tuple((comment_blk, format_decl))(input)
}

fn property_scalar_rpt<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], PropertyDescriptor, E> {
    map(
        tuple((comment_blk, property_scalar_decl)),
        |(cmt, (data_type, name))| {
            let (comments, obj_info) = split_vecs_of_either(cmt);

            PropertyDescriptor {
                data_type,
                name,
                comments,
                obj_info,
            }
        },
    )(input)
}

fn property_scalar_blk<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Vec<PropertyDescriptor>, E> {
    many1(property_scalar_rpt)(input)
}

fn property_list_rpt<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], ListPropertyDescriptor, E> {
    map(
        tuple((comment_blk, property_list_decl)),
        |(cmt, (count_type, data_type, name))| {
            let (comments, obj_info) = split_vecs_of_either(cmt);

            ListPropertyDescriptor {
                count_type,
                data_type,
                name,
                comments,
                obj_info,
            }
        },
    )(input)
}

fn property_list_blk<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Vec<ListPropertyDescriptor>, E> {
    many1(property_list_rpt)(input)
}

fn property_blk<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Either<Vec<PropertyDescriptor>, Vec<ListPropertyDescriptor>>, E> {
    alt((map(property_list_blk, Right), map(property_scalar_blk, Left)))(input)
}

fn element_rpt<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError>>(
    input: &'a [u8],
) -> IResult<&'a [u8], ElementDescriptor, E> {
    map(
        tuple((comment_blk, element_decl, property_blk)),
        |(cmt, (name, count), prp)| {
            let (comments, obj_info) = split_vecs_of_either(cmt);

            match prp {
                Left(properties) => ElementDescriptor {
                    name,
                    count,
                    properties,
                    list_properties: Vec::new(),
                    comments,
                    obj_info,
                },
                Right(list_properties) => ElementDescriptor {
                    name,
                    count,
                    properties: Vec::new(),
                    list_properties,
                    comments,
                    obj_info,
                },
            }
        },
    )(input)
}

fn element_blk<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Vec<ElementDescriptor>, E> {
    many1(element_rpt)(input)
}

pub fn header<'a, E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError>>(
    input: &'a [u8],
) -> IResult<&'a [u8], PlyDescriptor, E> {
    map(
        tuple((
            opt(whitespace),
            tag(PLY),
            newline,
            format_blk,
            element_blk,
            tag(END_HEADER),
            newline,
        )),
        |(_, _, _, (cmt, format_type), elements, _, _)| {
            let (comments, obj_info) = split_vecs_of_either(cmt);

            PlyDescriptor {
                format_type,
                elements,
                comments,
                obj_info,
            }
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_minimal_ascii_parses_correctly() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
        assert_eq!(
            header::<nom::error::Error<_>>(&input[..]),
            Ok((
                &b"1.0\n"[..],
                PlyDescriptor {
                    format_type: FormatType::Ascii,
                    elements: vec![ElementDescriptor {
                        name: String::from("vertex"),
                        count: 1usize,
                        properties: vec![PropertyDescriptor {
                            data_type: DataType::F32,
                            name: String::from("x"),
                            comments: Vec::new(),
                            obj_info: Vec::new()
                        }],
                        list_properties: Vec::new(),
                        comments: Vec::new(),
                        obj_info: Vec::new()
                    }],
                    comments: Vec::new(),
                    obj_info: Vec::new()
                }
            ))
        );
    }
}
