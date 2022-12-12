use std::num::ParseIntError;
use either::{Either, Left, Right};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_while1},
    character::{
        complete::{alpha1, alphanumeric1, digit1},
        is_newline, is_space,
    },
    combinator::{map, recognize, value, map_res},
    multi::{many0_count, many0, many1},
    sequence::{pair, tuple},
    IResult,
};

use crate::types::{
    CommentDescriptor, CountType, DataType, FormatType, ListPropertyDescriptor, ObjInfoDescriptor, PropertyDescriptor, ElementDescriptor, PlyDescriptor, Ply,
};

const PLY: &'static [u8] = b"ply";
const END_HEADER: &'static [u8] = b"end_header";
const FORMAT: &'static [u8] = b"format";
const ELEMENT: &'static [u8] = b"element";
const COMMENT: &'static [u8] = b"comment";
const OBJ_INFO: &'static [u8] = b"obj_info";
const PROPERTY: &'static [u8] = b"property";
const LIST: &'static [u8] = b"list";
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

fn split_vecs_of_either<L, R>(mut input: Vec<Either<L, R>>) -> (Vec<L>, Vec<R>) {
    let mut left = Vec::new();
    let mut right = Vec::new();
    for left_right in input.drain(..) {
        match left_right {
            Left(l) => left.push(l),
            Right(r) => right.push(r),
        }
    }

    (left, right)
}

fn space(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(is_space)(input)
}

fn newline(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(is_newline)(input)
}

fn single_line_text(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_till1(is_newline)(input)
}

fn identifier(input: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(pair(
        alt((alpha1, tag(b"_"))),
        many0_count(alt((alphanumeric1, tag(b"_")))),
    ))(input)
}

fn ply_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(PLY)(input)
}

fn end_header_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(END_HEADER)(input)
}

fn format_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(FORMAT)(input)
}

fn format_type(input: &[u8]) -> IResult<&[u8], FormatType> {
    alt((
        value(FormatType::Ascii, tag(ASCII)),
        value(FormatType::BinaryBigEndian, tag(BINARY_BIG_ENDIAN)),
        value(FormatType::BinaryLittleEndian, tag(BINARY_LITTLE_ENDIAN)),
    ))(input)
}

fn format_version(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(b"1.0")(input)
}

fn format_decl(input: &[u8]) -> IResult<&[u8], FormatType> {
    map(
        tuple((format_kwd, space, format_type, space, format_version, newline)),
        |(_, _, ft, _, _, _)| ft,
    )(input)
}

fn comment_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(COMMENT)(input)
}

fn comment_decl(input: &[u8]) -> IResult<&[u8], CommentDescriptor> {
    map(
        tuple((comment_kwd, space, single_line_text, newline)),
        |(_, _, c, _)| CommentDescriptor(String::from_utf8_lossy(c).to_string()),
    )(input)
}

fn obj_info_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(OBJ_INFO)(input)
}

fn obj_info_decl(input: &[u8]) -> IResult<&[u8], ObjInfoDescriptor> {
    map(
        tuple((obj_info_kwd, space, single_line_text, newline)),
        |(_, _, c, _)| ObjInfoDescriptor(String::from_utf8_lossy(c).to_string()),
    )(input)
}

fn element_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(ELEMENT)(input)
}

fn element_decl(input: &[u8]) -> IResult<&[u8], (String, usize)> {
    map_res(
        tuple((element_kwd, space, identifier, space, digit1, newline)),
        |(_, _, nm, _, cnt, _)| {
            let nm = String::from_utf8_lossy(nm).to_string();
            let cnt = String::from_utf8_lossy(cnt);
            let cnt: usize = usize::from_str_radix(&cnt, 10)?;

            Result::<_, ParseIntError>::Ok((nm, cnt))
        },
    )(input)
}

fn property_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(PROPERTY)(input)
}

fn list_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(LIST)(input)
}

fn data_type(input: &[u8]) -> IResult<&[u8], DataType> {
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

fn count_type(input: &[u8]) -> IResult<&[u8], CountType> {
    alt((
        value(CountType::U16, tag(USHORT)),
        value(CountType::U32, tag(UINT32)),
        value(CountType::U16, tag(UINT16)),
        value(CountType::U8, tag(UINT8)),
        value(CountType::U8, tag(UCHAR)),
        value(CountType::U32, tag(UINT)),
    ))(input)
}

fn property_scalar_decl(input: &[u8]) -> IResult<&[u8], (DataType, String)> {
    map(
        tuple((property_kwd, space, data_type, space, identifier, newline)),
        |(_, _, dt, _, nm, _)| (dt, String::from_utf8_lossy(nm).to_string()),
    )(input)
}

fn property_list_decl(input: &[u8]) -> IResult<&[u8], (CountType, DataType, String)> {
    map(
        tuple((
            property_kwd,
            space,
            list_kwd,
            space,
            count_type,
            space,
            data_type,
            space,
            identifier,
            newline,
        )),
        |(_, _, _, _, ct, _, dt, _, nm, _)| (ct, dt, String::from_utf8_lossy(nm).to_string()),
    )(input)
}

fn comment_blk(input: &[u8]) -> IResult<&[u8], Vec<Either<CommentDescriptor, ObjInfoDescriptor>>> {
    many0(alt((map(comment_decl, Left), map(obj_info_decl, Right))))(input)
}

fn format_blk(input: &[u8]) -> IResult<&[u8], (Vec<Either<CommentDescriptor, ObjInfoDescriptor>>, FormatType)> {
    tuple((comment_blk, format_decl))(input)
}

fn property_scalar_rpt(input: &[u8]) -> IResult<&[u8], PropertyDescriptor> {
    map(tuple((comment_blk, property_scalar_decl)), |(cmt, (data_type, name))| {
        let (comments, obj_info) = split_vecs_of_either(cmt);

        PropertyDescriptor {
            data_type,
            name,
            comments,
            obj_info,
        }
    })(input)
}

fn property_scalar_blk(input: &[u8]) -> IResult<&[u8], Vec<PropertyDescriptor>> {
    many1(property_scalar_rpt)(input)
} 

fn property_list_rpt(input: &[u8]) -> IResult<&[u8], ListPropertyDescriptor> {
    map(tuple((comment_blk, property_list_decl)), |(cmt, (count_type, data_type, name))| {
        let (comments, obj_info) = split_vecs_of_either(cmt);

        ListPropertyDescriptor { 
            count_type,
            data_type,
            name,
            comments,
            obj_info,
        }
    })(input)
} 

fn property_list_blk(input: &[u8]) -> IResult<&[u8], Vec<ListPropertyDescriptor>> {
    many1(property_list_rpt)(input)
} 

fn property_blk(input: &[u8]) -> IResult<&[u8], Either<Vec<PropertyDescriptor>, Vec<ListPropertyDescriptor>>> {
    alt((
        map(property_list_blk, Right), 
        map(property_scalar_blk, Left),
    ))(input)
}

fn element_rpt(input: &[u8]) -> IResult<&[u8], ElementDescriptor> {
    map(
        tuple((
            comment_blk,
            element_decl,
            property_blk,
        )),
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
        }
    )(input)
}

fn element_blk(input: &[u8]) -> IResult<&[u8], Vec<ElementDescriptor>> {
    many1(element_rpt)(input)
}

fn header(input: &[u8]) -> IResult<&[u8], PlyDescriptor> {
    map(
        tuple((
            ply_kwd,
            newline,
            format_blk,
            element_blk,
            end_header_kwd,
            newline,
        )),
        |(_, _, (cmt, format_type), elements, _, _)| {
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
    use nom::error::dbg_dmp;
    use proptest::{prop_assert_eq, proptest, string::bytes_regex};

    use super::*;

    const EMPTY: &'static [u8] = b"";

    proptest! {
        #[test]
        fn space_matches_x09_and_x20(ref input in bytes_regex(r"[\x09\x20]+").unwrap()) {
            prop_assert_eq!(dbg_dmp(space, "proptest_space")(&input[..]), Ok((EMPTY, &input[..])))
        }

        #[test]
        fn newline_matches_x0a(ref input in bytes_regex(r"\x0a+").unwrap()) {
            prop_assert_eq!(dbg_dmp(newline, "proptest_newline")(&input[..]), Ok((EMPTY, &input[..])))
        }

        #[test]
        fn single_line_text_matches_anything_till_exclusive_x0a(ref input in bytes_regex(r"[\w]+\x0a").unwrap()) {
            prop_assert_eq!(dbg_dmp(single_line_text, "proptest_single_line_text")(&input[..]), Ok((&b"\n"[..], &input[..input.len()-1])))
        }
    }

    #[test]
    fn ply_kwd_detects_ply_byte_sequence() {
        let input = &b"ply\nformat ascii 1.0\nend_header\n"[..];
        let rest = &b"\nformat ascii 1.0\nend_header\n"[..];

        let r = ply_kwd(input);

        assert_eq!(r, Ok((rest, PLY)))
    }

    #[test]
    fn end_header_kwd_detects_end_header_byte_sequence() {
        let input = b"end_header\n1234\n";
        let rest = b"\n1234\n";

        let r = end_header_kwd(input.as_slice());

        assert_eq!(r, Ok((rest.as_slice(), END_HEADER)))
    }
}
