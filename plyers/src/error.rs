use std::{
    num::{ParseFloatError, ParseIntError},
    string::FromUtf8Error as SFU8E,
};
use std::ops::Range;

use crate::types::{FORMAT_TYPES, COUNT_TYPES, DATA_TYPES};

use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Found a list property with no parent element")]
    UnexpectedListProperty,
    #[error("Found a property with no parent element")]
    UnexpectedProperty,
    #[error("Missing format type")]
    MissingFormatType,
    #[error("Missing format version")]
    MissingFormatVersion,
    #[error("None of the supplied patterns matched")]
    NoMatchingPatterns,
    #[error("Unexpected byte {:#x} at position {:#x}", .0, .1)]
    UnexpectedByte(u8, u64),
    #[error("The specified file ended unexpectedly at position {:#x}", .0)]
    UnexpectedEndOfFile(u64),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] SFU8E),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    FromBytesError(#[from] OneOfManyError),
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("received byte sequence {:?}, but expected one of {:?}", .received, .expected)]
pub struct OneOfManyError {
    received: Vec<u8>,
    expected: &'static [&'static [u8]],
}

impl OneOfManyError {
    pub fn new(received: Vec<u8>, expected: &'static [&'static [u8]]) -> Self {
        OneOfManyError { received, expected }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlyError {
    #[error("received byte sequence {received:?}, but expected one of {:?}", FORMAT_TYPES)]
    FormatType { received: Vec<u8> },
    #[error("received byte sequence {received:?}, but expected one of {:?}", COUNT_TYPES)]
    CountType { received: Vec<u8> },
    #[error("received byte sequence {received:?}, but expected one of {:?}", DATA_TYPES)]
    DataType { received: Vec<u8> },
}
