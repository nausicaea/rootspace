use std::collections::BTreeMap;

use ecs::{Entities, Index};
use rose_tree::Hierarchy;

use crate::components::{camera::Camera, renderable::RenderableSource, transform::Transform};

use super::Asset;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Scene {
    entities: Entities,
    hierarchy: Hierarchy<Index>,
    cameras: BTreeMap<Index, Camera>,
    transforms: BTreeMap<Index, Transform>,
    renderables: BTreeMap<Index, RenderableSource>,
}

impl Asset for Scene {
    fn with_path(_res: &ecs::Resources, path: &std::path::Path) -> Result<Self, super::Error> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let asset = serde_json::from_reader::<_, Scene>(reader)?;

        Ok(asset)
    }
}
