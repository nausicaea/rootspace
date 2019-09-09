use crate::{
    assets::Vertex,
    graphics::{BackendTrait, IndexBufferTrait, ShaderTrait, TextureTrait, VertexBufferTrait},
};
use crate::components::Renderable;
use ecs::Component;
use failure::Error;
use snowflake::ProcessUniqueId;
use std::{collections::HashMap, fmt, path::Path};
use typename::TypeName;
use std::ops::{Deref, DerefMut};
use serde::{Deserialize, de::{self, Deserializer, Visitor, MapAccess}, ser::{Serialize, Serializer, SerializeStruct}};
use std::marker::PhantomData;

#[derive(TypeName)]
pub struct BackendResource<B>
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

impl<B> BackendResource<B>
where
    B: BackendTrait,
{
    pub fn new(title: &str, dimensions: (u32, u32), vsync: bool, msaa: u16) -> Result<Self, Error> {
        Ok(BackendResource {
            title: title.to_string(),
            dimensions,
            vsync,
            msaa,
            textures: HashMap::default(),
            shaders: HashMap::default(),
            vertex_buffers: HashMap::default(),
            index_buffers: HashMap::default(),
            inner: B::new(title, dimensions, vsync, msaa)?,
        })
    }

    pub fn reload_assets(&mut self, renderables: &mut <Renderable as Component>::Storage) -> Result<(), Error> {
        self.textures.clear();
        self.shaders.clear();
        self.vertex_buffers.clear();
        self.index_buffers.clear();
        for r in renderables.iter_mut() {
            r.reload(self)?;
        }

        Ok(())
    }

    pub fn create_texture<P: AsRef<Path>>(&mut self, image: P) -> Result<TextureId, Error> {
        let t = B::Texture::from_path(&self.inner, &image)?;
        let id = TextureId::generate();
        self.textures.insert(id, t);
        Ok(id)
    }

    pub fn create_empty_texture(&mut self, dimensions: (u32, u32)) -> Result<TextureId, Error> {
        let t = B::Texture::empty(&self.inner, dimensions)?;
        let id = TextureId::generate();
        self.textures.insert(id, t);
        Ok(id)
    }

    pub fn create_shader<P: AsRef<Path>>(&mut self, vs: P, fs: P) -> Result<ShaderId, Error> {
        let s = B::Shader::from_paths(&self.inner, &vs, &fs)?;
        let id = ShaderId::generate();
        self.shaders.insert(id, s);
        Ok(id)
    }

    pub fn create_source_shader<S: AsRef<str>>(&mut self, vs: S, fs: S) -> Result<ShaderId, Error> {
        let s = B::Shader::from_source(&self.inner, &vs, &fs)?;
        let id = ShaderId::generate();
        self.shaders.insert(id, s);
        Ok(id)
    }

    pub fn create_vertex_buffer(&mut self, vertices: &[Vertex]) -> Result<VertexBufferId, Error> {
        let vbuf = B::VertexBuffer::from_vertices(&self.inner, vertices)?;
        let id = VertexBufferId::generate();
        self.vertex_buffers.insert(id, vbuf);
        Ok(id)
    }

    pub fn create_index_buffer(&mut self, indices: &[u16]) -> Result<IndexBufferId, Error> {
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

impl<B> fmt::Debug for BackendResource<B>
where
    B: BackendTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "BackendResource(#t: {}, #s: {}, #vbuf: {}, #ibuf: {})",
            self.textures.len(),
            self.shaders.len(),
            self.vertex_buffers.len(),
            self.index_buffers.len()
        )
    }
}

impl<B> PartialEq<BackendResource<B>> for BackendResource<B>
where
    B: BackendTrait,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.title.eq(&rhs.title)
            && self.dimensions.eq(&rhs.dimensions)
            && self.vsync.eq(&rhs.vsync)
            && self.msaa.eq(&rhs.msaa)
    }
}

impl<B> Serialize for BackendResource<B>
where
    B: BackendTrait,
{
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = ser.serialize_struct("BackendResource", 4)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("dimensions", &self.dimensions)?;
        state.serialize_field("vsync", &self.vsync)?;
        state.serialize_field("msaa", &self.msaa)?;
        state.end()
    }
}

impl<'de, B> Deserialize<'de> for BackendResource<B>
where
    B: BackendTrait,
{
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["title", "dimensions", "vsync", "msaa"];

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Title,
            Dimensions,
            Vsync,
            Msaa,
        }

        struct BackendResourceVisitor<B>(PhantomData<B>);

        impl<B> Default for BackendResourceVisitor<B> {
            fn default() -> Self {
                BackendResourceVisitor(PhantomData::default())
            }
        }

        impl<'de, B> Visitor<'de> for BackendResourceVisitor<B>
        where
            B: BackendTrait,
        {
            type Value = BackendResource<B>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a struct BackendResource")
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut title: Option<String> = None;
                let mut dimensions = None;
                let mut vsync = None;
                let mut msaa = None;

                while let Some(key) = access.next_key()? {
                    match key {
                        Field::Title => {
                            if title.is_some() {
                                return Err(de::Error::duplicate_field("title"));
                            }
                            title = Some(access.next_value()?);
                        },
                        Field::Dimensions => {
                            if dimensions.is_some() {
                                return Err(de::Error::duplicate_field("dimensions"));
                            }
                            dimensions = Some(access.next_value()?);
                        },
                        Field::Vsync => {
                            if vsync.is_some() {
                                return Err(de::Error::duplicate_field("vsync"));
                            }
                            vsync = Some(access.next_value()?);
                        },
                        Field::Msaa => {
                            if msaa.is_some() {
                                return Err(de::Error::duplicate_field("msaa"));
                            }
                            msaa = Some(access.next_value()?);
                        },
                    }
                }

                let title = title.ok_or_else(|| de::Error::missing_field("title"))?;
                let dimensions = dimensions.ok_or_else(|| de::Error::missing_field("dimensions"))?;
                let vsync = vsync.ok_or_else(|| de::Error::missing_field("vsync"))?;
                let msaa = msaa.ok_or_else(|| de::Error::missing_field("msaa"))?;

                let b = BackendResource::new(&title, dimensions, vsync, msaa)
                    .map_err(|e| de::Error::custom(format!("failed to initialise the graphical backend ({})", e)))?;

                Ok(b)
            }
        }

        de.deserialize_struct("BackendResource", FIELDS, BackendResourceVisitor::<B>::default())
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
    fn backend_resource_headless() {
        let _: BackendResource<HeadlessBackend> = BackendResource::new("Title", (800, 600), true, 8).unwrap();
    }

    #[test]
    fn serde_headless() {
        let b: BackendResource<HeadlessBackend> = BackendResource::new("Title", (800, 600), true, 8).unwrap();

        assert_tokens(&b, &[
            Token::Struct { name: "BackendResource", len: 4 },
            Token::Str("title"),
            Token::Str("Title"),
            Token::Str("dimensions"),
            Token::Tuple { len: 2 },
            Token::U32(800),
            Token::U32(600),
            Token::TupleEnd,
            Token::Str("vsync"),
            Token::Bool(true),
            Token::Str("msaa"),
            Token::U16(8),
            Token::StructEnd,
        ]);
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
