use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt,
    ops::{Deref, DerefMut},
    path::Path,
};

use anyhow::{Error, Result};

use ecs::{Component, Resource};
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
