use std::path::{Path, PathBuf};

use ecs::Resources;

pub mod material;
pub mod mesh;
pub mod model;
pub mod scene;
pub mod texture;

pub trait Asset: Sized {
    fn with_path(res: &Resources, path: &Path) -> Result<Self, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not deserialization function is configured for the asset")]
    NoDeserializationConfigured,
    #[error("The asset tree was not found")]
    AssetTreeNotFound,
    #[error("Is not within the asset tree: {}", .0.display())]
    OutOfTree(PathBuf),
    #[error("The asset group or name contain disallowed characters: group='{:?}', name='{:?}'", .0, .1)]
    InvalidCharacters(String, String),
    #[error(transparent)]
    FileError(#[from] file_manipulation::FileError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    PlyError(#[from] plyers::PlyError),
    #[error("The specified file format is not supported for loading assets")]
    UnsupportedFileFormat,
    #[error("No element named 'vertex' was found")]
    NoVertexElement,
    #[error("No element named 'face' was found")]
    NoFaceElement,
    #[error("The element named 'face' contains no property 'vertex_indices' with triangle indices")]
    NoVertexIndices,
    #[error("The mesh does not use triangles as face primitive")]
    NoTriangleFaces,
}
