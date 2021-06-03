use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt,
    ops::{Deref, DerefMut},
};

use anyhow::{Error, Result};

use ecs::{Component, Resource, SerializationName};
use file_manipulation::FilePathBuf;
use index_buffer_id::IndexBufferId;
use shader_id::ShaderId;
use texture_id::TextureId;
use vertex_buffer_id::VertexBufferId;

use crate::{
    components::Renderable,
    graphics::{BackendTrait, IndexBufferTrait, ShaderTrait, TextureTrait, Vertex, VertexBufferTrait},
    resources::AssetDatabase,
};
use serde::{
    de,
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{convert::TryInto, marker::PhantomData};
use try_default::TryDefault;

pub mod index_buffer_id;
pub mod shader_id;
pub mod texture_id;
pub mod vertex_buffer_id;

pub struct GraphicsBackendBuilder<B> {
    title: String,
    dimensions: (u32, u32),
    vsync: bool,
    msaa: u16,
    _b: PhantomData<B>,
}

impl<B> GraphicsBackendBuilder<B>
where
    B: BackendTrait,
{
    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn with_dimensions(mut self, dimensions: (u32, u32)) -> Self {
        self.dimensions = dimensions;
        self
    }

    pub fn with_vsync(mut self, enabled: bool) -> Self {
        self.vsync = enabled;
        self
    }

    pub fn with_msaa(mut self, msaa: u16) -> Self {
        self.msaa = msaa;
        self
    }

    pub fn build(self) -> Result<GraphicsBackend<B>, Error> {
        self.try_into()
    }
}

impl<B> Default for GraphicsBackendBuilder<B>
where
    B: BackendTrait,
{
    fn default() -> Self {
        GraphicsBackendBuilder {
            title: String::default(),
            dimensions: (800, 600),
            vsync: false,
            msaa: 0,
            _b: PhantomData::default(),
        }
    }
}

// FIXME: The GraphicsBackend cannot be automatically initialized by World
pub struct GraphicsBackend<B>
where
    B: BackendTrait,
{
    title: String,
    dimensions: (u32, u32),
    vsync: bool,
    msaa: u16,
    textures: HashMap<TextureId, B::Texture>,
    shaders: HashMap<ShaderId, B::Shader>,
    vertex_buffers: HashMap<VertexBufferId, B::VertexBuffer>,
    index_buffers: HashMap<IndexBufferId, B::IndexBuffer>,
    inner: B,
}

impl<B> GraphicsBackend<B>
where
    B: BackendTrait,
{
    pub fn builder() -> GraphicsBackendBuilder<B> {
        GraphicsBackendBuilder::default()
    }

    pub fn reload_assets(
        &mut self,
        db: &AssetDatabase,
        renderables: &mut <Renderable as Component>::Storage,
    ) -> Result<()> {
        self.textures.clear();
        self.shaders.clear();
        self.vertex_buffers.clear();
        self.index_buffers.clear();
        for r in renderables.iter_mut() {
            r.reload(self, db)?;
        }

        Ok(())
    }

    pub fn create_texture(&mut self, image: &FilePathBuf) -> Result<TextureId> {
        let t = B::Texture::from_path(&self.inner, image)?;
        let id = TextureId::generate();
        self.textures.insert(id, t);
        Ok(id)
    }

    pub fn create_empty_texture(&mut self, dimensions: (u32, u32)) -> Result<TextureId> {
        let t = B::Texture::empty(&self.inner, dimensions)?;
        let id = TextureId::generate();
        self.textures.insert(id, t);
        Ok(id)
    }

    pub fn create_shader(&mut self, vs: &FilePathBuf, fs: &FilePathBuf) -> Result<ShaderId> {
        let s = B::Shader::from_paths(&self.inner, vs, fs)?;
        let id = ShaderId::generate();
        self.shaders.insert(id, s);
        Ok(id)
    }

    pub fn create_source_shader(&mut self, vs: &str, fs: &str) -> Result<ShaderId> {
        let s = B::Shader::from_source(&self.inner, &vs, &fs)?;
        let id = ShaderId::generate();
        self.shaders.insert(id, s);
        Ok(id)
    }

    pub fn create_vertex_buffer(&mut self, vertices: &[Vertex]) -> Result<VertexBufferId> {
        let vbuf = B::VertexBuffer::from_vertices(&self.inner, vertices)?;
        let id = VertexBufferId::generate();
        self.vertex_buffers.insert(id, vbuf);
        Ok(id)
    }

    pub fn create_index_buffer(&mut self, indices: &[u16]) -> Result<IndexBufferId> {
        let ibuf = B::IndexBuffer::from_indices(&self.inner, indices)?;
        let id = IndexBufferId::generate();
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
        self.vertex_buffers
            .get(id)
            .expect("Could not find the requested vertex buffer")
    }

    pub fn borrow_index_buffer(&self, id: &IndexBufferId) -> &B::IndexBuffer {
        self.index_buffers
            .get(id)
            .expect("Could not find the requested index buffer")
    }
}

impl<B> TryDefault for GraphicsBackend<B>
where
    B: BackendTrait,
{
    fn try_default() -> Result<Self, Error> {
        GraphicsBackendBuilder::<B>::default().build()
    }
}

impl<B> Resource for GraphicsBackend<B> where B: BackendTrait + 'static {}

impl<B> SerializationName for GraphicsBackend<B>
where
    B: BackendTrait,
{
    fn name() -> String {
        String::from("GraphicsBackend")
    }
}

impl<B> fmt::Debug for GraphicsBackend<B>
where
    B: BackendTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GraphicsBackend(#textures: {}, #shaders: {}, #vertex_buffers: {}, #index_buffers: {})",
            self.textures.len(),
            self.shaders.len(),
            self.vertex_buffers.len(),
            self.index_buffers.len()
        )
    }
}

impl<B> Deref for GraphicsBackend<B>
where
    B: BackendTrait,
{
    type Target = B;

    fn deref(&self) -> &B {
        &self.inner
    }
}

impl<B> DerefMut for GraphicsBackend<B>
where
    B: BackendTrait,
{
    fn deref_mut(&mut self) -> &mut B {
        &mut self.inner
    }
}

impl<B> TryFrom<GraphicsBackendBuilder<B>> for GraphicsBackend<B>
where
    B: BackendTrait,
{
    type Error = Error;

    fn try_from(value: GraphicsBackendBuilder<B>) -> Result<Self, Self::Error> {
        let inner = B::new(&value.title, value.dimensions, value.vsync, value.msaa)?;

        Ok(GraphicsBackend {
            title: value.title,
            dimensions: value.dimensions,
            vsync: value.vsync,
            msaa: value.msaa,
            textures: HashMap::default(),
            shaders: HashMap::default(),
            vertex_buffers: HashMap::default(),
            index_buffers: HashMap::default(),
            inner,
        })
    }
}

impl<B> Serialize for GraphicsBackend<B>
where
    B: BackendTrait,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("GraphicsBackend", 5)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("dimensions", &self.dimensions)?;
        state.serialize_field("vsync", &self.vsync)?;
        state.serialize_field("msaa", &self.msaa)?;
        state.skip_field("textures")?;
        state.skip_field("shaders")?;
        state.skip_field("vertex_buffers")?;
        state.skip_field("index_buffers")?;
        state.skip_field("inner")?;
        state.end()
    }
}

impl<'de, B> Deserialize<'de> for GraphicsBackend<B>
where
    B: BackendTrait,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "GraphicsBackend",
            GRAPHICS_BACKEND_FIELDS,
            GraphicsBackendVisitor::default(),
        )
    }
}

const GRAPHICS_BACKEND_FIELDS: &[&str] = &["title", "dimensions", "vsync", "msaa"];

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum GraphicsBackendField {
    Title,
    Dimensions,
    Vsync,
    Msaa,
}

struct GraphicsBackendVisitor<B>(PhantomData<B>);

impl<B> Default for GraphicsBackendVisitor<B> {
    fn default() -> Self {
        GraphicsBackendVisitor(PhantomData::default())
    }
}

impl<'de, B> Visitor<'de> for GraphicsBackendVisitor<B>
where
    B: BackendTrait,
{
    type Value = GraphicsBackend<B>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a serialized GraphicsBackend struct")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut title: Option<String> = None;
        let mut dimensions: Option<(u32, u32)> = None;
        let mut vsync: Option<bool> = None;
        let mut msaa: Option<u16> = None;

        while let Some(field_name) = map.next_key()? {
            match field_name {
                GraphicsBackendField::Title => {
                    if title.is_some() {
                        return Err(de::Error::duplicate_field("title"));
                    }
                    title = Some(map.next_value()?);
                }
                GraphicsBackendField::Dimensions => {
                    if dimensions.is_some() {
                        return Err(de::Error::duplicate_field("dimensions"));
                    }
                    dimensions = Some(map.next_value()?);
                }
                GraphicsBackendField::Vsync => {
                    if vsync.is_some() {
                        return Err(de::Error::duplicate_field("vsync"));
                    }
                    vsync = Some(map.next_value()?);
                }
                GraphicsBackendField::Msaa => {
                    if msaa.is_some() {
                        return Err(de::Error::duplicate_field("msaa"));
                    }
                    msaa = Some(map.next_value()?);
                }
            }
        }

        let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
        let dimensions = dimensions.ok_or_else(|| de::Error::missing_field("dimensions"))?;
        let vsync = vsync.ok_or_else(|| de::Error::missing_field("vsync"))?;
        let msaa = msaa.ok_or_else(|| de::Error::missing_field("msaa"))?;

        let inner = B::new(&title, dimensions, vsync, msaa).map_err(|e| de::Error::custom(format!("{}", e)))?;

        Ok(GraphicsBackend {
            title,
            dimensions,
            vsync,
            msaa,
            textures: HashMap::default(),
            shaders: HashMap::default(),
            vertex_buffers: HashMap::default(),
            index_buffers: HashMap::default(),
            inner,
        })
    }
}
