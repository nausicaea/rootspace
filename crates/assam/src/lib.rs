use std::path::PathBuf;
use std::path::Path;
use ecs::resources::Resources;

pub mod asset_database;

pub use self::asset_database::{AssetDatabase, AssetDatabaseDeps};

pub trait LoadAsset {
    type Output;

    fn with_path(
        res: &Resources,
        path: &Path,
    ) -> impl std::future::Future<Output = Result<Self::Output, anyhow::Error>> + Send;
}

pub trait SaveAsset {
    fn to_path(&self, path: &Path) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The asset tree was not found")]
    AssetTreeNotFound,
    #[error("Is not within the asset tree: {}", .0.display())]
    OutOfTree(PathBuf),
    #[error("Could not determine the asset group from path: {}", .0.display())]
    NoAssetGroup(PathBuf),
    #[error("Could not determine the asset name from path: {}", .0.display())]
    NoAssetName(PathBuf),
    #[error("The asset group or name contain disallowed characters: group='{:?}', name='{:?}'", .0, .1)]
    InvalidCharacters(String, String),
    #[error(transparent)]
    File(#[from] file_manipulation::FileError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Ply(#[from] plyers::PlyError),
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
