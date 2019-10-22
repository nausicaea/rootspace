pub mod image;
pub mod mesh;
pub mod text;
pub mod vertex;

pub use self::{image::Image, mesh::Mesh, text::Text, vertex::Vertex};
use failure::{Error, Fail};
use std::{
    io::Error as IoError,
    path::{Path, PathBuf},
};

pub trait AssetTrait: Sized {
    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
}

#[derive(Debug, Fail)]
pub enum AssetError {
    #[fail(display = "{:?} is not within the asset tree", _0)]
    OutOfTree(PathBuf),
    #[fail(display = "{:?} does not exist", _0)]
    DoesNotExist(PathBuf),
    #[fail(display = "{:?} is not a file", _0)]
    NotAFile(PathBuf),
    #[fail(display = "{} ({:?})", _1, _0)]
    Generic(PathBuf, #[cause] IoError),
}
