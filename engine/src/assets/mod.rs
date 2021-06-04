pub mod image;
pub mod mesh;

pub use self::{image::Image, mesh::Mesh};
use anyhow::Result;
use file_manipulation::FileError;
use std::path::{Path, PathBuf};
use thiserror::Error;

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
}
