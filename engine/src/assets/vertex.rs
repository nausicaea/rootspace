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

    pub fn position(&self) -> &[f32; 3] {
        &self.position
    }

    pub fn tex_coord(&self) -> &[f32; 2] {
        &self.tex_coord
    }

    pub fn normals(&self) -> &[f32; 3] {
        &self.normals
    }
}

implement_vertex!(Vertex, position, tex_coord, normals);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _: Vertex = Vertex::new([0.0; 3], [0.0; 2], [0.0; 3]);
    }

    #[test]
    fn accessors() {
        let v = Vertex::new([0.0; 3], [0.0; 2], [0.0; 3]);

        assert_eq!(v.position(), &[0.0; 3]);
        assert_eq!(v.tex_coord(), &[0.0; 2]);
        assert_eq!(v.normals(), &[0.0; 3]);
    }
}
