use std::collections::BTreeMap;

use either::{Either, Left, Right};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, map_res, opt, value},
    error::{context, ContextError, FromExternalError, ParseError},
    multi::{fold_many1, many0},
    sequence::tuple,
    IResult,
};

use super::super::types::{
    CommentDescriptor, CountType, DataType, ElementDescriptor, ElementId, FormatType, ObjInfoDescriptor, PlyDescriptor,
    PropertyDescriptor, PropertyId,
};
use super::{
    common::{identifier, newline, single_line_text, space, split_vecs_of_either, whitespace},
    ParseNumError,
};
use crate::urn::Urn;

const PLY: &[u8] = b"ply";
const END_HEADER: &[u8] = b"end_header";
const FORMAT: &[u8] = b"format";
const ELEMENT: &[u8] = b"element";
const COMMENT: &[u8] = b"comment";
const OBJ_INFO: &[u8] = b"obj_info";
const PROPERTY: &[u8] = b"property";
const PROPERTY_LIST: &[u8] = b"property list";
const ASCII: &[u8] = b"ascii";
const BINARY_LITTLE_ENDIAN: &[u8] = b"binary_little_endian";
const BINARY_BIG_ENDIAN: &[u8] = b"binary_big_endian";

const FLOAT64: &[u8] = b"float64";
const FLOAT32: &[u8] = b"float32";

const USHORT: &[u8] = b"ushort";
const UINT32: &[u8] = b"uint32";
const UINT16: &[u8] = b"uint16";
const DOUBLE: &[u8] = b"double";

const UINT8: &[u8] = b"uint8";
const UCHAR: &[u8] = b"uchar";
const SHORT: &[u8] = b"short";
const INT32: &[u8] = b"int32";
const INT16: &[u8] = b"int16";
const FLOAT: &[u8] = b"float";

const UINT: &[u8] = b"uint";
const INT8: &[u8] = b"int8";
const CHAR: &[u8] = b"char";

const INT: &[u8] = b"int";

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

fn element_decl<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], (String, usize), E>
where
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]>,
{
    context(
        "plyers::de::header::element_decl",
        map_res(
            tuple((tag(ELEMENT), space, identifier, space, digit1, newline)),
            |(_, _, nm, _, cnt, _)| {
                let nm = std::str::from_utf8(nm)?.to_string();
                let cnt = std::str::from_utf8(cnt)?;
                let cnt: usize = cnt.parse::<usize>()?;

                Result::<_, ParseNumError>::Ok((nm, cnt))
            },
        ),
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

fn comment_blk<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], Vec<Either<CommentDescriptor, ObjInfoDescriptor>>, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context(
        "plyers::de::header::comment_blk",
        many0(alt((map(comment_decl, Left), map(obj_info_decl, Right)))),
    )(input)
}

type FormatBlockOutput = (FormatType, Vec<Either<CommentDescriptor, ObjInfoDescriptor>>);

fn format_blk<'a, E>(
    input: &'a [u8],
) -> IResult<&'a [u8], FormatBlockOutput, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context("plyers::de::header::format_blk", tuple((format_decl, comment_blk)))(input)
}

fn property_scalar_decl<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], PropertyDescriptor, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context(
        "plyers::de::header::property_scalar_decl",
        map(
            tuple((comment_blk, tag(PROPERTY), space, data_type, space, identifier, newline)),
            |(cmt, _, _, data_type, _, name, _)| {
                let (comments, obj_info) = split_vecs_of_either(cmt);

                PropertyDescriptor::Scalar {
                    name: String::from_utf8_lossy(name).to_string(),
                    data_type,
                    comments,
                    obj_info,
                }
            },
        ),
    )(input)
}

fn property_list_decl<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], PropertyDescriptor, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context(
        "plyers::de::header::property_list_decl",
        map(
            tuple((
                comment_blk,
                tag(PROPERTY_LIST),
                space,
                count_type,
                space,
                data_type,
                space,
                identifier,
                newline,
            )),
            |(cmt, _, _, count_type, _, data_type, _, name, _)| {
                let (comments, obj_info) = split_vecs_of_either(cmt);

                PropertyDescriptor::List {
                    name: String::from_utf8_lossy(name).to_string(),
                    count_type,
                    data_type,
                    comments,
                    obj_info,
                }
            },
        ),
    )(input)
}

fn property_rpt<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], PropertyDescriptor, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    context(
        "plyers::de::header::property_rpt",
        alt((property_scalar_decl, property_list_decl)),
    )(input)
}

fn property_blk_fct<'a, 'b, E>(
    p_urn: &'b mut Urn<PropertyId>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], BTreeMap<PropertyId, PropertyDescriptor>, E> + 'b
where
    'a: 'b,
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]> + 'b,
{
    context(
        "plyers::de::header::property_blk",
        fold_many1(
            property_rpt,
            BTreeMap::<PropertyId, PropertyDescriptor>::new,
            |mut p_acc, p| {
                p_acc.insert(p_urn.take(), p);
                p_acc
            },
        ),
    )
}

fn element_rpt_fct<'a, 'b, E>(
    p_urn: &'b mut Urn<PropertyId>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], ElementDescriptor, E> + 'b
where
    'a: 'b,
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]> + 'b,
{
    context(
        "plyers::de::header::element_rpt_fct",
        map(
            tuple((comment_blk, element_decl, property_blk_fct(p_urn))),
            |(cmt, (name, count), properties)| {
                let (comments, obj_info) = split_vecs_of_either(cmt);

                ElementDescriptor {
                    name,
                    count,
                    properties,
                    comments,
                    obj_info,
                }
            },
        ),
    )
}

fn element_blk_fct<'a, 'b, E>(
    e_urn: &'b mut Urn<ElementId>,
    p_urn: &'b mut Urn<PropertyId>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], BTreeMap<ElementId, ElementDescriptor>, E> + 'b
where
    'a: 'b,
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]> + 'b,
{
    context(
        "plyers::de::header::element_blk_fct",
        fold_many1(
            element_rpt_fct(p_urn),
            BTreeMap::<ElementId, ElementDescriptor>::new,
            |mut e_acc, e| {
                e_acc.insert(e_urn.take(), e);
                e_acc
            },
        ),
    )
}

pub fn header_fct<'a, 'b, E>(
    e_urn: &'b mut Urn<ElementId>,
    p_urn: &'b mut Urn<PropertyId>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], PlyDescriptor, E> + 'b
where
    'a: 'b,
    E: ParseError<&'a [u8]> + FromExternalError<&'a [u8], ParseNumError> + ContextError<&'a [u8]> + 'b,
{
    context(
        "plyers::de::header::header_fct",
        map(
            tuple((
                opt(whitespace),
                tag(PLY),
                newline,
                format_blk,
                element_blk_fct(e_urn, p_urn),
                tag(END_HEADER),
                newline,
            )),
            |(_, _, _, (format_type, cmt), elements, _, _)| {
                let (comments, obj_info) = split_vecs_of_either(cmt);

                PlyDescriptor {
                    format_type,
                    elements,
                    comments,
                    obj_info,
                }
            },
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_minimal_ascii_parses_correctly() {
        let input = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/minimal_ascii.ply"));
        let mut e_urn = Urn::<ElementId>::default();
        let mut p_urn = Urn::<PropertyId>::default();
        assert_eq!(
            header_fct::<nom::error::Error<_>>(&mut e_urn, &mut p_urn)(&input[..]),
            Ok((
                &b"1.0\n"[..],
                PlyDescriptor {
                    format_type: FormatType::Ascii,
                    elements: vec![(
                        ElementId(0),
                        ElementDescriptor {
                            name: String::from("vertex"),
                            count: 1usize,
                            properties: vec![(
                                PropertyId(0),
                                PropertyDescriptor::Scalar {
                                    data_type: DataType::F32,
                                    name: String::from("x"),
                                    comments: Vec::new(),
                                    obj_info: Vec::new()
                                }
                            )]
                            .into_iter()
                            .collect(),
                            comments: Vec::new(),
                            obj_info: Vec::new()
                        }
                    )]
                    .into_iter()
                    .collect(),
                    comments: Vec::new(),
                    obj_info: Vec::new()
                }
            ))
        );
    }
}
