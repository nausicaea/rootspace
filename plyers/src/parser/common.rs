use either::{Either, Left, Right};
use nom::{
branch::alt,
bytes::complete::{tag, take_till1, take_while1},
character::{
    complete::{alpha1, alphanumeric1},
    is_newline, is_space,
},
combinator::recognize,
error::ParseError,
multi::many0_count,
sequence::pair,
IResult,
};

pub fn split_vecs_of_either<L, R>(mut input: Vec<Either<L, R>>) -> (Vec<L>, Vec<R>) {
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

pub fn is_whitespace(b: u8) -> bool {
    (b >= 0x09 && b <= 0x0d) || (b == 0x20)
}

pub fn whitespace<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E> {
    take_while1(is_whitespace)(input)
}

pub fn space<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E> {
    take_while1(is_space)(input)
}

pub fn newline<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E> {
    take_while1(is_newline)(input)
}

pub fn single_line_text<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E> {
    take_till1(is_newline)(input)
}

pub fn identifier<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], &'a [u8], E> {
    recognize(pair(
        alt((alpha1, tag(b"_"))),
        many0_count(alt((alphanumeric1, tag(b"_")))),
    ))(input)
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
            prop_assert_eq!(dbg_dmp(space::<nom::error::Error<_>>, "proptest_space")(&input[..]), Ok((EMPTY, &input[..])))
        }

        #[test]
        fn newline_matches_x0a(ref input in bytes_regex(r"\x0a+").unwrap()) {
            prop_assert_eq!(dbg_dmp(newline::<nom::error::Error<_>>, "proptest_newline")(&input[..]), Ok((EMPTY, &input[..])))
        }

        #[test]
        fn single_line_text_matches_anything_till_exclusive_x0a(ref input in bytes_regex(r"[\w]+\x0a").unwrap()) {
            prop_assert_eq!(dbg_dmp(single_line_text::<nom::error::Error<_>>, "proptest_single_line_text")(&input[..]), Ok((&b"\n"[..], &input[..input.len()-1])))
        }

        #[test]
        fn whitespace_matches_all_ascii_whitespace(ref input in bytes_regex(r"[\x09-\x0d\x20]+").unwrap()) {
            prop_assert_eq!(dbg_dmp(whitespace::<nom::error::Error<_>>, "proptest_whitespace")(&input[..]), Ok((EMPTY, &input[..])))
        }
    }
}
