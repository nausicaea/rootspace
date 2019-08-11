use crate::{
    assets::Vertex,
    graphics::{BackendTrait, IndexBufferTrait, ShaderTrait, TextureTrait, VertexBufferTrait},
};
use ecs::Resource;
use failure::Error;
use snowflake::ProcessUniqueId;
use std::{
    fmt,
    collections::HashMap,
    marker::PhantomData,
    path::Path,
};

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureId(ProcessUniqueId);

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderId(ProcessUniqueId);

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VertexBufferId(ProcessUniqueId);

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexBufferId(ProcessUniqueId);

#[cfg_attr(feature = "diagnostics", derive(TypeName))]
pub struct Backend<B>
where
    B: BackendTrait,
{
    textures: HashMap<TextureId, B::Texture>,
    shaders: HashMap<ShaderId, B::Shader>,
    vertex_buffers: HashMap<VertexBufferId, B::VertexBuffer>,
    index_buffers: HashMap<IndexBufferId, B::IndexBuffer>,
    _b: PhantomData<B>,
}

impl<B> Backend<B>
where
    B: BackendTrait,
{
    pub fn create_texture<P: AsRef<Path>>(&mut self, backend: &B, image: P) -> Result<TextureId, Error> {
        let t = B::Texture::from_path(backend, &image)?;
        let id = TextureId(ProcessUniqueId::default());
        self.textures.insert(id, t);
        Ok(id)
    }

    pub fn create_empty_texture(&mut self, backend: &B, dimensions: (u32, u32)) -> Result<TextureId, Error> {
        let t = B::Texture::empty(backend, dimensions)?;
        let id = TextureId(ProcessUniqueId::default());
        self.textures.insert(id, t);
        Ok(id)
    }

    pub fn create_shader<P: AsRef<Path>>(&mut self, backend: &B, vs: P, fs: P) -> Result<ShaderId, Error> {
        let s = B::Shader::from_paths(backend, &vs, &fs)?;
        let id = ShaderId(ProcessUniqueId::default());
        self.shaders.insert(id, s);
        Ok(id)
    }

    pub fn create_source_shader<S: AsRef<str>>(&mut self, backend: &B, vs: S, fs: S) -> Result<ShaderId, Error> {
        let s = B::Shader::from_source(backend, &vs, &fs)?;
        let id = ShaderId(ProcessUniqueId::default());
        self.shaders.insert(id, s);
        Ok(id)
    }

    pub fn create_vertex_buffer(&mut self, backend: &B, vertices: &[Vertex]) -> Result<VertexBufferId, Error> {
        let vbuf = B::VertexBuffer::from_vertices(backend, vertices)?;
        let id = VertexBufferId(ProcessUniqueId::default());
        self.vertex_buffers.insert(id, vbuf);
        Ok(id)
    }

    pub fn create_index_buffer(&mut self, backend: &B, indices: &[u16]) -> Result<IndexBufferId, Error> {
        let ibuf = B::IndexBuffer::from_indices(backend, indices)?;
        let id = IndexBufferId(ProcessUniqueId::default());
        self.index_buffers.insert(id, ibuf);
        Ok(id)
    }

    pub fn borrow_texture(&self, id: &TextureId) -> &B::Texture {
        self.textures.get(id).expect("Could not find the requested texture")
    }

    pub fn borrow_shader(&self, id: &ShaderId) -> &B::Shader {
        self.shaders.get(id).expect("Could not find the requested shader")
    }

    pub fn borrow_vertex_buffer(&self, id: &VertexBufferId) -> &B::VertexBuffer {
        self.vertex_buffers.get(id).expect("Could not find the requested vertex buffer")
    }

    pub fn borrow_index_buffer(&self, id: &IndexBufferId) -> &B::IndexBuffer {
        self.index_buffers.get(id).expect("Could not find the requested index buffer")
    }
}

impl<B> Default for Backend<B>
where
    B: BackendTrait,
{
    fn default() -> Self {
        Backend {
            textures: HashMap::default(),
            shaders: HashMap::default(),
            vertex_buffers: HashMap::default(),
            index_buffers: HashMap::default(),
            _b: PhantomData::default(),
        }
    }
}

impl<B> fmt::Debug for Backend<B>
where
    B: BackendTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Backend(#t: {}, #s: {}, #vbuf: {}, #ibuf: {})",
            self.textures.len(),
            self.shaders.len(),
            self.vertex_buffers.len(),
            self.index_buffers.len()
        )
    }
}

impl<B> Resource for Backend<B> where B: BackendTrait + 'static {}
