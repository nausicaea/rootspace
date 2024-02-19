use downcast_rs::__std::path::PathBuf;
use crate::file_manipulation::FileError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorldError {
    #[error(transparent)]
    FileError(#[from] FileError),
    #[error("{}: {}", .1, .0.display())]
    IoError(PathBuf, #[source] std::io::Error),
    #[error("{}: {}", .1, .0.display())]
    JsonError(PathBuf, #[source] serde_json::Error),
}
