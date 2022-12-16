use std::{io::Error as IoError, path::PathBuf};

use file_manipulation::FileError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("The asset tree was not found")]
    AssetTreeNotFound,
    #[error("Is not within the asset tree: {}", .0.display())]
    OutOfTree(PathBuf),
    #[error("The asset name {:?} contains path separators", .0)]
    InvalidCharacters(String),
    #[error(transparent)]
    FileError(#[from] FileError),
    #[error(transparent)]
    IoError(#[from] IoError),
}
