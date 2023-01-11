use std::{
    convert::TryFrom,
    fs::create_dir_all,
    path::{is_separator, Path, PathBuf},
};

use anyhow::{Context, Error};
use directories::ProjectDirs;
use ecs::{with_dependencies::WithDependencies, Resource};
use file_manipulation::{copy_recursive, DirPathBuf, FilePathBuf, NewOrExFilePathBuf};

const APP_QUALIFIER: &str = "net";
const APP_ORGANIZATION: &str = "nausicaea";

pub trait AssetDatabaseDeps {
    fn name(&self) -> &str;
    fn force_init(&self) -> bool;
}

#[derive(Clone, Debug)]
pub struct AssetDatabase {
    game_name: String,
    project_dirs: ProjectDirs,
    assets: DirPathBuf,
    states: DirPathBuf,
}

impl AssetDatabase {
    pub fn asset_directory(&self) -> &Path {
        &self.assets
    }

    pub fn state_directory(&self) -> &Path {
        &self.states
    }

    pub fn create_state_path<S: AsRef<str>>(&self, name: S) -> Result<NewOrExFilePathBuf, AssetError> {
        let name_str = name.as_ref();
        if name_str.chars().any(|c| is_separator(c)) {
            return Err(AssetError::InvalidCharacters(name_str.to_string()));
        }

        let states = self.states.join(name_str);

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
        let asset_path = FilePathBuf::try_from(self.assets.join(path))?;

        if !asset_path.as_path().starts_with(&self.assets) {
            return Err(AssetError::OutOfTree(asset_path.into()));
        }

        Ok(asset_path)
    }

    pub fn find_state<S: AsRef<str>>(&self, name: S) -> Result<FilePathBuf, AssetError> {
        let name_str = name.as_ref();
        if name_str.chars().any(|c| is_separator(c)) {
            return Err(AssetError::InvalidCharacters(name_str.to_string()));
        }

        let states = self.states.join(name_str);

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
        let mut data = Vec::new();
        for dir_entry in std::fs::read_dir(&self.states)? {
            let dir_entry = dir_entry?;
            let entry_path = FilePathBuf::try_from(dir_entry.path())?;
            data.push(entry_path);
        }

        Ok(data)
    }
}

impl Resource for AssetDatabase {}

impl<D: AssetDatabaseDeps> WithDependencies<D> for AssetDatabase {
    fn with_deps(deps: &D) -> Result<Self, Error> {
        let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, deps.name())
            .context("Could not find the project directories")?;

        let data_local_dir = project_dirs.data_local_dir();
        let asset_database = data_local_dir.join("assets");
        let state_database = data_local_dir.join("states");

        if deps.force_init() || !asset_database.is_dir() {
            let source_assets = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("assets")
                .join(deps.name());
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

        Ok(AssetDatabase {
            game_name: deps.name().to_string(),
            project_dirs,
            assets: DirPathBuf::try_from(asset_database)?,
            states: DirPathBuf::try_from(state_database)?,
        })
    }
}

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
    use ecs::{End, Reg, ResourceRegistry, World};

    use super::*;

    struct TDeps<'a> {
        name: &'a str,
        force_init: bool,
    }

    impl Default for TDeps<'static> {
        fn default() -> Self {
            TDeps {
                name: "test",
                force_init: false,
            }
        }
    }

    impl<'a> AssetDatabaseDeps for TDeps<'a> {
        fn name(&self) -> &str {
            self.name
        }

        fn force_init(&self) -> bool {
            self.force_init
        }
    }

    #[test]
    fn asset_database_reg_macro() {
        type _RR = Reg![AssetDatabase];
    }

    #[test]
    fn asset_database_resource_registry() {
        let deps = TDeps::default();
        let _rr = ResourceRegistry::push(End, AssetDatabase::with_deps(&deps).unwrap());
    }

    #[test]
    fn asset_database_world() {
        let deps = TDeps::default();
        let _w = World::with_dependencies::<Reg![AssetDatabase], Reg![], Reg![], Reg![], _>(&deps).unwrap();
    }
}
