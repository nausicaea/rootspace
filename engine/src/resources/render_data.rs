use crate::{
    assets::Vertex,
    graphics::{BackendTrait, IndexBufferTrait, ShaderTrait, TextureTrait, VertexBufferTrait},
};
use ecs::Resource;
use failure::Error;
use snowflake::ProcessUniqueId;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    path::Path,
};

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureId(u64);

impl TextureId {
    fn from_args<P: AsRef<Path>>(image: P) -> Self {
        let mut hasher = DefaultHasher::new();
        image.as_ref().hash(&mut hasher);
        TextureId(hasher.finish())
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderId(u64);

impl ShaderId {
    fn from_args<P: AsRef<Path>>(vs: P, fs: P) -> Self {
        let mut hasher = DefaultHasher::new();
        vs.as_ref().hash(&mut hasher);
        fs.as_ref().hash(&mut hasher);
        ShaderId(hasher.finish())
    }
}

#[derive(Copy, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct VertexBufferId(ProcessUniqueId);

#[derive(Copy, Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct IndexBufferId(ProcessUniqueId);

pub struct RenderData<B>
where
    B: BackendTrait,
{
    textures: HashMap<TextureId, B::Texture>,
    shaders: HashMap<ShaderId, B::Shader>,
    vertex_buffers: HashMap<VertexBufferId, B::VertexBuffer>,
    index_buffers: HashMap<IndexBufferId, B::IndexBuffer>,
    _b: PhantomData<B>,
}

impl<B> RenderData<B>
where
    B: BackendTrait,
{
    pub fn create_texture<P: AsRef<Path>>(&mut self, backend: &B, image: P) -> Result<TextureId, Error> {
        let t = B::Texture::from_path(backend, &image)?;
        let id = TextureId::from_args(image);
        self.textures.insert(id, t);
        Ok(id)
    }

    pub fn create_shader<P: AsRef<Path>>(&mut self, backend: &B, vs: P, fs: P) -> Result<ShaderId, Error> {
        let s = B::Shader::from_paths(backend, &vs, &fs)?;
        let id = ShaderId::from_args(vs, fs);
        self.shaders.insert(id, s);
        Ok(id)
    }

    pub fn create_vertex_buffer(&mut self, backend: &B, vertices: &[Vertex]) -> Result<VertexBufferId, Error> {
        let vbuf = B::VertexBuffer::from_vertices(backend, vertices)?;
        let id = VertexBufferId::default();
        self.vertex_buffers.insert(id, vbuf);
        Ok(id)
    }

    pub fn create_index_buffer(&mut self, backend: &B, indices: &[u16]) -> Result<IndexBufferId, Error> {
        let ibuf = B::IndexBuffer::from_indices(backend, indices)?;
        let id = IndexBufferId::default();
        self.index_buffers.insert(id, ibuf);
        Ok(id)
    }
}

impl<B> Default for RenderData<B>
where
    B: BackendTrait,
{
    fn default() -> Self {
        RenderData {
            textures: HashMap::default(),
            shaders: HashMap::default(),
            vertex_buffers: HashMap::default(),
            index_buffers: HashMap::default(),
            _b: PhantomData::default(),
        }
    }
}

impl<B> fmt::Debug for RenderData<B>
where
    B: BackendTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RenderData(#t: {}, #s: {}, #vbuf: {}, #ibuf: {})",
            self.textures.len(),
            self.shaders.len(),
            self.vertex_buffers.len(),
            self.index_buffers.len()
        )
    }
}

impl<B> Resource for RenderData<B> where B: BackendTrait + 'static {}
