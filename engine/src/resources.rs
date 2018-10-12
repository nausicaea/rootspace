#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
    normals: [f32; 3],
}

impl Vertex {
    pub fn new(position: [f32; 3], tex_coord: [f32; 2], normals: [f32; 3]) -> Self {
        Vertex {
            position,
            tex_coord,
            normals,
        }
    }
}

