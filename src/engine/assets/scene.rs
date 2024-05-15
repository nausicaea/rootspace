use std::{collections::BTreeMap, path::Path};

use anyhow::Context;

use super::private::PrivLoadAsset;
use crate::{
    ecs::{
        entities::Entities,
        entity::{index::Index, Entity},
        resources::Resources,
        storage::Storage,
    },
    engine::{
        assets::private::PrivSaveAsset,
        components::{
            camera::Camera, info::Info, renderable::Renderable, transform::Transform,
        },
        resources::asset_database::AssetDatabase,
    },
    rose_tree::hierarchy::Hierarchy,
};

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Scene {
    entities: Entities,
    hierarchy: Hierarchy<Index>,
    infos: BTreeMap<Index, Info>,
    cameras: BTreeMap<Index, Camera>,
    transforms: BTreeMap<Index, Transform>,
    renderables: BTreeMap<Index, RenderableSource>,
}

impl Scene {
    pub fn with_resources(res: &Resources) -> Self {
        Scene {
            entities: res.read::<Entities>().clone(),
            hierarchy: res.read::<Hierarchy<Index>>().clone(),
            infos: res
                .read_components::<Info>()
                .indexed_iter()
                .map(|(i, info)| (i, Info::new(&info.name, &info.description)))
                .collect(),
            cameras: res
                .read_components::<Camera>()
                .indexed_iter()
                .map(|(i, c)| (i, c.clone()))
                .collect(),
            transforms: res
                .read_components::<Transform>()
                .indexed_iter()
                .map(|(i, t)| (i, t.clone()))
                .collect(),
            renderables: res
                .read_components::<Renderable>()
                .indexed_iter()
                .map(|(i, r)| {
                    (
                        i,
                        RenderableSource::Reference {
                            group: r.group.clone(),
                            name: r.name.clone(),
                        },
                    )
                })
                .collect(),
        }
    }

    pub fn create_entity(&mut self) -> EntityBuilder {
        EntityBuilder::new(self)
    }

    #[tracing::instrument(skip_all)]
    pub async fn submit<S>(mut self, res: &Resources, group: S, name: S) -> Result<(), anyhow::Error>
    where
        S: AsRef<str> + std::fmt::Debug,
    {
        fn error_recovery<'a, I: IntoIterator<Item = &'a Index>>(res: &Resources, iter: I) {
            let mut entities = res.write::<Entities>();
            let mut hierarchy = res.write::<Hierarchy<Index>>();
            for &i_new in iter {
                entities.destroy(i_new);
                hierarchy.remove(i_new);
            }
        }

        fn register_origin(scene: &mut Scene, group: &str, name: &str) {
            for entity in &scene.entities {
                scene
                    .infos
                    .entry(entity.idx())
                    .and_modify(|info| info.origin = Some((group.to_string(), name.to_string())))
                    .or_insert_with(|| Info {
                        origin: Some((group.to_string(), name.to_string())),
                        ..Default::default()
                    });
            }
        }

        fn load_hierarchy_additive(
            scene: &Scene,
            entities: &mut Entities,
            hierarchy: &mut Hierarchy<Index>,
        ) -> BTreeMap<Index, Index> {
            let mut map: BTreeMap<Index, Index> = BTreeMap::new();

            for i_prev in scene.hierarchy.bfs_iter() {
                if let Some(anc_prev) = scene.hierarchy.ancestors(i_prev).nth(1) {
                    let i_new = entities.create().idx();
                    map.insert(i_prev, i_new);

                    let anc_new = map
                        .get(&anc_prev)
                        .expect("A (parent) scene-based entity has no corresponding world entity");

                    hierarchy.insert_child(anc_new, i_new);
                } else {
                    let i_new = entities.create().idx();
                    map.insert(i_prev, i_new);
                    hierarchy.insert(i_new);
                }
            }

            map
        }

        #[tracing::instrument(skip_all)]
        async fn load_components_additive(
            scene: &Scene,
            map: &BTreeMap<Index, Index>,
            res: &Resources,
        ) -> Result<(), anyhow::Error> {
            for (&i_prev, &i_new) in map {
                if let Some(info) = scene.infos.get(&i_prev).cloned() {
                    res.write_components::<Info>().insert(i_new, info);
                }

                if let Some(camera) = scene.cameras.get(&i_prev).cloned() {
                    res.write_components::<Camera>().insert(i_new, camera);
                }

                if let Some(transform) = scene.transforms.get(&i_prev).cloned() {
                    res.write_components::<Transform>().insert(i_new, transform);
                }

                if let Some(RenderableSource::Reference { group, name }) = scene.renderables.get(&i_prev) {
                    let renderable = Renderable::with_model(res, group, name).await?;
                    res.write_components::<Renderable>().insert(i_new, renderable);
                }
            }

            Ok(())
        }

        register_origin(&mut self, group.as_ref(), name.as_ref());

        let map = load_hierarchy_additive(&self, &mut res.write(), &mut res.write());

        if let Err(e) = load_components_additive(&self, &map, res).await {
            error_recovery(res, map.values());
            return Err(e).context("Adding the scene's components to the existing loaded components");
        }

        Ok(())
    }
}

impl PrivLoadAsset for Scene {
    type Output = ();

    async fn with_path(res: &Resources, path: &Path) -> Result<Self::Output, anyhow::Error> {
        let file = std::fs::File::open(path).with_context(|| format!("Opening the file '{}'", path.display()))?;
        let reader = std::io::BufReader::new(file);

        let scene = ciborium::de::from_reader::<Scene, _>(reader).context("Loading the Scene")?;

        // Since the Info::origin field is not serialized, make sure to assign it to every entity
        // based on the scene asset name.
        let (group, name) = res.read::<AssetDatabase>().find_asset_name(path)?;

        scene.submit(res, group, name).await
    }
}

impl PrivSaveAsset for Scene {
    async fn to_path(&self, path: &Path) -> Result<(), anyhow::Error> {
        let file = std::fs::File::create(path).with_context(|| format!("Creating the file '{}'", path.display()))?;
        let writer = std::io::BufWriter::new(file);

        ciborium::ser::into_writer(self, writer).context("Serializing the Scene")?;

        Ok(())
    }
}

pub struct EntityBuilder<'a> {
    scene: &'a mut Scene,
    parent: Option<Index>,
    info: Option<Info>,
    camera: Option<Camera>,
    transform: Option<Transform>,
    renderable: Option<RenderableSource>,
}

impl<'a> EntityBuilder<'a> {
    fn new(scene: &'a mut Scene) -> Self {
        EntityBuilder {
            scene,
            parent: None,
            info: None,
            camera: None,
            transform: None,
            renderable: None,
        }
    }

    pub fn with_parent<I: Into<Index>>(mut self, parent: I) -> Self {
        self.parent = Some(parent.into());
        self
    }

    pub fn with_info(mut self, info: Info) -> Self {
        self.info = Some(info);
        self
    }

    pub fn with_camera(mut self, cam: Camera) -> Self {
        self.camera = Some(cam);
        self
    }

    pub fn with_transform(mut self, trf: Transform) -> Self {
        self.transform = Some(trf);
        self
    }

    pub fn with_renderable(mut self, rdb: RenderableSource) -> Self {
        self.renderable = Some(rdb);
        self
    }

    pub fn submit(self) -> Entity {
        let e = self.scene.entities.create();
        let i = e.idx();

        if let Some(parent) = self.parent {
            self.scene.hierarchy.insert_child(parent, i);
        } else {
            self.scene.hierarchy.insert(i);
        }

        if let Some(info) = self.info {
            self.scene.infos.insert(i, info);
        }

        if let Some(camera) = self.camera {
            self.scene.cameras.insert(i, camera);
        }

        if let Some(transform) = self.transform {
            self.scene.transforms.insert(i, transform);
        }

        if let Some(renderable) = self.renderable {
            self.scene.renderables.insert(i, renderable);
        }

        e
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum RenderableSource {
    Reference { group: String, name: String },
}
