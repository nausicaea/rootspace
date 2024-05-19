use std::collections::HashMap;

use anyhow::Context;

use super::{private::PrivLoadAsset, Error};
use crate::resources::graphics::vertex::Vertex;
use plyers::{
    load_ply,
    types::{
        AsSlice, Ply, Primitive, PropertyDescriptor, FACE_ELEMENT, NX_PROPERTY, NY_PROPERTY, NZ_PROPERTY,
        S_PROPERTY, TEXTURE_U_PROPERTY, TEXTURE_V_PROPERTY, T_PROPERTY, U_PROPERTY, VERTEX_ELEMENT,
        VERTEX_INDICES_LIST_PROPERTY, V_PROPERTY, X_PROPERTY, Y_PROPERTY, Z_PROPERTY,
    },
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CpuMesh {
    pub label: Option<String>,
    pub texture_names: Vec<String>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl PrivLoadAsset for CpuMesh {
    type Output = Self;

    async fn with_path(
        _res: &ecs::resources::Resources,
        path: &std::path::Path,
    ) -> Result<Self::Output, anyhow::Error> {
        let label = path.file_stem().and_then(|n| n.to_str()).map(|n| n.to_owned());

        if let Some("ply") = path.extension().and_then(|ext| ext.to_str()) {
            let ply =
                load_ply(path).with_context(|| format!("Loading a Stanford Ply file from '{}'", path.display()))?;
            let mesh = Self::with_ply(&ply, label)?;
            Ok(mesh)
        } else {
            Err(Error::UnsupportedFileFormat.into())
        }
    }
}

impl CpuMesh {
    fn with_ply(ply: &Ply, label: Option<String>) -> Result<Self, Error> {
        let (v_e_id, num_vertices) = ply
            .descriptor
            .elements
            .iter()
            .filter(|(_, e)| e.name == VERTEX_ELEMENT)
            .map(|(&e_id, e)| (e_id, e.count))
            .next()
            .ok_or(Error::NoVertexElement)?;

        tracing::trace!("Located vertex element {} with {} vertices", v_e_id, num_vertices);

        let f_e_id = ply
            .descriptor
            .elements
            .iter()
            .filter(|(_, e)| e.name == FACE_ELEMENT)
            .map(|(&e_id, _)| e_id)
            .next()
            .ok_or(Error::NoFaceElement)?;

        tracing::trace!("Located face element {}", f_e_id);

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

        tracing::trace!("Located {} vertex element properties", v_p_index.len());

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

        tracing::trace!("Located vertex indices property {}", vertex_indices_id);

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
            .flat_map(|inner| inner.iter())
            .copied()
            .collect();

        tracing::trace!("Loaded {} vertices and {} indices", vertex_data.len(), indices.len());

        let texture_names: Vec<_> = Self::find_texture_names(ply).map(|n| n.to_owned()).collect();

        tracing::trace!("Located the following texture names: {}", texture_names.join(", "));

        Ok(CpuMesh {
            label,
            texture_names,
            vertices,
            indices,
        })
    }

    fn find_texture_names(ply: &Ply) -> impl Iterator<Item = &str> {
        ply.descriptor
            .comments
            .iter()
            .chain(ply.descriptor.elements.values().flat_map(|e| e.comments.iter()))
            .chain(
                ply.descriptor
                    .elements
                    .values()
                    .flat_map(|e| e.properties.values().flat_map(|p| p.comments())),
            )
            .map(AsRef::<str>::as_ref)
            .filter(|c| c.starts_with("TextureFile"))
            .map(|c| c.trim_start_matches("TextureFile "))
            .chain(
                ply.descriptor
                    .obj_info
                    .iter()
                    .chain(ply.descriptor.elements.values().flat_map(|e| e.obj_info.iter()))
                    .chain(
                        ply.descriptor
                            .elements
                            .values()
                            .flat_map(|e| e.properties.values().flat_map(|p| p.obj_info())),
                    )
                    .map(AsRef::<str>::as_ref)
                    .filter(|c| c.starts_with("texture"))
                    .map(|c| c.trim_start_matches("texture ")),
            )
    }
}
