use ecs::Resource;
use crate::graphics::BackendTrait;
use crate::assets::Text;
use std::marker::PhantomData;
use std::fmt;
use std::collections::HashMap;

pub struct VertexBufferId;

pub struct IndexBufferId;

pub struct ShaderId;

pub struct TextureId;

pub struct TextId;

pub struct RenderData<B>
where
    B: BackendTrait,
{
    textures: HashMap<TextureId, B::Texture>,
    shaders: HashMap<ShaderId, B::Shader>,
    vertex_buffers: HashMap<VertexBufferId, B::VertexBuffer>,
    index_buffers: HashMap<IndexBufferId, B::IndexBuffer>,
    texts: HashMap<TextId, Text<B>>,
    _b: PhantomData<B>,
}

impl<B> RenderData<B>
where
    B: BackendTrait,
{
    pub fn create_vertex_buffer() -> VertexBufferId {
        unimplemented!()
    }

    pub fn create_index_buffer() -> IndexBufferId {
        unimplemented!()
    }

    pub fn create_shader() -> ShaderId {
        unimplemented!()
    }

    pub fn create_texture() -> TextureId {
        unimplemented!()
    }

    pub fn create_text() -> TextId {
        unimplemented!()
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
            texts: HashMap::default(),
            _b: PhantomData::default(),
        }
    }
}

impl<B> fmt::Debug for RenderData<B>
where
    B: BackendTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RenderData(#t: {}, #s: {}, #vbuf: {}, #ibuf: {}, #txt: {})", self.textures.len(), self.shaders.len(), self.vertex_buffers.len(), self.index_buffers.len(), self.texts.len())
    }
}

impl<B> Resource for RenderData<B> where B: BackendTrait {}
