use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorldError {
    #[error(transparent)]
    FileError(#[from] file_manipulation::FileError),
}
