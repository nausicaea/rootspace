use std::{
    convert::TryFrom,
    fs::create_dir_all,
    path::{is_separator, Path, PathBuf},
};

use anyhow::{Context, Error};
use directories::ProjectDirs;
use ecs::Resource;
use file_manipulation::{copy_recursive, DirPathBuf, FilePathBuf, NewOrExFilePathBuf};
use serde::{Deserialize, Serialize};

const APP_QUALIFIER: &str = "net";
const APP_ORGANIZATION: &str = "nausicaea";

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AssetDatabase {
    game_name: Option<String>,
    #[serde(skip)]
    project_dirs: Option<ProjectDirs>,
    #[serde(skip)]
    assets: Option<DirPathBuf>,
    #[serde(skip)]
    states: Option<DirPathBuf>,
}

impl AssetDatabase {
    pub fn initialize(&mut self, name: &str, force: bool) -> Result<(), Error> {
        let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, name.as_ref())
            .context("Could not find the project directories")?;

        let data_local_dir = project_dirs.data_local_dir();
        let asset_database = data_local_dir.join("assets");
        let state_database = data_local_dir.join("states");

        if force || !asset_database.is_dir() {
            let source_assets = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("assets")
                .join(name);
            if source_assets.is_dir() {
                copy_recursive(&source_assets, &asset_database)
                    .context("Could not copy the asset database contents to the new directory")?;
            } else {
                create_dir_all(&asset_database).context("Could not create the asset database")?;
            }
        }
        if !state_database.is_dir() {
            std::fs::create_dir_all(&state_database).context("Could not create the state directory")?;
        }

        self.game_name = Some(name.to_string());
        self.project_dirs = Some(project_dirs);
        self.assets = Some(DirPathBuf::try_from(asset_database)?);
        self.states = Some(DirPathBuf::try_from(state_database)?);

        Ok(())
    }

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
        let states = states.join(name_str);

        let states = match states.extension() {
            None => states.with_extension("json"),
            Some(ext) if ext != "json" => states.with_extension("json"),
            Some(_) => states,
        };

        let state_path = NewOrExFilePathBuf::try_from(&states)?;

        if !state_path.starts_with(&states) {
            return Err(AssetError::OutOfTree(state_path.into()));
        }

        Ok(state_path)
    }

    pub fn find_asset<P: AsRef<Path>>(&self, path: P) -> Result<FilePathBuf, AssetError> {
        let assets = self.assets.as_ref().ok_or(AssetError::AssetTreeNotFound)?;
        let asset_path = FilePathBuf::try_from(assets.join(path))?;

        if !asset_path.as_path().starts_with(&assets) {
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
        let states = states.join(name_str);

        let states = match states.extension() {
            None => states.with_extension("json"),
            Some(ext) if ext != "json" => states.with_extension("json"),
            Some(_) => states,
        };

        let state_path = FilePathBuf::try_from(&states)?;

        if !state_path.as_path().starts_with(&states) {
            return Err(AssetError::OutOfTree(state_path.into()));
        }

        Ok(state_path)
    }

    pub fn all_states(&self) -> Result<Vec<FilePathBuf>, AssetError> {
        let states = self.states.as_ref().ok_or(AssetError::AssetTreeNotFound)?;
        let mut data = Vec::new();
        for dir_entry in std::fs::read_dir(states)? {
            let dir_entry = dir_entry?;
            let entry_path = FilePathBuf::try_from(dir_entry.path())?;
            data.push(entry_path);
        }

        Ok(data)
    }
}

impl Resource for AssetDatabase {}

#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("The asset tree was not found")]
    AssetTreeNotFound,
    #[error("Is not within the asset tree: {}", .0.display())]
    OutOfTree(PathBuf),
    #[error("The asset name {:?} contains path separators", .0)]
    InvalidCharacters(String),
    #[error(transparent)]
    FileError(#[from] file_manipulation::FileError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_database_implements_default() {
        let _: AssetDatabase = Default::default();
    }
}
