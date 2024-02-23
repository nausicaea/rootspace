use crate::ecs::entities::Entities;
use crate::ecs::entity::index::Index;
use crate::ecs::entity::Entity;
use crate::ecs::resources::Resources;
use crate::ecs::storage::Storage;
use crate::engine::assets::private::PrivSaveAsset;
use crate::engine::components::camera::Camera;
use crate::engine::components::renderable::{Renderable, RenderableSource};
use crate::engine::components::transform::Transform;
use crate::engine::resources::asset_database::AssetDatabase;
use crate::rose_tree::hierarchy::Hierarchy;
use anyhow::Context;
use std::collections::BTreeMap;
use std::path::Path;

use super::{model::Model, private::PrivLoadAsset};

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Scene {
    entities: Entities,
    hierarchy: Hierarchy<Index>,
    cameras: BTreeMap<Index, Camera>,
    transforms: BTreeMap<Index, Transform>,
    renderables: BTreeMap<Index, RenderableSource>,
}

impl Scene {
    pub fn create_entity(&mut self) -> EntityBuilder {
        EntityBuilder::new(self)
    }

    pub fn hierarchy(&self) -> &Hierarchy<Index> {
        &self.hierarchy
    }

    pub fn load_additive(self, res: &Resources) -> Result<(), anyhow::Error> {
        let map = self.load_hierarchy_additive(&mut res.borrow_mut(), &mut res.borrow_mut());

        if let Err(e) = self.load_components_additive(&map, res) {
            Self::unload_entities(res, map.values());
            return Err(e).context("trying to add the scene's components to the existing loaded components");
        }

        Ok(())
    }

    fn unload_entities<'a, I: IntoIterator<Item = &'a Index>>(res: &Resources, iter: I) {
        let mut entities = res.borrow_mut::<Entities>();
        let mut hierarchy = res.borrow_mut::<Hierarchy<Index>>();
        for &i_new in iter {
            entities.destroy(i_new);
            hierarchy.remove(i_new);
        }
    }

    fn load_hierarchy_additive(
        &self,
        entities: &mut Entities,
        hierarchy: &mut Hierarchy<Index>,
    ) -> BTreeMap<Index, Index> {
        let mut map: BTreeMap<Index, Index> = BTreeMap::new();

        for i_prev in self.hierarchy.bfs_iter() {
            if let Some(anc_prev) = self.hierarchy.ancestors(i_prev).nth(1) {
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

    fn load_components_additive(&self, map: &BTreeMap<Index, Index>, res: &Resources) -> Result<(), anyhow::Error> {
        for (&i_prev, &i_new) in map {
            if let Some(camera) = self.cameras.get(&i_prev).cloned() {
                res.borrow_components_mut::<Camera>().insert(i_new, camera);
            }

            if let Some(transform) = self.transforms.get(&i_prev).cloned() {
                res.borrow_components_mut::<Transform>().insert(i_new, transform);
            }

            if let Some(renderable_source) = self.renderables.get(&i_prev) {
                match renderable_source {
                    RenderableSource::Model { group, name } => {
                        let path = res.borrow::<AssetDatabase>().find_asset(group, name).with_context(|| {
                            format!("trying to find the path of asset '{}' in group '{}'", name, group)
                        })?;
                        let model = Model::with_path(res, &path).with_context(|| {
                            format!(
                                "trying to load {} from path '{}'",
                                std::any::type_name::<Model>(),
                                path.display()
                            )
                        })?;
                        let renderable = Renderable(model);
                        res.borrow_components_mut::<Renderable>().insert(i_new, renderable);
                    }
                }
            }
        }

        Ok(())
    }
}

impl PrivLoadAsset for Scene {
    type Output = ();

    fn with_path(res: &Resources, path: &Path) -> Result<Self::Output, anyhow::Error> {
        let file =
            std::fs::File::open(path).with_context(|| format!("trying to open the file '{}'", path.display()))?;
        let reader = std::io::BufReader::new(file);

        let scene = ciborium::de::from_reader::<Scene, _>(reader)
            .with_context(|| format!("trying to deserialize the {}", std::any::type_name::<Scene>()))?;

        scene.load_additive(res)
    }
}

impl PrivSaveAsset for Scene {
    fn to_path(&self, path: &Path) -> Result<(), anyhow::Error> {
        let file =
            std::fs::File::create(path).with_context(|| format!("trying to create the file '{}'", path.display()))?;
        let writer = std::io::BufWriter::new(file);

        ciborium::ser::into_writer(self, writer)
            .with_context(|| format!("trying to deserialize the {}", std::any::type_name::<Scene>()))?;

        Ok(())
    }
}

pub struct EntityBuilder<'a> {
    scene: &'a mut Scene,
    parent: Option<Index>,
    camera: Option<Camera>,
    transform: Option<Transform>,
    renderable: Option<RenderableSource>,
}

impl<'a> EntityBuilder<'a> {
    fn new(scene: &'a mut Scene) -> Self {
        EntityBuilder {
            scene,
            parent: None,
            camera: None,
            transform: None,
            renderable: None,
        }
    }

    pub fn with_parent<I: Into<Index>>(mut self, parent: I) -> Self {
        self.parent = Some(parent.into());
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
