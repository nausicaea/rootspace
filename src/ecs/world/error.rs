use crate::file_manipulation::FileError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorldError {
    #[error(transparent)]
    FileError(#[from] FileError),
}
