use std::{collections::HashMap, path::Path};

use plyers::{
    load_ply,
    types::{AsSlice, Primitive, PropertyDescriptor},
};

use crate::resources::graphics::{
    ids::{BindGroupId, BufferId, SamplerId, TextureId, TextureViewId},
    vertex::Vertex,
    Graphics,
};

#[derive(Debug)]
pub struct Model {
    pub mesh: Mesh,
}

impl Model {
    pub fn with_file<P: AsRef<Path>>(gfx: &mut Graphics, path: P) -> Result<Self, Error> {
        match path.as_ref().extension().and_then(|ext| ext.to_str()) {
            Some("ply") => {
                let ply = load_ply(path)?;
                Self::with_ply(gfx, &ply)
            }
            _ => Err(Error::UnsupportedFileFormat),
        }
    }

    pub fn with_ply(gfx: &mut Graphics, ply: &plyers::Ply) -> Result<Self, Error> {
        Ok(Model {
            mesh: Mesh::with_ply(gfx, ply)?,
        })
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertex_buffer: BufferId,
    pub index_buffer: BufferId,
    pub num_indices: u32,
}

impl Mesh {
    pub fn with_ply(gfx: &mut Graphics, ply: &plyers::Ply) -> Result<Self, Error> {
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

        let f_e_id = ply
            .descriptor
            .elements
            .iter()
            .filter(|(_, e)| &e.name == plyers::FACE_ELEMENT)
            .map(|(&e_id, _)| e_id)
            .next()
            .ok_or(Error::NoFaceElement)?;

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

        Ok(RawMesh { vertices, indices })
    }
}

#[derive(Debug)]
pub struct Material {
    texture: Texture,
    bind_group: BindGroupId,
}

#[derive(Debug)]
pub struct Texture {
    texture: TextureId,
    view: TextureViewId,
    sampler: SamplerId,
}

impl Texture {
    pub fn with_file<P: AsRef<Path>>(gfx: &mut Graphics, fmt: image::ImageFormat, path: P) -> Result<Self, Error> {
        let f = std::fs::File::open(path)?;
        let img = image::load(std::io::BufReader::new(f), fmt)?;

        let texture = gfx.create_texture().with_image(img).submit(None);

        let view = gfx.create_texture_view(texture);

        let sampler = gfx.create_sampler().submit();

        Ok(Texture { texture, view, sampler })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
    #[error("The specified file format is not supported for loading assets")]
    UnsupportedFileFormat,
    #[error(transparent)]
    PlyError(#[from] plyers::PlyError),
    #[error("No element named 'vertex' was found")]
    NoVertexElement,
    #[error("No element named 'face' was found")]
    NoFaceElement,
    #[error("The element named 'face' contains no property 'vertex_indices' with triangle indices")]
    NoVertexIndices,
    #[error("The vertex indices do not denote triangles")]
    NoTriangleFaces,
    #[error("The property named '{}' is not a scalar", .0)]
    NotAScalar(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playground() {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../plyers/tests/valid/cube.ply"));
        let ply = load_ply(path).unwrap();

        let _ = RawMesh::with_ply(&ply).unwrap();
    }
}
