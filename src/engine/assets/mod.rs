use std::path::PathBuf;

use crate::{file_manipulation, plyers};

pub mod cpu_material;
pub mod cpu_mesh;
pub mod cpu_model;
pub mod cpu_texture;
pub mod gpu_material;
pub mod gpu_mesh;
pub mod gpu_model;
pub mod gpu_texture;
pub mod scene;

pub trait LoadAsset: private::PrivLoadAsset {}

impl<T: private::PrivLoadAsset + Sized> LoadAsset for T {}

pub trait SaveAsset: private::PrivSaveAsset {}

impl<T: private::PrivSaveAsset> SaveAsset for T {}

pub mod private {
    use std::path::Path;

    use crate::ecs::resources::Resources;

    pub trait PrivLoadAsset: Sized {
        type Output;

        fn with_path(
            res: &Resources,
            path: &Path,
        ) -> impl std::future::Future<Output = Result<Self::Output, anyhow::Error>> + Send;
    }

    pub trait PrivSaveAsset {
        fn to_path(&self, path: &Path) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;
    }
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
