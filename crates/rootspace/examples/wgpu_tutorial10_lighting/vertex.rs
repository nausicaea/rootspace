pub trait Vertex {
    fn desc() -> griffon::wgpu::VertexBufferLayout<'static>;
}
