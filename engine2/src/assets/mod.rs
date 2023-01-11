use std::{collections::HashMap, path::Path};

use plyers::{load_ply, types::Primitive};

#[derive(Debug, Default, Clone)]
pub struct Vertex {
    position: [f32; 3],
    normals: [f32; 3],
    tex_coords: [f32; 2],
}

#[derive(Debug)]
pub struct RawMesh {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl RawMesh {
    pub fn with_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let ply = load_ply(path)?;
        Self::with_ply(&ply)
    }

    pub fn with_ply(ply: &plyers::Ply<f32, u16>) -> Result<Self, Error> {
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

        if !ply.data.contains_key(plyers::VERTEX_ELEMENT) {
            return Err(Error::NoVertexElement);
        }

        let prp: HashMap<_, _> = ply
            .descriptor
            .elements
            .iter()
            .filter(|e| &e.name == plyers::VERTEX_ELEMENT)
            .flat_map(|e| e.properties.iter().enumerate())
            .filter_map(|(i, p)| match p.name.as_ref() {
                plyers::X_PROPERTY => Some((plyers::X_PROPERTY, i)),
                plyers::Y_PROPERTY => Some((plyers::Y_PROPERTY, i)),
                plyers::Z_PROPERTY => Some((plyers::Z_PROPERTY, i)),
                plyers::NX_PROPERTY => Some((plyers::NX_PROPERTY, i)),
                plyers::NY_PROPERTY => Some((plyers::NY_PROPERTY, i)),
                plyers::NZ_PROPERTY => Some((plyers::NZ_PROPERTY, i)),
                plyers::TEXTURE_U_PROPERTY => Some((plyers::TEXTURE_U_PROPERTY, i)),
                plyers::TEXTURE_V_PROPERTY => Some((plyers::TEXTURE_V_PROPERTY, i)),
                _ => None,
            })
            .collect();

        let (num_vertices, num_properties) = ply
            .descriptor
            .elements
            .iter()
            .filter(|e| &e.name == plyers::VERTEX_ELEMENT)
            .map(|e| (e.count, e.properties.len()))
            .next()
            .unwrap_or((0, 0));
        let mut vertices = vec![Vertex::default(); num_vertices];
        let vertex_data = &ply.data[plyers::VERTEX_ELEMENT].0;
        for i in 0..num_vertices {
            if let Some(p_idx) = prp.get(plyers::X_PROPERTY) {
                vertices[i].position[0] = vertex_data[i * num_properties + p_idx]
            }

            if let Some(p_idx) = prp.get(plyers::Y_PROPERTY) {
                vertices[i].position[1] = vertex_data[i * num_properties + p_idx]
            }

            if let Some(p_idx) = prp.get(plyers::Z_PROPERTY) {
                vertices[i].position[2] = vertex_data[i * num_properties + p_idx]
            }

            if let Some(p_idx) = prp.get(plyers::NX_PROPERTY) {
                vertices[i].normals[0] = vertex_data[i * num_properties + p_idx]
            }

            if let Some(p_idx) = prp.get(plyers::NY_PROPERTY) {
                vertices[i].normals[1] = vertex_data[i * num_properties + p_idx]
            }

            if let Some(p_idx) = prp.get(plyers::NZ_PROPERTY) {
                vertices[i].normals[2] = vertex_data[i * num_properties + p_idx]
            }

            if let Some(p_idx) = prp.get(plyers::TEXTURE_U_PROPERTY) {
                vertices[i].tex_coords[0] = vertex_data[i * num_properties + p_idx]
            }

            if let Some(p_idx) = prp.get(plyers::TEXTURE_V_PROPERTY) {
                vertices[i].tex_coords[1] = vertex_data[i * num_properties + p_idx]
            }
        }

        dbg!(&vertices);

        if !ply.data.contains_key(plyers::FACE_ELEMENT) {
            return Err(Error::NoTriangleIndices);
        }

        if ply.face_type() != Some(Primitive::Triangles) {
            return Err(Error::NoTriangleIndices);
        }

        let indices = ply.data[plyers::FACE_ELEMENT]
            .1
            .iter()
            .flatten()
            .map(|i| *i)
            .collect::<Vec<_>>();

        let m = RawMesh { vertices, indices };
        dbg!(&m);
        Ok(m)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    PlyError(#[from] plyers::PlyError),
    #[error("No element named 'vertex' was found")]
    NoVertexElement,
    #[error("The element named 'face' contains no property 'vertex_indices' with triangle indices")]
    NoTriangleIndices,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playground() {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../plyers/tests/cube.ply"));
        let ply = load_ply(path).unwrap();

        let _ = RawMesh::with_ply(&ply).unwrap();
    }
}
