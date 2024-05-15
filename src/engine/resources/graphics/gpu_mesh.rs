use wgpu::{BufferAddress, BufferUsages};

use crate::{
    ecs::resources::Resources,
    engine::resources::graphics::{ids::BufferId, instance::Instance, Graphics},
};
use crate::engine::assets::cpu_mesh::CpuMesh;

#[derive(Debug)]
pub struct GpuMesh {
    pub vertex_buffer: BufferId,
    pub instance_buffer: BufferId,
    pub index_buffer: BufferId,
    pub num_indices: u32,
    pub instance_id: u32,
}

impl GpuMesh {
    pub fn with_mesh(res: &Resources, mesh: &CpuMesh) -> Self {
        let mut gfx = res.write::<Graphics>();
        let vertex_buffer = gfx.create_buffer_init(
            mesh.label.as_ref().map(|l| format!("{}:vertex-buffer", &l)).as_deref(),
            BufferUsages::VERTEX,
            &mesh.vertices,
        );
        let instance_buffer = {
            let max_instances = gfx.max_instances();
            let buffer_alignment = std::mem::size_of::<Instance>() as u64;
            let buffer_size = (max_instances * buffer_alignment) as BufferAddress;
            gfx.create_buffer(
                mesh.label
                    .as_ref()
                    .map(|l| format!("{}:instance-buffer", &l))
                    .as_deref(),
                buffer_size,
                BufferUsages::VERTEX | BufferUsages::COPY_DST,
            )
        };
        let index_buffer = gfx.create_buffer_init(
            mesh.label.as_ref().map(|l| format!("{}:index-buffer", &l)).as_deref(),
            BufferUsages::INDEX,
            &mesh.indices,
        );

        GpuMesh {
            vertex_buffer,
            instance_buffer,
            index_buffer,
            num_indices: mesh.indices.len() as u32,
            instance_id: 0,
        }
    }
}
