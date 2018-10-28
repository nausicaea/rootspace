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

implement_vertex!(Vertex, position, tex_coord, normals);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_vertex() {
        let _: Vertex = Vertex::new([0.0; 3], [0.0; 2], [0.0; 3]);
    }
}
