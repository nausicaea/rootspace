use nom::{bytes::complete::{tag, take_while1, take_till1}, IResult, branch::alt, sequence::{pair, delimited, separated_pair, terminated, tuple}, character::{is_space, is_newline, complete::{alphanumeric1, alpha1}}, combinator::recognize, multi::many0_count};

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
    recognize(
        pair(
            alt((alpha1, tag(b"_"))),
            many0_count(alt((alphanumeric1, tag(b"_"))))
        )
    )(input)
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

fn format_type(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        tag(ASCII),
        tag(BINARY_BIG_ENDIAN),
        tag(BINARY_LITTLE_ENDIAN),
    ))(input)
}

fn format_version(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(b"1.0")(input)
}

fn format_stmt(input: &[u8]) -> IResult<&[u8], &[u8]> {
    terminated(delimited(pair(format_kwd, space), format_type, pair(space, format_version)), newline)(input)
}

fn comment_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(COMMENT)(input)
}

fn comment_stmt(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    terminated(separated_pair(comment_kwd, space, single_line_text), newline)(input)
}

fn obj_info_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(OBJ_INFO)(input)
}

fn obj_info_stmt(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    terminated(separated_pair(obj_info_kwd, space, single_line_text), newline)(input)
}

fn property_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(PROPERTY)(input)
}

fn list_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(LIST)(input)
}

fn data_type(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        tag(FLOAT64),
        tag(FLOAT32),
        tag(USHORT),
        tag(UINT32),
        tag(UINT16),
        tag(DOUBLE),
        tag(UINT8),
        tag(UCHAR),
        tag(SHORT),
        tag(INT32),
        tag(INT16),
        tag(FLOAT),
        tag(UINT),
        tag(INT8),
        tag(CHAR),
        tag(INT),
    ))(input)
}

fn count_type(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        tag(USHORT),
        tag(UINT32),
        tag(UINT16),
        tag(UINT8),
        tag(UCHAR),
        tag(UINT),
    ))(input)
}

fn property_stmt(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8], &[u8])> {
    todo!()
}

#[cfg(test)]
mod tests {
    use proptest::{proptest, prop_assert_eq, string::bytes_regex};
    use nom::error::dbg_dmp;

    use super::*;

    const EMPTY: &'static [u8] = b"";

    proptest!{
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

        #[test]
        fn format_stmt_matches_keyword_type_version_and_a_newline_returns_only_type(ref input in bytes_regex(r"format[\x09\x20](ascii|binary_big_endian|binary_little_endian)[\x09\x20]1\.0\x0a").unwrap()) {
            assert_eq!(format_stmt(&input[..]), Ok((EMPTY, &input[7..input.len()-5])))
        }

        #[test]
        fn comment_stmt_matches_keyword_freetext_and_a_newline_returns_everything(ref input in bytes_regex(r"comment[\x09\x20][\w]+\x0a").unwrap()) {
            prop_assert_eq!(dbg_dmp(comment_stmt, "proptest_comment")(&input[..]), Ok((EMPTY, (COMMENT, &input[8..input.len()-1]))))
        }

        #[test]
        fn obj_info_stmt_matches_keyword_freetext_and_a_newline_returns_everything(ref input in bytes_regex(r"obj_info[\x09\x20][\w]+\x0a").unwrap()) {
            prop_assert_eq!(dbg_dmp(obj_info_stmt, "proptest_obj_info")(&input[..]), Ok((EMPTY, (OBJ_INFO, &input[9..input.len()-1]))))
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
