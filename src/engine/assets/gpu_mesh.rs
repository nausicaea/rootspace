use super::cpu_mesh::CpuMesh;
use crate::ecs::resources::Resources;
use crate::engine::resources::graphics::ids::BufferId;
use crate::engine::resources::graphics::Graphics;

#[derive(Debug)]
pub struct GpuMesh {
    pub vertex_buffer: BufferId,
    pub index_buffer: BufferId,
    pub num_indices: u32,
}

impl GpuMesh {
    pub fn with_mesh(res: &Resources, mesh: &CpuMesh) -> Self {
        let mut gfx = res.write::<Graphics>();
        let vertex_buffer = gfx.create_buffer_init(
            mesh.label.as_ref().map(|l| format!("{}:vertex-buffer", &l)).as_deref(),
            wgpu::BufferUsages::VERTEX,
            &mesh.vertices,
        );
        let index_buffer = gfx.create_buffer_init(
            mesh.label.as_ref().map(|l| format!("{}:index-buffer", &l)).as_deref(),
            wgpu::BufferUsages::INDEX,
            &mesh.indices,
        );

        GpuMesh {
            vertex_buffer,
            index_buffer,
            num_indices: mesh.indices.len() as u32,
        }
    }
}
