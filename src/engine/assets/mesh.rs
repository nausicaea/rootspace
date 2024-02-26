use super::{raw_mesh::RawMesh, Error};
use crate::ecs::resources::Resources;
use crate::engine::resources::graphics::ids::BufferId;
use crate::engine::resources::graphics::Graphics;

#[derive(Debug)]
pub struct Mesh {
    pub vertex_buffer: BufferId,
    pub index_buffer: BufferId,
    pub num_indices: u32,
}

impl Mesh {
    pub(crate) fn with_raw_mesh(res: &Resources, raw_mesh: &RawMesh) -> Result<Self, Error> {
        let mut gfx = res.borrow_mut::<Graphics>();
        let vertex_buffer = gfx.create_buffer_init(raw_mesh.label.as_ref().map(|l| format!("{}:vertex-buffer", &l)).as_deref(), wgpu::BufferUsages::VERTEX, &raw_mesh.vertices);
        let index_buffer = gfx.create_buffer_init(raw_mesh.label.as_ref().map(|l| format!("{}:index-buffer", &l)).as_deref(), wgpu::BufferUsages::INDEX, &raw_mesh.indices);

        Ok(Mesh {
            vertex_buffer,
            index_buffer,
            num_indices: raw_mesh.indices.len() as u32,
        })
    }
}
