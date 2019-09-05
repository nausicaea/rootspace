use combine::{
    error::ParseError,
    parser::{
        byte::{crlf, digit, newline, spaces},
        choice::optional,
        combinator::recognize,
        item::{satisfy, token, tokens2},
        repeat::{many1, skip_many, skip_many1},
        Parser,
    },
    stream::{Stream, StreamOnce},
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
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    many1::<Vec<_>, _>(satisfy(is_identity))
        .map(|s| String::from_utf8_lossy(&s).to_string())
        .expected("a name or identity")
}

/// Parses end-of-line sequences and maps them to the LF ASCII character.
pub fn eol<'a, I>() -> impl Parser<Input = I, Output = u8> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    crlf().or(newline()).expected("a line termination byte sequence")
}

/// Skips any whitespace after the supplied parser.
pub fn lex<'a, P>(parser: P) -> impl Parser<Input = P::Input, Output = P::Output>
where
    P: Parser,
    P::Input: Stream<Item = u8, Range = u8> + 'a,
    <P::Input as StreamOnce>::Error:
        ParseError<<P::Input as StreamOnce>::Item, <P::Input as StreamOnce>::Range, <P::Input as StreamOnce>::Position>,
{
    parser.skip(spaces())
}

/// Matches the specified keyword, or sequence of bytes.
pub fn keyword<'a, I>(kw: &'a [u8]) -> impl Parser<Input = I, Output = &'a [u8]> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
    I::Error: ParseError<I::Item, I::Range, I::Position>,
{
    fn cmp(l: &u8, r: u8) -> bool {
        l == &r
    }

    tokens2(cmp, kw)
}

/// Parses an unsigned integer from a stream of numeric ASCII characters.
pub fn ascii_unsigned_integral<'a, I, O>() -> impl Parser<Input = I, Output = O> + 'a
where
    I: Stream<Item = u8, Range = u8> + 'a,
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
    I: Stream<Item = u8, Range = u8> + 'a,
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
    I: Stream<Item = u8, Range = u8> + 'a,
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
    use combine::stream::{buffered::BufferedStream, state::State, ReadStream};

    #[test]
    fn is_eol() {
        let stream = BufferedStream::new(State::new(ReadStream::new(&b"\r\n"[..])), 1);
        assert!(eol().parse(stream).is_ok());

        let stream = BufferedStream::new(State::new(ReadStream::new(&b"\n"[..])), 1);
        assert!(eol().parse(stream).is_ok());
    }

    #[test]
    fn can_lex() {
        let stream = BufferedStream::new(State::new(ReadStream::new(&b"9 "[..])), 1);
        assert!(lex(digit()).parse(stream).is_ok());
    }

    #[test]
    fn unsigned_int() {
        let stream = BufferedStream::new(State::new(ReadStream::new(&b"10"[..])), 1);
        let r = ascii_unsigned_integral::<_, u32>().parse(stream);
        assert!(r.is_ok());
        if let Ok(r) = r {
            assert_eq!(r.0, 10u32);
        }
    }
}
