use std::path::{Path, is_separator};

use file_manipulation::{DirPathBuf, FilePathBuf, NewOrExFilePathBuf, ValidatedPath};

use crate::assets::AssetError;
use ecs::{Resource, SerializationName};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AssetDatabase {
    assets: Option<DirPathBuf>,
    states: Option<DirPathBuf>,
}

impl Resource for AssetDatabase {}

impl SerializationName for AssetDatabase {}

impl AssetDatabase {
    pub fn asset_directory(&self) -> Option<&Path> {
        self.assets.as_ref().map(|p| p.as_path())
    }

    pub fn state_directory(&self) -> Option<&Path> {
        self.states.as_ref().map(|p| p.as_path())
    }

    pub fn create_state_path<S: AsRef<str>>(&self, name: S) -> Result<NewOrExFilePathBuf, AssetError> {
        let name_str = name.as_ref();
        if name_str.chars().any(|c| is_separator(c)) {
            return Err(AssetError::InvalidCharacters(name_str.to_string()));
        }

        let states = self.states.as_ref().ok_or(AssetError::AssetTreeNotFound)?;
        let state_path = NewOrExFilePathBuf::try_from(states.join(name_str))?;

        if !state_path.starts_with(&states) {
            return Err(AssetError::OutOfTree(state_path.into()));
        }

        Ok(state_path)
    }

    pub fn find_asset<P: AsRef<Path>>(&self, path: P) -> Result<FilePathBuf, AssetError> {
        let assets = self.assets.as_ref().ok_or(AssetError::AssetTreeNotFound)?;
        let asset_path = FilePathBuf::try_from(assets.join(path))?;

        if !asset_path.path().starts_with(&assets) {
            return Err(AssetError::OutOfTree(asset_path.into()));
        }

        Ok(asset_path)
    }

    pub fn find_state<S: AsRef<str>>(&self, name: S) -> Result<FilePathBuf, AssetError> {
        let name_str = name.as_ref();
        if name_str.chars().any(|c| is_separator(c)) {
            return Err(AssetError::InvalidCharacters(name_str.to_string()));
        }

        let states = self.states.as_ref().ok_or(AssetError::AssetTreeNotFound)?;
        let state_path = FilePathBuf::try_from(states.join(name_str))?;

        if !state_path.path().starts_with(&states) {
            return Err(AssetError::OutOfTree(state_path.into()));
        }

        Ok(state_path)
    }
}
