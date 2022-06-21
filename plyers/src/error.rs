use thiserror::Error as ThisError;
use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("parsing is complete")]
    ParsingComplete,
    #[error("Found a list property with no parent element at offset {:#x}", .0)]
    UnexpectedListProperty(usize),
    #[error("Found a property with no parent element at offset {:#x}", .0)]
    UnexpectedProperty(usize),
    #[error("Missing format type")]
    MissingFormatType,
    #[error("Missing format version")]
    MissingFormatVersion,
    #[error("Unexpected byte {:#x} at offset {:#x}", .0, .1)]
    UnexpectedByte(u8, usize),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("The specified file ended unexpectedly at offset {:#x}", .0)]
    UnexpectedEndOfFile(usize),
    #[error("The specified file is not a PLY file")]
    NotAPlyFile,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

