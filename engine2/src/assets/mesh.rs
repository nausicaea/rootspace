use std::collections::HashMap;

use plyers::types::{AsSlice, Primitive, PropertyDescriptor};

use crate::resources::graphics::{ids::BufferId, vertex::Vertex, Graphics};

use super::Error;

#[derive(Debug)]
pub struct Mesh {
    pub vertex_buffer: BufferId,
    pub index_buffer: BufferId,
    pub num_indices: u32,
}

impl Mesh {
    pub(crate) fn with_ply(gfx: &mut Graphics, ply: &plyers::Ply) -> Result<Self, Error> {
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
    fn with_ply(ply: &plyers::Ply) -> Result<Self, Error> {
        let (v_e_id, num_vertices) = ply
            .descriptor
            .elements
            .iter()
            .filter(|(_, e)| &e.name == plyers::VERTEX_ELEMENT)
            .map(|(&e_id, e)| (e_id, e.count))
            .next()
            .ok_or(Error::NoVertexElement)?;

        log::trace!("Located vertex element {} with {} vertices", v_e_id, num_vertices);

        let f_e_id = ply
            .descriptor
            .elements
            .iter()
            .filter(|(_, e)| &e.name == plyers::FACE_ELEMENT)
            .map(|(&e_id, _)| e_id)
            .next()
            .ok_or(Error::NoFaceElement)?;

        log::trace!("Located face element {}", f_e_id);

        let v_p_index: HashMap<_, _> = ply.descriptor.elements[&v_e_id]
            .properties
            .iter()
            .filter_map(|(&p_id, p)| match p {
                PropertyDescriptor::Scalar { ref name, .. } => match name.as_ref() {
                    plyers::X_PROPERTY => Some((plyers::X_PROPERTY, p_id)),
                    plyers::Y_PROPERTY => Some((plyers::Y_PROPERTY, p_id)),
                    plyers::Z_PROPERTY => Some((plyers::Z_PROPERTY, p_id)),
                    plyers::NX_PROPERTY => Some((plyers::NX_PROPERTY, p_id)),
                    plyers::NY_PROPERTY => Some((plyers::NY_PROPERTY, p_id)),
                    plyers::NZ_PROPERTY => Some((plyers::NZ_PROPERTY, p_id)),
                    plyers::TEXTURE_U_PROPERTY | plyers::S_PROPERTY | plyers::U_PROPERTY => {
                        Some((plyers::TEXTURE_U_PROPERTY, p_id))
                    }
                    plyers::TEXTURE_V_PROPERTY | plyers::T_PROPERTY | plyers::V_PROPERTY => {
                        Some((plyers::TEXTURE_V_PROPERTY, p_id))
                    }
                    _ => None,
                },
                _ => None,
            })
            .collect();

        log::trace!("Located {} vertex element properties", v_p_index.len());

        let vertex_indices_id = ply.descriptor.elements[&f_e_id]
            .properties
            .iter()
            .find_map(|(&p_id, p)| match p {
                PropertyDescriptor::List { ref name, .. } => match name.as_ref() {
                    plyers::VERTEX_INDICES_LIST_PROPERTY => Some(p_id),
                    _ => None,
                },
                _ => None,
            })
            .ok_or(Error::NoVertexIndices)?;

        log::trace!("Located vertex indices property {}", vertex_indices_id);

        if ply.primitive() != Some(Primitive::Triangles) {
            return Err(Error::NoTriangleFaces);
        }

        let mut vertices = vec![Vertex::default(); num_vertices];

        let vertex_data = &ply.data;
        for i in 0..num_vertices {
            if let Some(p_idx) = v_p_index.get(plyers::X_PROPERTY) {
                vertices[i].position[0] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(plyers::Y_PROPERTY) {
                vertices[i].position[1] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(plyers::Z_PROPERTY) {
                vertices[i].position[2] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(plyers::NX_PROPERTY) {
                vertices[i].normals[0] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(plyers::NY_PROPERTY) {
                vertices[i].normals[1] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(plyers::NZ_PROPERTY) {
                vertices[i].normals[2] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(plyers::TEXTURE_U_PROPERTY) {
                vertices[i].tex_coords[0] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(plyers::TEXTURE_V_PROPERTY) {
                vertices[i].tex_coords[1] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }
        }

        let indices: Vec<u32> = ply.data[&vertex_indices_id]
            .1
            .as_slice()
            .map(|inner: &[u32]| inner)
            .iter()
            .flat_map(|inner| inner.iter())
            .map(|i| *i)
            .collect();

        log::trace!("Loaded {} vertices and {} indices", vertex_data.len(), indices.len());

        Ok(RawMesh { vertices, indices })
    }
}
