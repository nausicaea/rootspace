use std::{
    convert::TryFrom,
    fs::create_dir_all,
    path::{is_separator, Path, PathBuf},
};

use anyhow::{Context, Error};
use directories::ProjectDirs;
use ecs::{Resource, SerializationName};
use file_manipulation::{copy_recursive, DirPathBuf, FilePathBuf, NewOrExFilePathBuf, ValidatedPath};
use serde::{Deserialize, Serialize};

use crate::{assets::AssetError, APP_ORGANIZATION, APP_QUALIFIER};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AssetDatabaseSerDe {
    game_name: String,
}

impl From<AssetDatabase> for AssetDatabaseSerDe {
    fn from(value: AssetDatabase) -> Self {
        AssetDatabaseSerDe {
            game_name: value.game_name.expect("No game name was defined previously"),
        }
    }
}

impl TryFrom<AssetDatabaseSerDe> for AssetDatabase {
    type Error = anyhow::Error;

    fn try_from(value: AssetDatabaseSerDe) -> Result<Self, Self::Error> {
        let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, &value.game_name)
            .context("Could not find the project directories")?;

        let data_local_dir = project_dirs.data_local_dir();
        let asset_database = data_local_dir.join("assets");
        let state_database = data_local_dir.join("states");

        Ok(AssetDatabase {
            game_name: Some(value.game_name),
            project_dirs: Some(project_dirs),
            assets: Some(DirPathBuf::try_from(asset_database)?),
            states: Some(DirPathBuf::try_from(state_database)?),
        })
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(into = "AssetDatabaseSerDe", try_from = "AssetDatabaseSerDe")]
pub struct AssetDatabase {
    game_name: Option<String>,
    project_dirs: Option<ProjectDirs>,
    assets: Option<DirPathBuf>,
    states: Option<DirPathBuf>,
}

impl Resource for AssetDatabase {}

impl SerializationName for AssetDatabase {}

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
