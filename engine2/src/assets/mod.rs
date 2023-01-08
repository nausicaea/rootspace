use std::{borrow::Borrow, collections::BTreeMap, path::Path};

use plyers::load_ply;

pub struct Mesh {}

impl Mesh {
    pub fn with_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let ply = load_ply(path)?;

        Self::with_ply(&ply)
    }

    pub fn with_ply(ply: &plyers::Ply) -> Result<Self, Error> {
        // For the mesh, we need both vertices and indices
        //
        // The following element and property names (ie. element.property) were found in my test
        // files:
        // vertex.x
        // vertex.y
        // vertex.z
        // vertex.red
        // vertex.green
        // vertex.blue
        // vertex.confidence
        // vertex.intensity
        // face.vertex_indices
        // vertex.nx
        // vertex.ny
        // vertex.nz
        // vertex.alpha
        // vertex.texture_u
        // vertex.texture_v

        let mut index: BTreeMap<&str, (usize, BTreeMap<&str, plyers::DataType>)> = BTreeMap::new();
        for element in &ply.descriptor.elements {
            if !index.contains_key(&element.name.borrow()) {
                index.insert(&element.name, (element.count, BTreeMap::default()));
            }

            for property in &element.properties {
                index
                    .get_mut(&element.name.borrow())
                    .map(|(_, e)| e.insert(&property.name, property.data_type));
            }

            for list_property in &element.list_properties {
                index
                    .get_mut(&element.name.borrow())
                    .map(|(_, e)| e.insert(&list_property.name, list_property.data_type));
            }
        }

        dbg!(&index);

        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    PlyError(#[from] plyers::PlyError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playground() {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../plyers/tests/bun_zipper.ply"));

        let _ = Mesh::with_file(&path).unwrap();
    }
}
