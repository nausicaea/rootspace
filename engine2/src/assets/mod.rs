use std::{collections::HashMap, path::Path};

use plyers::{
    load_ply,
    types::{Primitive, PropertyDescriptor},
};

use crate::resources::graphics::{ids::BufferId, vertex::Vertex, Graphics};

#[derive(Debug)]
pub struct Model {
    mesh: Mesh,
}

#[derive(Debug)]
pub struct Mesh {
    pub(crate) vertex_buffer: BufferId,
    pub(crate) index_buffer: BufferId,
    pub(crate) num_indices: u32,
}

impl Mesh {
    pub fn with_file<P: AsRef<Path>>(gfx: &mut Graphics, path: P) -> Result<Self, Error> {
        let ply = load_ply(path)?;
        Self::with_ply(gfx, &ply)
    }

    pub fn with_ply(gfx: &mut Graphics, ply: &plyers::Ply<f32, u32>) -> Result<Self, Error> {
        let rm = RawMesh::with_ply(ply)?;

        let vertex_buffer = gfx.create_buffer(None, wgpu::BufferUsages::VERTEX, &rm.vertices);
        let index_buffer = gfx.create_buffer(None, wgpu::BufferUsages::INDEX, &rm.indices);

        Ok(Mesh {
            vertex_buffer,
            index_buffer,
            num_indices: rm.indices.len() as u32,
        })
    }
}

#[derive(Debug)]
struct RawMesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl RawMesh {
    fn with_ply(ply: &plyers::Ply<f32, u32>) -> Result<Self, Error> {
        if !ply.data.contains_key(plyers::VERTEX_ELEMENT) {
            return Err(Error::NoVertexElement);
        }

        let prp: HashMap<_, _> = ply
            .descriptor
            .elements
            .iter()
            .filter(|e| &e.name == plyers::VERTEX_ELEMENT)
            .flat_map(|e| e.properties.iter().enumerate())
            .filter_map(|(i, p)| match p {
                PropertyDescriptor::Scalar { ref name, .. } => match name.as_ref() {
                    plyers::X_PROPERTY => Some((plyers::X_PROPERTY, i)),
                    plyers::Y_PROPERTY => Some((plyers::Y_PROPERTY, i)),
                    plyers::Z_PROPERTY => Some((plyers::Z_PROPERTY, i)),
                    plyers::NX_PROPERTY => Some((plyers::NX_PROPERTY, i)),
                    plyers::NY_PROPERTY => Some((plyers::NY_PROPERTY, i)),
                    plyers::NZ_PROPERTY => Some((plyers::NZ_PROPERTY, i)),
                    plyers::TEXTURE_U_PROPERTY => Some((plyers::TEXTURE_U_PROPERTY, i)),
                    plyers::TEXTURE_V_PROPERTY => Some((plyers::TEXTURE_V_PROPERTY, i)),
                    _ => None,
                },
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
        let vertex_data = &ply.data[plyers::VERTEX_ELEMENT];
        for i in 0..num_vertices {
            if let Some(p_idx) = prp.get(plyers::X_PROPERTY) {
                vertices[i].position[0] = vertex_data[i * num_properties + p_idx]
                    .map_left(|l| Ok(l))
                    .left_or(Err(Error::NotAScalar(plyers::X_PROPERTY)))?;
            }

            if let Some(p_idx) = prp.get(plyers::Y_PROPERTY) {
                vertices[i].position[1] = vertex_data[i * num_properties + p_idx]
                    .map_left(|l| Ok(l))
                    .left_or(Err(Error::NotAScalar(plyers::Y_PROPERTY)))?;
            }

            if let Some(p_idx) = prp.get(plyers::Z_PROPERTY) {
                vertices[i].position[2] = vertex_data[i * num_properties + p_idx]
                    .map_left(|l| Ok(l))
                    .left_or(Err(Error::NotAScalar(plyers::Z_PROPERTY)))?;
            }

            if let Some(p_idx) = prp.get(plyers::NX_PROPERTY) {
                vertices[i].normals[0] = vertex_data[i * num_properties + p_idx]
                    .map_left(|l| Ok(l))
                    .left_or(Err(Error::NotAScalar(plyers::NX_PROPERTY)))?;
            }

            if let Some(p_idx) = prp.get(plyers::NY_PROPERTY) {
                vertices[i].normals[1] = vertex_data[i * num_properties + p_idx]
                    .map_left(|l| Ok(l))
                    .left_or(Err(Error::NotAScalar(plyers::NY_PROPERTY)))?;
            }

            if let Some(p_idx) = prp.get(plyers::NZ_PROPERTY) {
                vertices[i].normals[2] = vertex_data[i * num_properties + p_idx]
                    .map_left(|l| Ok(l))
                    .left_or(Err(Error::NotAScalar(plyers::NZ_PROPERTY)))?;
            }

            if let Some(p_idx) = prp.get(plyers::TEXTURE_U_PROPERTY) {
                vertices[i].tex_coords[0] = vertex_data[i * num_properties + p_idx]
                    .map_left(|l| Ok(l))
                    .left_or(Err(Error::NotAScalar(plyers::TEXTURE_U_PROPERTY)))?;
            }

            if let Some(p_idx) = prp.get(plyers::TEXTURE_V_PROPERTY) {
                vertices[i].tex_coords[1] = vertex_data[i * num_properties + p_idx]
                    .map_left(|l| Ok(l))
                    .left_or(Err(Error::NotAScalar(plyers::TEXTURE_V_PROPERTY)))?;
            }
        }

        if !ply.data.contains_key(plyers::FACE_ELEMENT) {
            return Err(Error::NoTriangleIndices);
        }

        let lprp: HashMap<_, _> = ply
            .descriptor
            .elements
            .iter()
            .filter(|e| &e.name == plyers::FACE_ELEMENT)
            .flat_map(|e| e.properties.iter().enumerate())
            .filter_map(|(i, p)| match p {
                PropertyDescriptor::List { ref name, .. } => match name.as_ref() {
                    plyers::VERTEX_INDICES_LIST_PROPERTY => Some((plyers::VERTEX_INDICES_LIST_PROPERTY, i)),
                    _ => None,
                },
                _ => None,
            })
            .collect();

        if !lprp.contains_key(plyers::VERTEX_INDICES_LIST_PROPERTY) {
            return Err(Error::NoTriangleIndices);
        }

        if ply.face_type() != Some(Primitive::Triangles) {
            return Err(Error::NoTriangleIndices);
        }

        let indices = &ply.data[plyers::FACE_ELEMENT][lprp[plyers::VERTEX_INDICES_LIST_PROPERTY]]
            .as_ref()
            .right_or_else(|_| Vec::new())
            .iter()
            .map(|i| *i)
            .collect::<Vec<_>>();

        Ok(RawMesh { vertices, indices })
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
    #[error("The property named '{}' is not a scalar", .0)]
    NotAScalar(&'static str),
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
