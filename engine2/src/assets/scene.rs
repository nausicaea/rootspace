use std::collections::BTreeMap;

use ecs::{Entities, Index, Resources, Storage};
use rose_tree::Hierarchy;

use crate::{
    components::{
        camera::Camera,
        renderable::{Renderable, RenderableSource},
        transform::Transform,
    },
    resources::asset_database::AssetDatabase,
};

use super::{model::Model, private::LoadAsset, Error};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Scene {
    entities: Entities,
    hierarchy: Hierarchy<Index>,
    cameras: BTreeMap<Index, Camera>,
    transforms: BTreeMap<Index, Transform>,
    renderables: BTreeMap<Index, RenderableSource>,
}

impl Scene {
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

                let anc_new = entities.create().idx();
                map.insert(anc_prev, anc_new);

                hierarchy.insert_child(anc_new, i_new);
            } else {
                let i_new = entities.create().idx();
                map.insert(i_prev, i_new);
                hierarchy.insert(i_new);
            }
        }

        map
    }

    fn load_components_additive(&self, map: &BTreeMap<Index, Index>, res: &Resources) -> Result<(), Error> {
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
                        let path = res.borrow::<AssetDatabase>().find_asset(group, name)?;
                        let renderable = Renderable(Model::with_path(res, &path)?);
                        res.borrow_components_mut::<Renderable>().insert(i_new, renderable);
                    }
                }
            }
        }

        Ok(())
    }
}

impl LoadAsset for Scene {
    type Output = ();

    fn with_path(res: &ecs::Resources, path: &std::path::Path) -> Result<Self::Output, Error> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let scene = serde_json::from_reader::<_, Scene>(reader)?;

        let map = scene.load_hierarchy_additive(&mut res.borrow_mut(), &mut res.borrow_mut());
        scene.load_components_additive(&map, res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_playground() {
        let mut entities = Entities::default();
        let e1 = entities.create();
        let e2 = entities.create();
        let e3 = entities.create();
        let e4 = entities.create();
        let e5 = entities.create();

        let mut hierarchy: Hierarchy<Index> = Hierarchy::default();
        hierarchy.insert(e1);
        hierarchy.insert(e2);
        hierarchy.insert_child(e1, e3);
        hierarchy.insert_child(e2, e4);
        hierarchy.insert_child(e2, e5);

        let scene = Scene {
            entities,
            hierarchy,
            cameras: BTreeMap::default(),
            transforms: BTreeMap::default(),
            renderables: BTreeMap::default(),
        };

        let mut new_hierarchy: Hierarchy<Index> = Hierarchy::default();
        for i in scene.hierarchy.bfs_iter() {
            if let Some(anc) = scene.hierarchy.ancestors(i).nth(1) {
                new_hierarchy.insert_child(anc, i);
            } else {
                new_hierarchy.insert(i);
            }
        }

        assert_eq!(&scene.hierarchy, &new_hierarchy);
    }
}
