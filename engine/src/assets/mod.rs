pub mod image;
pub mod mesh;
pub mod text;
pub mod vertex;

pub use self::{image::Image, mesh::Mesh, text::Text, vertex::Vertex};
use anyhow::Result;
use thiserror::Error;
use std::{
    path::{Path, PathBuf},
};

pub trait AssetTrait: Sized {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self>;
}

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("{} is not within the asset tree", .0.display())]
    OutOfTree(PathBuf),
    #[error("{} does not exist", .0.display())]
    DoesNotExist(PathBuf),
    #[error("{} is not a file", .0.display())]
    NotAFile(PathBuf),
    #[error("{} ({})", .1, .0.display())]
    Generic(PathBuf, #[source] std::io::Error),
}
