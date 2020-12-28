pub mod image;
pub mod mesh;

use anyhow::Result;
use thiserror::Error;
use std::{
    path::{Path, PathBuf},
};
pub use self::image::Image;
pub use self::mesh::Mesh;
use file_manipulation::FileError;

pub trait AssetTrait: Sized {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self>;
}

pub trait AssetMutTrait: AssetTrait {
    fn to_path<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Is not within the asset tree: {}", .0.display())]
    OutOfTree(PathBuf),
    #[error(transparent)]
    FileError(#[from] FileError),
}
