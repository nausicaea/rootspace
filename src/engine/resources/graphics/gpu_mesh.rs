use crate::{
    engine::resources::graphics::ids::BufferId,
};

#[derive(Debug)]
pub struct GpuMesh {
    pub vertex_buffer: BufferId,
    pub instance_buffer: BufferId,
    pub index_buffer: BufferId,
    pub num_indices: u32,
    pub instance_id: u32,
}