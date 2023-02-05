use crate::resources::{asset_database::AssetDatabase, graphics::Graphics};

pub mod material;
pub mod mesh;
pub mod model;
pub mod texture;

pub trait Asset: Sized {
    type Error;

    fn with_file<S: AsRef<str>>(
        adb: &AssetDatabase,
        gfx: &mut Graphics,
        group: S,
        name: S,
    ) -> Result<Self, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
    #[error(transparent)]
    AssetError(#[from] crate::resources::asset_database::AssetError),
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
