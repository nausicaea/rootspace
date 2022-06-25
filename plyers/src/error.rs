use thiserror::Error as ThisError;
use std::{num::{ParseFloatError, ParseIntError}, string::FromUtf8Error as SFU8E};
use std::io::SeekFrom;
use crate::types::FromBytesError as FBE;

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
    #[error("Unexpected byte {:#x} at position {:?}", .0, .1)]
    UnexpectedByte(u8, SeekFrom),
    #[error("The specified file ended unexpectedly at position {:?}", .0)]
    UnexpectedEndOfFile(SeekFrom),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] SFU8E),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    FromBytesError(#[from] FBE),
}

