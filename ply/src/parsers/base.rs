use combine::{
    error::ParseError,
    parser::{
        byte::{crlf, digit, newline, spaces},
        choice::optional,
        combinator::recognize,
        item::token,
        range::take_while1,
        repeat::{many1, skip_many, skip_many1},
        Parser,
    },
    stream::{RangeStream, Stream, StreamOnce},
};
use num_traits::{cast, Float, Num, PrimInt, Signed, Unsigned};

/// Returns true if the supplied byteacter is ASCII alphabetic.
fn is_alphabetic(b: u8) -> bool {
    match b {
        b'a'..=b'z' | b'A'..=b'Z' => true,
        _ => false,
    }
}

/// Returns true if the supplied byteacter is ASCII numeric.
fn is_numeric(b: u8) -> bool {
    match b {
        b'0'..=b'9' => true,
        _ => false,
    }
}

/// Returns true if the supplied byteacter is ASCII alphanumeric or a limited set of special
/// characters.
fn is_identity(b: u8) -> bool {
    is_alphabetic(b) || is_numeric(b) || b".-_".iter().any(|s| s == &b)
}

/// Parses a set of characters if it can be interpreted as a name or identity.
pub fn identity<'a, I>() -> impl Parser<Input = I, Output = String> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    take_while1(is_identity)
        .map(|s| String::from_utf8_lossy(s).to_string())
        .expected("a name or identity")
}

/// Parses end-of-line sequences and maps them to the LF ASCII character.
pub fn eol<'a, I>() -> impl Parser<Input = I, Output = u8> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    crlf().or(newline()).expected("a line termination byte sequence")
}

/// Skips any whitespace after the supplied parser.
pub fn lex<'a, P>(parser: P) -> impl Parser<Input = P::Input, Output = P::Output>
where
    P: Parser,
    P::Input: Stream<Item = u8, Range = &'a [u8]> + 'a,
    <P::Input as StreamOnce>::Error:
        ParseError<<P::Input as StreamOnce>::Item, <P::Input as StreamOnce>::Range, <P::Input as StreamOnce>::Position>,
{
    parser.skip(spaces())
}

/// Parses an unsigned integer from a stream of numeric ASCII characters.
pub fn ascii_unsigned_integral<'a, I, O>() -> impl Parser<Input = I, Output = O> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    O: Num + PrimInt + Unsigned,
{
    many1::<Vec<_>, _>(digit())
        .map(|v| {
            let mut n: O = O::zero();
            for byte in v {
                n = n * cast::<_, O>(10).unwrap() + (cast::<_, O>(byte).unwrap() - cast::<_, O>(b'0').unwrap());
            }
            n
        }).expected("an unsigned integer")
}

/// Parses a signed integer from a stream of numeric ASCII characters.
pub fn ascii_signed_integral<'a, I, O>() -> impl Parser<Input = I, Output = O> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    O: Num + PrimInt + Signed,
{
    (optional(token(b'-')), many1::<Vec<_>, _>(digit()))
        .map(|(s, v)| {
            let mut n: O = O::zero();
            for byte in v {
                n = n * cast::<_, O>(10).unwrap() + (cast::<_, O>(byte).unwrap() - cast::<_, O>(b'0').unwrap());
            }

            if let Some(b'-') = s {
                -n
            } else {
                n
            }
        }).expected("a signed integer")
}

pub fn ascii_floating_point<'a, I, O>() -> impl Parser<Input = I, Output = O> + 'a
where
    I: RangeStream<Item = u8, Range = &'a [u8]> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
    O: Num + Float + ::std::str::FromStr,
    <O as ::std::str::FromStr>::Err: ::std::fmt::Debug,
{
    recognize::<Vec<_>, _>((
        optional(token(b'-')),
        skip_many1(digit()),
        optional((token(b'.'), skip_many(digit()))),
    )).map(|bs| {
        String::from_utf8(bs)
            .expect("Failed to parse validated parser output as UTF-8 string")
            .parse::<O>()
            .expect("Failed to parse validated parser output as signed float")
    }).expected("a signed float")
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
    fn is_eol() {
        assert_ok!(eol().easy_parse(&b"\r\n"[..]));
        assert_ok!(eol().easy_parse(&b"\n"[..]));
    }

    #[test]
    fn can_lex() {
        let mut parser = lex(digit());

        assert_ok!(parser.easy_parse(&b"9 "[..]));
    }

    #[test]
    fn unsigned_int() {
        assert_eq!(ascii_unsigned_integral::<_, u32>().easy_parse(&b"10"[..]), Ok((10u32, &b""[..])));
    }
}
