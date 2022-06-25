use std::io::{Read, Seek, SeekFrom};

use super::Parser;
use crate::read_byte;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct OneOf<'a> {
    patterns: &'a [&'a [u8]],
}

impl<'a> Parser for OneOf<'a> {
    type Item = &'a [u8];

    fn parse<R>(self, r: &mut R) -> Result<Self::Item, Error> where Self:Sized, R: Read + Seek {
        let mut indices: Vec<usize> = vec![0; self.patterns.len()];
        let skip_back = loop {
            let (byte, _) = read_byte(r)?;

            // Search for the byte in the available patterns
            let mut skip_back = true;
            for c in 0..self.patterns.len() {
                // If the index of the current pattern is OOB, skip this pattern
                if indices[c] >= self.patterns[c].len() {
                    continue;
                }

                // Increase the index of the current pattern if the byte matches, or mark the
                // pattern as non-matching
                if byte == self.patterns[c][indices[c]] {
                    indices[c] += 1;
                    skip_back = false;
                } else {
                    indices[c] = usize::MAX;
                    skip_back = true;
                }
            }

            if indices.iter().enumerate().all(|(c, i)| i >= &self.patterns[c].len()) {
                break skip_back;
            }
        };

        // Move one byte back, since parsing had to go one over
        if skip_back {
            r.seek(SeekFrom::Current(-1))?;
        }

        // Abort if none of the patterns match
        if indices.iter().all(|i| i == &usize::MAX) {
            return Err(Error::NoMatchingPatterns);
        }

        // If at least one of the patterns matches, find the longest
        let matching_indices: Vec<(usize, usize)> = indices.iter()
            .enumerate()
            .filter(|(_, i)| i < &&usize::MAX)
            .map(|(c, i)| (c, *i))
            .collect();

        let target = matching_indices.iter()
            .max_by_key(|(_, i)| *i)
            .map(|(c, _)| *c)
            .unwrap_or_else(|| unreachable!());

        Ok(self.patterns[target])
    }
}

pub fn one_of<'a>(engrams: &'a [&'a [u8]]) -> OneOf<'a> {
    if engrams.is_empty() || engrams.iter().any(|e| e.is_empty()) {
        panic!("one_of cannot match when no patterns are given or when one pattern is empty");
    }

    OneOf {
        patterns: engrams,
    }
}

#[cfg(test)]
mod tests {
    use crate::{to_reader, types::DATA_TYPES};
    use super::*;

    #[test]
    fn one_of_succeeds_on_the_first_engram_that_matches() {
        let mut stream = to_reader("hello");

        let r = one_of(&[b"bye bye", b"hello"]).parse(&mut stream);

        match r {
            Ok(b"hello") => (),
            other => panic!("Expected Ok(b\"hello\"), got: {:?}", other),
        }
    }

    #[test]
    fn one_of_fails_on_the_first_byte_that_does_not_match_any_engram() {
        let mut stream = to_reader("bald eagle");

        let r = one_of(&[b"bye bye", b"hello"]).parse(&mut stream);

        match r {
            Err(Error::NoMatchingPatterns) => (),
            other => panic!("Expected Error::NoMatchingPatterns, got: {:?}", other),
        }
    }

    #[test]
    fn one_of_matches_longer_patterns_first() {
        let mut stream = to_reader("Hello, Samantha");

        let r = one_of(&[b"Hello", b"Hello, Samantha"]).parse(&mut stream);

        match r {
            Ok(b"Hello, Samantha") => (),
            other => panic!("Expected Ok(b\"Hello, Samantha\"), got: {:?}", other),
        }
    }

    #[test]
    fn one_of_correctly_parses_data_type() {
        let mut stream = to_reader("int32");

        let r = one_of(DATA_TYPES).parse(&mut stream);

        match r {
            Ok(b"int32") => (),
            other => panic!("Expected Ok(b\"int32\"), got: {:?}", other),
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
