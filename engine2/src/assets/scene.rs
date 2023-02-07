use std::collections::BTreeMap;

use ecs::{Entities, Index, Storage};
use rose_tree::Hierarchy;

use crate::{
    components::{
        camera::Camera,
        renderable::{Renderable, RenderableSource},
        transform::Transform,
    },
    resources::asset_database::AssetDatabase,
};

use super::{model::Model, Asset};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Scene {
    entities: Entities,
    hierarchy: Hierarchy<Index>,
    cameras: BTreeMap<Index, Camera>,
    transforms: BTreeMap<Index, Transform>,
    renderables: BTreeMap<Index, RenderableSource>,
}

impl Asset for Scene {
    type Output = ();

    fn with_path(res: &ecs::Resources, path: &std::path::Path) -> Result<Self::Output, super::Error> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let asset = serde_json::from_reader::<_, Scene>(reader)?;

        for e_prev in &asset.entities {
            let e_new = res.borrow_mut::<Entities>().create();

            let camera = asset.cameras[&e_prev.idx()].clone();
            res.borrow_components_mut::<Camera>().insert(e_new, camera);

            let transform = asset.transforms[&e_prev.idx()].clone();
            res.borrow_components_mut::<Transform>().insert(e_new, transform);

            match &asset.renderables[&e_prev.idx()] {
                RenderableSource::Model { group, name } => {
                    let path = res.borrow::<AssetDatabase>().find_asset(group, name)?;
                    let renderable = Renderable(Model::with_path(res, &path)?);
                    res.borrow_components_mut::<Renderable>().insert(e_new, renderable);
                }
            }
        }

        Ok(())
    }
}
