use crate::engine::resources::graphics::vertex::Vertex;
use crate::plyers::types::{
    AsSlice, Ply, Primitive, PropertyDescriptor, FACE_ELEMENT, NX_PROPERTY, NY_PROPERTY, NZ_PROPERTY, S_PROPERTY,
    TEXTURE_U_PROPERTY, TEXTURE_V_PROPERTY, T_PROPERTY, U_PROPERTY, VERTEX_ELEMENT, VERTEX_INDICES_LIST_PROPERTY,
    V_PROPERTY, X_PROPERTY, Y_PROPERTY, Z_PROPERTY,
};
use std::collections::HashMap;

use super::Error;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RawMesh {
    pub label: Option<String>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl RawMesh {
    pub(crate) fn with_ply(ply: &Ply) -> Result<Self, Error> {
        let label = ply
            .descriptor
            .obj_info
            .iter()
            .filter(|obj_info| obj_info.0.starts_with("label="))
            .map(|obj_info| obj_info.0.replace("label=", ""))
            .next();

        let (v_e_id, num_vertices) = ply
            .descriptor
            .elements
            .iter()
            .filter(|(_, e)| e.name == VERTEX_ELEMENT)
            .map(|(&e_id, e)| (e_id, e.count))
            .next()
            .ok_or(Error::NoVertexElement)?;

        log::trace!("Located vertex element {} with {} vertices", v_e_id, num_vertices);

        let f_e_id = ply
            .descriptor
            .elements
            .iter()
            .filter(|(_, e)| e.name == FACE_ELEMENT)
            .map(|(&e_id, _)| e_id)
            .next()
            .ok_or(Error::NoFaceElement)?;

        log::trace!("Located face element {}", f_e_id);

        let v_p_index: HashMap<_, _> = ply.descriptor.elements[&v_e_id]
            .properties
            .iter()
            .filter_map(|(&p_id, p)| match p {
                PropertyDescriptor::Scalar { ref name, .. } => match name.as_ref() {
                    X_PROPERTY => Some((X_PROPERTY, p_id)),
                    Y_PROPERTY => Some((Y_PROPERTY, p_id)),
                    Z_PROPERTY => Some((Z_PROPERTY, p_id)),
                    NX_PROPERTY => Some((NX_PROPERTY, p_id)),
                    NY_PROPERTY => Some((NY_PROPERTY, p_id)),
                    NZ_PROPERTY => Some((NZ_PROPERTY, p_id)),
                    TEXTURE_U_PROPERTY | S_PROPERTY | U_PROPERTY => Some((TEXTURE_U_PROPERTY, p_id)),
                    TEXTURE_V_PROPERTY | T_PROPERTY | V_PROPERTY => Some((TEXTURE_V_PROPERTY, p_id)),
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
                    VERTEX_INDICES_LIST_PROPERTY => Some(p_id),
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
        for (i, vertex) in vertices.iter_mut().enumerate() {
            if let Some(p_idx) = v_p_index.get(X_PROPERTY) {
                vertex.position[0] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(Y_PROPERTY) {
                vertex.position[1] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(Z_PROPERTY) {
                vertex.position[2] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(NX_PROPERTY) {
                vertex.normals[0] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(NY_PROPERTY) {
                vertex.normals[1] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(NZ_PROPERTY) {
                vertex.normals[2] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(TEXTURE_U_PROPERTY) {
                vertex.tex_coords[0] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }

            if let Some(p_idx) = v_p_index.get(TEXTURE_V_PROPERTY) {
                vertex.tex_coords[1] = vertex_data[p_idx].1.as_slice().unwrap()[i];
            }
        }

        let indices: Vec<u32> = ply.data[&vertex_indices_id]
            .1
            .as_slice()
            .map(|inner: &[u32]| inner)
            .iter()
            .flat_map(|inner| inner.iter()).copied()
            .collect();

        log::trace!("Loaded {} vertices and {} indices", vertex_data.len(), indices.len());

        Ok(RawMesh {
            label,
            vertices,
            indices,
        })
    }
}
