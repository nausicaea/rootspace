use nom::{bytes::complete::{tag, take_while1, take_till1}, IResult, branch::alt, sequence::{pair, delimited, separated_pair}, character::{is_space, is_newline}};

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
const CHAR: &'static [u8] = b"char";
const UCHAR: &'static [u8] = b"uchar";
const SHORT: &'static [u8] = b"short";
const USHORT: &'static [u8] = b"ushort";
const INT: &'static [u8] = b"int";
const UINT: &'static [u8] = b"uint";
const FLOAT: &'static [u8] = b"float";
const DOUBLE: &'static [u8] = b"double";
const INT8: &'static [u8] = b"int8";
const UINT8: &'static [u8] = b"uint8";
const INT16: &'static [u8] = b"int16";
const UINT16: &'static [u8] = b"uint16";
const INT32: &'static [u8] = b"int32";
const UINT32: &'static [u8] = b"uint32";
const FLOAT32: &'static [u8] = b"float32";
const FLOAT64: &'static [u8] = b"float64";

/// Tests if byte is ASCII space or tab
///
/// # Example
///
/// ```
/// # use plyers2::parser::is_delimiter;
/// assert_eq!(is_delimiter(0x09), true);
/// assert_eq!(is_delimiter(0x0a), true);
/// assert_eq!(is_delimiter(0x0b), true);
/// assert_eq!(is_delimiter(0x0c), true);
/// assert_eq!(is_delimiter(0x0d), true);
/// assert_eq!(is_delimiter(0x20), true);
/// ```
#[inline]
pub fn is_delimiter(chr: u8) -> bool {
    (chr >= 0x09 && chr <= 0x0d) || (chr == 0x20)
}

fn space(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(is_space)(input)
}

fn newline(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(is_newline)(input)
}

fn delimiter(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(is_delimiter)(input)
}

fn single_line_text(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_till1(is_newline)(input)
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
    delimited(pair(format_kwd, space), format_type, pair(space, format_version))(input)
}

fn comment_kwd(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(COMMENT)(input)
}

fn comment_stmt(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    separated_pair(comment_kwd, space, single_line_text)(input)
}

#[cfg(test)]
mod tests {
    use proptest::{proptest, prop_assert_eq, string::bytes_regex};
    use nom::error::dbg_dmp;

    use super::*;

    proptest!{
        #[test]
        fn space_matches_x09_and_x20(ref input in bytes_regex(r"[\x09\x20]+").unwrap()) {
            prop_assert_eq!(dbg_dmp(space, "proptest_space")(&input[..]), Ok((&b""[..], &input[..])))
        }

        #[test]
        fn newline_matches_x0a(ref input in bytes_regex(r"\x0a+").unwrap()) {
            prop_assert_eq!(dbg_dmp(newline, "proptest_newline")(&input[..]), Ok((&b""[..], &input[..])))
        }

        #[test]
        fn delimiter_matches_ascii_whitespace(ref input in bytes_regex(r"[\x09\x0a\x0b\x0c\x0d\x20]").unwrap()) {
            prop_assert_eq!(dbg_dmp(delimiter, "proptest_delim")(&input[..]), Ok((&b""[..], &input[..])))
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

        assert_eq!(r, Ok((rest, PLY)));
    }

    #[test]
    fn end_header_kwd_detects_end_header_byte_sequence() {
        let input = b"end_header\n1234\n";
        let rest = b"\n1234\n";

        let r = end_header_kwd(input.as_slice());

        assert_eq!(r, Ok((rest.as_slice(), END_HEADER)));
    }

    #[test]
    fn format_stmt_matches_keyword_format_and_version_and_returns_the_latter_two() {
        let input = b"end_header\n1234\n";
        let rest = b"\n1234\n";
    }
}
