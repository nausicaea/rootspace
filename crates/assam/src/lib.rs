#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

use ecs::resources::Resources;
use std::path::Path;
use std::path::PathBuf;

pub mod resources;

pub use self::resources::{AssetDatabase, AssetDatabaseDeps};

pub trait LoadAsset {
    type Output;

    fn with_path(res: &Resources, path: &Path) -> impl Future<Output = anyhow::Result<Self::Output>> + Send;
}

pub trait SaveAsset {
    fn to_path(&self, path: &Path) -> impl Future<Output = anyhow::Result<()>> + Send;
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
    #[error("The specified file format is not supported for loading assets")]
    UnsupportedFileFormat,
}
