use thiserror::Error as ThisError;
use std::{num::{ParseFloatError, ParseIntError}, string::FromUtf8Error as SFU8E};

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("the parser has been run to failure or completion already")]
    ParserExhausted,
    #[error("Found a list property with no parent element")]
    UnexpectedListProperty,
    #[error("Found a property with no parent element")]
    UnexpectedProperty,
    #[error("Missing format type")]
    MissingFormatType,
    #[error("Missing format version")]
    MissingFormatVersion,
    #[error("Unexpected byte {:#x}", .0)]
    UnexpectedByte(u8),
    #[error(transparent)]
    FromUtf8Error(#[from] SFU8E),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("The specified file ended unexpectedly")]
    UnexpectedEndOfFile,
    #[error("The specified file is not a PLY file")]
    NotAPlyFile,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

