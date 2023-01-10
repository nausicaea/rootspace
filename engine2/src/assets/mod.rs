use std::{borrow::Borrow, collections::BTreeMap, path::Path};

use plyers::load_ply;

#[derive(Debug)]
pub struct Vertex {
    position: [f32; 3],
    normals: [f32; 3],
    tex_coords: [f32; 2],
}

#[derive(Debug)]
pub struct Mesh {}

impl Mesh {
    const VERTEX_KWD: &'static str = "vertex";
    const X_KWD: &'static str = "x";
    const Y_KWD: &'static str = "y";
    const Z_KWD: &'static str = "z";

    pub fn with_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let ply = load_ply(path)?;

        Self::with_ply(&ply)
    }

    pub fn with_ply(ply: &plyers::Ply<f32>) -> Result<Self, Error> {
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

        let mut index: BTreeMap<&str, (usize, usize, BTreeMap<&str, (usize, plyers::DataType)>)> = BTreeMap::new();
        for (ei, element) in ply.descriptor.elements.iter().enumerate() {
            if !index.contains_key(&element.name.borrow()) {
                index.insert(&element.name, (ei, element.count, BTreeMap::default()));
            }

            for (pi, property) in element.properties.iter().enumerate() {
                index
                    .get_mut(&element.name.borrow())
                    .map(|(_, _, e)| e.insert(&property.name, (pi, property.data_type)));
            }

            for (pi, list_property) in element.list_properties.iter().enumerate() {
                index
                    .get_mut(&element.name.borrow())
                    .map(|(_, _, e)| e.insert(&list_property.name, (pi, list_property.data_type)));
            }
        }

        if !index.contains_key(Mesh::VERTEX_KWD) {
            return Err(Error::NoVertexElement);
        }

        if !index[Mesh::VERTEX_KWD].2.contains_key(Mesh::X_KWD) {
            return Err(Error::NoVertexPropertyX);
        }

        if !index[Mesh::VERTEX_KWD].2.contains_key(Mesh::Y_KWD) {
            return Err(Error::NoVertexPropertyY);
        }

        if !index[Mesh::VERTEX_KWD].2.contains_key(Mesh::Z_KWD) {
            return Err(Error::NoVertexPropertyZ);
        }

        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    PlyError(#[from] plyers::PlyError),
    #[error("No element named 'vertex' was found")]
    NoVertexElement,
    #[error("No property named 'x' found in element 'vertex'")]
    NoVertexPropertyX,
    #[error("No property named 'y' found in element 'vertex'")]
    NoVertexPropertyY,
    #[error("No property named 'z' found in element 'vertex'")]
    NoVertexPropertyZ,
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
