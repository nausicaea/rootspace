use std::{
    io::Read,
    num::ParseIntError,
    str::FromStr,
    task::Poll,
};

use super::{utilities::read_byte, Error};

pub fn parse_unsigned_from_ascii<T, R>(file: &mut R, offset: &mut usize) -> Result<T, Error>
where
    T: FromStr<Err = ParseIntError>,
    R: Read,
{
    let mut integer = String::new();
    parse_individual(file, offset, |b, o| match b {
        0x09..=0x0d | 0x20 => Poll::Ready(Ok(())),
        b @ 0x30..=0x39 => {
            integer.push(char::from(b));
            Poll::Pending
        }
        b => Poll::Ready(Err(Error::UnexpectedByte(b, o))),
    })?;
    let integer: T = integer.parse()?;
    Ok(integer)
}

pub fn parse_word_from_ascii<R>(file: &mut R, offset: &mut usize) -> Result<String, Error>
where
    R: Read,
{
    let mut word = String::new();
    parse_individual(file, offset, |b, _| match b {
        0x09..=0x0d | 0x20 => Poll::Ready(Ok(())),
        b => {
            word.push(char::from(b));
            Poll::Pending
        }
    })?;

    Ok(word)
}

pub fn parse_string_from_ascii<R>(file: &mut R, offset: &mut usize) -> Result<String, Error>
where
    R: Read,
{
    let mut string = String::new();
    parse_individual(file, offset, |b, _| match b {
        0x0a => Poll::Ready(Ok(())),
        b => {
            string.push(char::from(b));
            Poll::Pending
        }
    })?;

    Ok(string)
}

pub fn parse_from_lut<T, F, R>(file: &mut R, offset: &mut usize, lut: &[&[u8]], mut func: F) -> Result<T, Error>
where
    R: Read,
    F: FnMut(usize) -> Result<T, Error>,
{
    let mut indices: Vec<usize> = vec![0usize; lut.len()];
    loop {
        let byte = read_byte(file, offset)?;

        for k in 0..lut.len() {
            let index = indices[k];
            let expected_bytes = lut[k];
            if byte == expected_bytes[index] {
                indices[k] += 1;
            }

            if indices[k] >= lut[k].len() {
                eprintln!("Found: {:?}", std::str::from_utf8(lut[k]).unwrap());
                return func(k);
            }
        }
    }
}

pub fn parse_whitespace<R>(file: &mut R, offset: &mut usize) -> Result<(), Error>
where
    R: Read,
{
    let byte = read_byte(file, offset)?;
    match byte {
        0x09..=0x0d | 0x20 => Ok(()),
        b => Err(Error::UnexpectedByte(b, *offset)),
    }
}

pub fn parse_newline<R>(file: &mut R, offset: &mut usize) -> Result<(), Error>
where
    R: Read,
{
    let byte = read_byte(file, offset)?;
    match byte {
        0x0a => Ok(()),
        b => Err(Error::UnexpectedByte(b, *offset)),
    }
}

pub fn parse_individual<T, F, R>(file: &mut R, offset: &mut usize, mut func: F) -> Result<T, Error>
where
    R: Read,
    F: FnMut(u8, usize) -> Poll<Result<T, Error>>,
{
    loop {
        let byte = read_byte(file, offset)?;

        match func(byte, *offset) {
            Poll::Pending => (),
            Poll::Ready(r) => return r,
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn parse_string_from_ascii_terminates_at_the_first_newline(source in r"[[:ascii:]]+\n") {
            let expected: String = source.chars().take_while(|c| c != &'\n').collect();
            let mut stream = source.as_bytes();
            let mut offset = 0usize;

            let r = parse_string_from_ascii(&mut stream, &mut offset);

            prop_assert!(r.is_ok(), "{}", r.unwrap_err());
            prop_assert_eq!(r.unwrap(), expected);
        }

        #[test]
        fn parse_word_from_ascii_terminates_at_the_first_whitespace(source in r"[[:ascii:]]+[ \x09-\x0d]") {
            let expected: String = source.chars().take_while(|c| ![' ', '\t', '\r', '\n', '\x0b', '\x0c'].iter().any(|p| p == c)).collect();
            let mut stream = source.as_bytes();
            let mut offset = 0usize;

            let r = parse_word_from_ascii(&mut stream, &mut offset);

            prop_assert!(r.is_ok(), "{}", r.unwrap_err());
            prop_assert_eq!(r.unwrap(), expected);
        }

        #[test]
        fn parse_unsigned_from_ascii_parses_unsigned_integer_numbers(expected: u64, nl in r"[ \x09-\x0d]") {
            let source = format!("{}{}", expected, nl);
            let mut stream = source.as_bytes();
            let mut offset = 0usize;

            let r = parse_unsigned_from_ascii::<u64, _>(&mut stream, &mut offset);

            prop_assert!(r.is_ok(), "{}", r.unwrap_err());
            prop_assert_eq!(r.unwrap(), expected);
        }
    }
}
