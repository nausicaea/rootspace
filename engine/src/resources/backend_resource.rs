use crate::{
    assets::AssetError,
    components::Renderable,
    graphics::{BackendTrait, IndexBufferTrait, ShaderTrait, TextureTrait, VertexBufferTrait, Vertex},
};
use ecs::{Component, Resource, MaybeDefault};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use snowflake::ProcessUniqueId;
use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};
use file_manipulation::{FileError, FilePathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendSettings {
    title: String,
    dimensions: (u32, u32),
    vsync: bool,
    msaa: u16,
    asset_tree: PathBuf,
}

impl BackendSettings {
    pub fn new<S: AsRef<str>, P: AsRef<Path>>(
        title: S,
        dimensions: (u32, u32),
        vsync: bool,
        msaa: u16,
        asset_tree: P,
    ) -> Self {
        let asset_tree = asset_tree.as_ref().canonicalize().expect(&format!(
            "Could not canonicalize the path {}",
            asset_tree.as_ref().display()
        ));

        BackendSettings {
            title: title.as_ref().to_string(),
            dimensions,
            vsync,
            msaa,
            asset_tree,
        }
    }

    pub fn build<B: BackendTrait>(&self) -> Result<BackendResource<B>> {
        TryFrom::try_from(self)
    }
}

impl Resource for BackendSettings {}

impl MaybeDefault for BackendSettings {}

impl<B> From<BackendResource<B>> for BackendSettings
where
    B: BackendTrait,
{
    fn from(value: BackendResource<B>) -> Self {
        value.settings.clone()
    }
}

pub struct BackendResource<B>
where
    B: BackendTrait,
{
    settings: BackendSettings,
    textures: HashMap<TextureId, B::Texture>,
    shaders: HashMap<ShaderId, B::Shader>,
    vertex_buffers: HashMap<VertexBufferId, B::VertexBuffer>,
    index_buffers: HashMap<IndexBufferId, B::IndexBuffer>,
    inner: B,
}

impl<B> BackendResource<B>
where
    B: BackendTrait,
{
    pub fn settings(&self) -> &BackendSettings {
        &self.settings
    }

    pub fn find_asset<P: AsRef<Path>>(&self, path: P) -> Result<FilePathBuf, AssetError> {
        let asset_path = FilePathBuf::try_from(self.settings.asset_tree.join(path))?;

        if !asset_path.path().starts_with(&self.settings.asset_tree) {
            return Err(AssetError::OutOfTree(asset_path.into()));
        }

        Ok(asset_path)
    }

    pub fn reload_assets(&mut self, renderables: &mut <Renderable as Component>::Storage) -> Result<()> {
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

impl<B> Resource for BackendResource<B> where B: BackendTrait + 'static {}

impl<B> TryFrom<BackendSettings> for BackendResource<B>
where
    B: BackendTrait,
{
    type Error = Error;

    fn try_from(value: BackendSettings) -> Result<Self> {
        Ok(BackendResource {
            settings: value.clone(),
            textures: HashMap::default(),
            shaders: HashMap::default(),
            vertex_buffers: HashMap::default(),
            index_buffers: HashMap::default(),
            inner: B::new(value.title, value.dimensions, value.vsync, value.msaa)?,
        })
    }
}

impl<B> TryFrom<&BackendSettings> for BackendResource<B>
where
    B: BackendTrait,
{
    type Error = Error;

    fn try_from(value: &BackendSettings) -> Result<Self> {
        Ok(BackendResource {
            settings: value.clone(),
            textures: HashMap::default(),
            shaders: HashMap::default(),
            vertex_buffers: HashMap::default(),
            index_buffers: HashMap::default(),
            inner: B::new(&value.title, value.dimensions, value.vsync, value.msaa)?,
        })
    }
}

impl<B> fmt::Debug for BackendResource<B>
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

impl<B> Deref for BackendResource<B>
where
    B: BackendTrait,
{
    type Target = B;

    fn deref(&self) -> &B {
        &self.inner
    }
}

impl<B> DerefMut for BackendResource<B>
where
    B: BackendTrait,
{
    fn deref_mut(&mut self) -> &mut B {
        &mut self.inner
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureId(Option<ProcessUniqueId>);

impl TextureId {
    fn generate() -> Self {
        TextureId(Some(ProcessUniqueId::new()))
    }
}

impl Default for TextureId {
    fn default() -> Self {
        TextureId(None)
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderId(Option<ProcessUniqueId>);

impl ShaderId {
    fn generate() -> Self {
        ShaderId(Some(ProcessUniqueId::new()))
    }
}

impl Default for ShaderId {
    fn default() -> Self {
        ShaderId(None)
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct VertexBufferId(Option<ProcessUniqueId>);

impl VertexBufferId {
    fn generate() -> Self {
        VertexBufferId(Some(ProcessUniqueId::new()))
    }
}

impl Default for VertexBufferId {
    fn default() -> Self {
        VertexBufferId(None)
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndexBufferId(Option<ProcessUniqueId>);

impl IndexBufferId {
    fn generate() -> Self {
        IndexBufferId(Some(ProcessUniqueId::new()))
    }
}

impl Default for IndexBufferId {
    fn default() -> Self {
        IndexBufferId(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::headless::HeadlessBackend;
    use serde_test::{assert_tokens, Token};

    #[test]
    fn backend_settings_new() {
        let resource_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/rootspace");
        let _: BackendSettings = BackendSettings::new("Title", (800, 600), false, 0, resource_path);
    }

    #[test]
    fn backend_settings_serde() {
        let resource_path = env!("CARGO_MANIFEST_DIR");
        let b: BackendSettings = BackendSettings::new("Title", (800, 600), false, 0, resource_path);

        assert_tokens(
            &b,
            &[
                Token::Struct {
                    name: "BackendSettings",
                    len: 5,
                },
                Token::Str("title"),
                Token::Str("Title"),
                Token::Str("dimensions"),
                Token::Tuple { len: 2 },
                Token::U32(800),
                Token::U32(600),
                Token::TupleEnd,
                Token::Str("vsync"),
                Token::Bool(false),
                Token::Str("msaa"),
                Token::U16(0),
                Token::Str("asset_tree"),
                Token::Str(resource_path),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn backend_resource_headless() {
        let resource_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/rootspace");
        let b: BackendSettings = BackendSettings::new("Title", (800, 600), false, 0, resource_path);
        let _: BackendResource<HeadlessBackend> = BackendResource::try_from(b).unwrap();
    }

    #[test]
    fn texture_id_default() {
        let id: TextureId = Default::default();
        assert_eq!(id, TextureId(None));
    }

    #[test]
    fn shader_id_default() {
        let id: ShaderId = Default::default();
        assert_eq!(id, ShaderId(None));
    }

    #[test]
    fn vertex_buffer_id_default() {
        let id: VertexBufferId = Default::default();
        assert_eq!(id, VertexBufferId(None));
    }

    #[test]
    fn index_buffer_id_default() {
        let id: IndexBufferId = Default::default();
        assert_eq!(id, IndexBufferId(None));
    }
}
