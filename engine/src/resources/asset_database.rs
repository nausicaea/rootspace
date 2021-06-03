use std::path::Path;

use file_manipulation::{DirPathBuf, FilePathBuf};

use crate::assets::AssetError;
use ecs::{Resource, SerializationName};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AssetDatabase {
    asset_tree: Option<DirPathBuf>,
}

impl Resource for AssetDatabase {}

impl SerializationName for AssetDatabase {}

impl AssetDatabase {
    pub fn find_asset<P: AsRef<Path>>(&self, path: P) -> Result<FilePathBuf, AssetError> {
        let asset_tree = self.asset_tree.as_ref().ok_or(AssetError::TreeUnknown)?;

        let asset_path = FilePathBuf::try_from(asset_tree.join(path))?;

        if !asset_path.path().starts_with(&asset_tree) {
            return Err(AssetError::OutOfTree(asset_path.into()));
        }

        Ok(asset_path)
    }
}
