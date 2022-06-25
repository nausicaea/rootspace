use super::Parser;
use crate::utilities::read_byte;
use crate::error::Error;

#[derive(Debug)]
pub struct OneOf<'a> {
    patterns: &'a [&'a [u8]],
    exhausted: bool,
}

impl<'a> Parser for OneOf<'a> {
    type Item = &'a [u8];

    fn parse<R>(&mut self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: std::io::Read {
        if !self.exhausted {
            let mut indices: Vec<usize> = vec![0; self.patterns.len()];
            loop {
                let byte = read_byte(r)?;

                // Search for the byte in the available patterns
                for c in 0..self.patterns.len() {
                    if indices[c] >= self.patterns[c].len() {
                        continue;
                    }

                    if byte == self.patterns[c][indices[c]] {
                        indices[c] += 1;
                    } else {
                        indices[c] = std::usize::MAX;
                    }
                }

                // Abort if none of the patterns match
                if indices.iter().all(|i| i == &std::usize::MAX) {
                    self.exhausted = true;
                    return Err(Error::UnexpectedByte(byte));
                }

                // If at least one of the patterns matches, find the longest
                let matching_indices: Vec<(usize, usize)> = indices.iter()
                    .enumerate()
                    .filter(|(_, i)| i < &&std::usize::MAX)
                    .map(|(c, i)| (c, *i))
                    .collect();
                if matching_indices.iter().any(|(c, i)| i >= &self.patterns[*c].len()) {
                    let target = matching_indices.iter()
                        .max_by_key(|(_, i)| *i)
                        .map(|(c, _)| *c)
                        .ok_or(Error::UnexpectedByte(byte))?;

                    self.exhausted = true;
                    return Ok(self.patterns[target])
                }
            }
        }

        Err(Error::ParserExhausted)
    }
}

pub fn one_of<'a>(engrams: &'a [&'a [u8]]) -> OneOf<'a> {
    if engrams.is_empty() || engrams.iter().any(|e| e.is_empty()) {
        panic!("one_of cannot match when no patterns are given or when one pattern is empty");
    }

    OneOf {
        patterns: engrams,
        exhausted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_of_succeeds_on_the_first_engram_that_matches() {
        let source = "hello";
        let expected: Vec<u8> = source.as_bytes().iter().copied().collect();
        let mut stream = source.as_bytes();

        let mut p = one_of(&[b"bye bye", b"hello"]);

        let r = p.parse(&mut stream);

        match r {
            Ok(product) if product == expected => (),
            other => panic!("Expected Ok(b\"hello\" as Vec<u8>), got: {:?}", other),
        }
    }

    #[test]
    fn one_of_fails_on_the_first_byte_that_does_not_match_any_engram() {
        let source = "bald eagle";
        let mut stream = source.as_bytes();

        let mut p = one_of(&[b"bye bye", b"hello"]);

        let r = p.parse(&mut stream);

        match r {
            Err(Error::UnexpectedByte(b'a')) => (),
            other => panic!("Expected Error::UnexpectedByte(b'a'), got: {:?}", other),
        }
    }

    #[test]
    fn one_of_fails_when_called_after_completion() {
        let source = "hello";
        let mut stream = source.as_bytes();

        let mut p = one_of(&[b"bye bye", b"hello"]);

        let _ = p.parse(&mut stream);

        let r = p.parse(&mut stream);

        match r {
            Err(Error::ParserExhausted) => (),
            other => panic!("Expected Err(Error::ParserExhausted), got: {:?}", other),
        }
    }

    #[test]
    fn one_of_matches_longer_patterns_first() {
        let source = "Hello, Samantha";
        let expected: Vec<u8> = source.as_bytes().iter().cloned().collect();
        let mut stream = source.as_bytes();

        let mut p = one_of(&[b"Hello", b"Hello, Samantha"]);
        let r = p.parse(&mut stream);

        match r {
            Ok(product) if product == expected => (),
            other => panic!("Expected Ok(b\"Hello, Samantha\" as Vec<u8>), got: {:?}", other),
        }
    }

    #[test]
    #[should_panic]
    fn one_of_panics_when_no_patterns_are_given() {
        let _ = one_of(&[]);
    }

    #[test]
    #[should_panic]
    fn one_of_panics_when_an_empty_pattern_is_given() {
        let _ = one_of(&[&[], b"Hello"]);
    }
}
