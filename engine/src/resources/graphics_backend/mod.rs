use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt,
    ops::{Deref, DerefMut},
    path::Path,
};

use anyhow::{Error, Result};

use ecs::{Component, Resource, MaybeDefault, SerializationProxy};
use file_manipulation::{FilePathBuf, DirPathBuf};
use index_buffer_id::IndexBufferId;
use shader_id::ShaderId;
use texture_id::TextureId;
use vertex_buffer_id::VertexBufferId;

use crate::{
    assets::AssetError,
    components::Renderable,
    graphics::{
        BackendTrait, IndexBufferTrait, ShaderTrait, TextureTrait, Vertex, VertexBufferTrait,
    },
};
use crate::resources::settings::Settings;
use serde::{Serialize, Deserialize, Serializer, Deserializer, de};
use serde::ser::SerializeStruct;
use std::marker::PhantomData;
use serde::de::{Visitor, MapAccess};
use std::fmt::Formatter;

pub mod texture_id;
pub mod shader_id;
pub mod vertex_buffer_id;
pub mod index_buffer_id;

// FIXME: The GraphicsBackend cannot be automatically initialized by World
pub struct GraphicsBackend<B>
where
    B: BackendTrait,
{
    asset_tree: DirPathBuf,
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
    pub fn new(settings: &Settings) -> Result<Self, Error> {
        Ok(GraphicsBackend {
        asset_tree: settings.asset_tree.clone(),
        title: settings.title.clone(),
        dimensions: settings.dimensions,
        vsync: settings.vsync,
        msaa: settings.msaa,
        textures: HashMap::default(),
        shaders: HashMap::default(),
        vertex_buffers: HashMap::default(),
        index_buffers: HashMap::default(),
        inner: B::new(
            &settings.title,
            settings.dimensions,
            settings.vsync,
            settings.msaa,
        )?,
    })
    }

    pub fn find_asset<P: AsRef<Path>>(&self, path: P) -> Result<FilePathBuf, AssetError> {
        let asset_path = FilePathBuf::try_from(self.asset_tree.join(path))?;

        if !asset_path.path().starts_with(&self.asset_tree) {
            return Err(AssetError::OutOfTree(asset_path.into()));
        }

        Ok(asset_path)
    }

    pub fn reload_assets(
        &mut self,
        renderables: &mut <Renderable as Component>::Storage,
    ) -> Result<()> {
        self.textures.clear();
        self.shaders.clear();
        self.vertex_buffers.clear();
        self.index_buffers.clear();
        for r in renderables.iter_mut() {
            r.reload(self)?;
        }

        Ok(())
    }

    pub fn create_texture<P: AsRef<Path>>(&mut self, image: P) -> Result<TextureId> {
        let image = self.find_asset(image)?;
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

    pub fn create_shader<P: AsRef<Path>>(&mut self, vs: P, fs: P) -> Result<ShaderId> {
        let vs = self.find_asset(vs)?;
        let fs = self.find_asset(fs)?;
        let s = B::Shader::from_paths(&self.inner, vs, fs)?;
        let id = ShaderId::generate();
        self.shaders.insert(id, s);
        Ok(id)
    }

    pub fn create_source_shader<S: AsRef<str>>(&mut self, vs: S, fs: S) -> Result<ShaderId> {
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
        self.textures
            .get(id)
            .expect("Could not find the requested texture")
    }

    pub fn borrow_shader(&self, id: &ShaderId) -> &B::Shader {
        self.shaders
            .get(id)
            .expect("Could not find the requested shader")
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

impl<B> Resource for GraphicsBackend<B> where B: BackendTrait + 'static {}

impl<B> SerializationProxy for GraphicsBackend<B>
where B: BackendTrait,
{
    fn name() -> String {
        String::from("GraphicsBackend")
    }
}

impl<B> MaybeDefault for GraphicsBackend<B> where B: BackendTrait {}

impl<B> fmt::Debug for GraphicsBackend<B>
where
    B: BackendTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BackendResource(#textures: {}, #shaders: {}, #vertex_buffers: {}, #index_buffers: {})",
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

impl<B> Serialize for GraphicsBackend<B>
where
    B: BackendTrait,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("GraphicsBackend", 5)?;
        state.serialize_field("asset_tree", &self.asset_tree)?;
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

const GRAPHICS_BACKEND_FIELDS: &'static [&'static str] = &[
    "asset_tree",
    "title",
    "dimensions",
    "vsync",
    "msaa",
];

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "snake_case")]
enum GraphicsBackendField {
    AssetTree,
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
        let mut asset_tree: Option<DirPathBuf> = None;
        let mut title: Option<String> = None;
        let mut dimensions: Option<(u32, u32)> = None;
        let mut vsync: Option<bool> = None;
        let mut msaa: Option<u16> = None;

        while let Some(field_name) = map.next_key()? {
            match field_name {
                GraphicsBackendField::AssetTree => {
                    if asset_tree.is_some() {
                        return Err(de::Error::duplicate_field("asset_tree"));
                    }
                    asset_tree = Some(map.next_value()?);
                },
                GraphicsBackendField::Title => {
                    if title.is_some() {
                        return Err(de::Error::duplicate_field("title"));
                    }
                    title = Some(map.next_value()?);
                },
                GraphicsBackendField::Dimensions => {
                    if dimensions.is_some() {
                        return Err(de::Error::duplicate_field("dimensions"));
                    }
                    dimensions = Some(map.next_value()?);
                },
                GraphicsBackendField::Vsync => {
                    if vsync.is_some() {
                        return Err(de::Error::duplicate_field("vsync"));
                    }
                    vsync = Some(map.next_value()?);
                },
                GraphicsBackendField::Msaa => {
                    if msaa.is_some() {
                        return Err(de::Error::duplicate_field("msaa"));
                    }
                    msaa = Some(map.next_value()?);
                }
            }
        }

        let asset_tree = asset_tree.ok_or_else(|| de::Error::missing_field("asset_tree"))?;
        let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
        let dimensions = dimensions.ok_or_else(|| de::Error::missing_field("dimensions"))?;
        let vsync = vsync.ok_or_else(|| de::Error::missing_field("vsync"))?;
        let msaa = msaa.ok_or_else(|| de::Error::missing_field("msaa"))?;

        let inner = B::new(
            &title,
            dimensions,
            vsync,
            msaa,
        ).map_err(|e| de::Error::custom(format!("{}", e)))?;

        Ok(GraphicsBackend {
            asset_tree,
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