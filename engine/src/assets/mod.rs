pub mod image;
pub mod mesh;

use std::{
    io::Error as IoError,
    path::{Path, PathBuf},
};

use anyhow::Result;
use file_manipulation::FileError;
use thiserror::Error;

pub use self::{image::Image, mesh::Mesh};

pub trait AssetTrait: Sized {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self>;
}

pub trait AssetMutTrait: AssetTrait {
    fn to_path<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}

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
