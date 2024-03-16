use thiserror::Error;

use crate::file_manipulation::FileError;

#[derive(Debug, Error)]
pub enum WorldError {
    #[error(transparent)]
    FileError(#[from] FileError),
}
