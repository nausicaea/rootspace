use std::path::{Path, PathBuf};

use crate::ecs::resource::Resource;
use crate::ecs::resources::Resources;
use crate::ecs::with_dependencies::WithDependencies;
use crate::engine::assets::{Error, LoadAsset, SaveAsset};
use crate::file_manipulation::copy_recursive;
use anyhow::Context;
use directories::ProjectDirs;
use tokio::fs::{create_dir_all, remove_dir_all};

const APP_QUALIFIER: &str = "net";
const APP_ORGANIZATION: &str = "nausicaea";

lazy_static::lazy_static! {
    static ref WITHIN_REPO_ASSETS: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    static ref GROUP_AND_NAME_ALLOWLIST: regex::Regex = regex::RegexBuilder::new("^[-._0-9a-zA-Z]+$")
        .multi_line(true)
        .build()
        .unwrap();
}

pub trait AssetDatabaseDeps {
    /// Specifies the name of the game (must be a valid directory name)
    fn name(&self) -> &str;
    /// Overwrite the existing asset cache
    fn force_init(&self) -> bool;
    /// Load and save assets from within the code repository (this only makes sense in development)
    fn within_repo(&self) -> bool;
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AssetDatabase {
    game_name: String,
    project_dirs: ProjectDirs,
    assets: PathBuf,
}

impl AssetDatabase {
    pub async fn load_asset<A, S>(&self, res: &Resources, group: S, name: S) -> Result<A::Output, anyhow::Error>
    where
        A: LoadAsset,
        S: AsRef<str>,
    {
        let path = self.find_asset(&group, &name).with_context(|| {
            format!(
                "Finding the path of asset '{}' in group '{}'",
                name.as_ref(),
                group.as_ref()
            )
        })?;
        let asset = A::with_path(res, &path).await.with_context(|| {
            format!(
                "Loading a {} asset from path '{}'",
                std::any::type_name::<A>(),
                path.display()
            )
        })?;

        Ok(asset)
    }

    pub async fn save_asset<A, S>(&self, asset: &A, group: S, name: S) -> Result<(), anyhow::Error>
    where
        A: SaveAsset,
        S: AsRef<str>,
    {
        let path = self.find_asset(&group, &name).with_context(|| {
            format!(
                "Finding the path of asset '{}' in group '{}'",
                name.as_ref(),
                group.as_ref()
            )
        })?;

        if let Some(parent) = path.parent() {
            create_dir_all(parent)
                .await
                .with_context(|| format!("Creating the parent directories of path '{}'", path.display()))?;
        }

        asset.to_path(&path).await.with_context(|| {
            format!(
                "Saving a {} asset to path '{}'",
                std::any::type_name::<A>(),
                path.display()
            )
        })?;

        Ok(())
    }

    pub fn find_asset<S: AsRef<str>>(&self, group: S, name: S) -> Result<PathBuf, Error> {
        let group = group.as_ref();
        let name = name.as_ref();

        if !(GROUP_AND_NAME_ALLOWLIST.is_match(group) && GROUP_AND_NAME_ALLOWLIST.is_match(name)) {
            return Err(Error::InvalidCharacters(group.to_string(), name.to_string()));
        }

        let asset_path = self.assets.join(group).join(name);

        if !asset_path.starts_with(&self.assets) {
            return Err(Error::OutOfTree(asset_path));
        }

        Ok(asset_path)
    }
}

impl Resource for AssetDatabase {}

impl<D: AssetDatabaseDeps> WithDependencies<D> for AssetDatabase {
    async fn with_deps(deps: &D) -> Result<Self, anyhow::Error> {
        let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, deps.name()).with_context(|| {
            format!(
                "Finding the project directories of triplet ({}, {}, {})",
                APP_QUALIFIER,
                APP_ORGANIZATION,
                deps.name()
            )
        })?;

        let assets = if deps.within_repo() {
            WITHIN_REPO_ASSETS.join(deps.name())
        } else {
            let data_local_dir = project_dirs.data_local_dir();
            data_local_dir.join("assets")
        };

        if (deps.force_init() && !deps.within_repo()) || !assets.is_dir() {
            remove_dir_all(&assets)
                .await
                .with_context(|| format!("Removing all contents of the path '{}'", assets.display()))?;

            let source_assets = WITHIN_REPO_ASSETS.join(deps.name());
            if source_assets.is_dir() {
                copy_recursive(&source_assets, &assets).await.with_context(|| {
                    format!(
                        "Copying the asset database contents from '{}' to '{}'",
                        source_assets.display(),
                        assets.display()
                    )
                })?;
            } else {
                create_dir_all(&assets)
                    .await
                    .with_context(|| format!("Creating the asset database directory at '{}'", assets.display()))?;
            }
        }

        Ok(AssetDatabase {
            game_name: deps.name().to_string(),
            project_dirs,
            assets,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::registry::{End, ResourceRegistry};
    use crate::ecs::world::World;
    use crate::Reg;

    struct TDeps<'a> {
        name: &'a str,
        force_init: bool,
        within_repo: bool,
    }

    impl Default for TDeps<'static> {
        fn default() -> Self {
            TDeps {
                name: "test",
                force_init: false,
                within_repo: true,
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

        fn within_repo(&self) -> bool {
            self.within_repo
        }
    }

    #[test]
    fn asset_database_reg_macro() {
        type _RR = Reg![AssetDatabase];
    }

    #[tokio::test]
    async fn asset_database_resource_registry() {
        let deps = TDeps::default();
        let _rr = ResourceRegistry::push(End, AssetDatabase::with_deps(&deps).await.unwrap());
    }

    #[tokio::test]
    async fn asset_database_world() {
        let deps = TDeps::default();
        let _w = World::with_dependencies::<Reg![AssetDatabase], Reg![], Reg![], (), Reg![], _>(&deps)
            .await
            .unwrap();
    }
}
