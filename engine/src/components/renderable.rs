use crate::{
    assets::{AssetTrait, Mesh, Text},
    graphics::BackendTrait,
    resources::{BackendResource, IndexBufferId, ShaderId, TextureId, VertexBufferId},
};
use ecs::{Component, VecStorage};
use failure::{Error, Fail, format_err};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use typename::TypeName;

#[derive(Debug, Clone, TypeName, Serialize, Deserialize)]
pub enum SourceData {
    Mesh {
        file: PathBuf,
        vertex_shader: PathBuf,
        fragment_shader: PathBuf,
        diffuse_texture: PathBuf,
        normal_texture: Option<PathBuf>,
    },
    Text {
        text: String,
        font: PathBuf,
        text_scale: f32,
        text_width: f32,
        virtual_pixel_text_width: u32,
        cache_size: (u32, u32),
        vertex_shader: PathBuf,
        fragment_shader: PathBuf,
    },
}

#[derive(Debug, Default, TypeName, Serialize, Deserialize)]
pub struct Renderable {
    source: Option<SourceData>,
    #[serde(skip)]
    vertices: VertexBufferId,
    #[serde(skip)]
    indices: IndexBufferId,
    #[serde(skip)]
    shader: ShaderId,
    #[serde(skip)]
    diffuse_texture: TextureId,
    #[serde(skip)]
    normal_texture: Option<TextureId>,
}

impl Renderable {
    pub fn builder() -> RenderableBuilder {
        RenderableBuilder::default()
    }

    #[cfg(test)]
    pub fn new(v: VertexBufferId, i: IndexBufferId, s: ShaderId, dt: TextureId, nt: Option<TextureId>) -> Self {
        Renderable {
            source: None,
            vertices: v,
            indices: i,
            shader: s,
            diffuse_texture: dt,
            normal_texture: nt,
        }
    }

    pub fn reload<B: BackendTrait>(&mut self, factory: &mut BackendResource<B>) -> Result<(), Error> {
        match self.source {
            Some(SourceData::Mesh { ref file, ref vertex_shader, ref fragment_shader, ref diffuse_texture, ref normal_texture }) => {
                let mesh = Mesh::from_path(file)?;

                self.vertices = factory.create_vertex_buffer(&mesh.vertices)?;
                self.indices = factory.create_index_buffer(&mesh.indices)?;
                self.shader = factory.create_shader(vertex_shader, fragment_shader)?;
                self.diffuse_texture = factory.create_texture(diffuse_texture)?;
                self.normal_texture = if let Some(ref p) = normal_texture {
                    Some(factory.create_texture(p)?)
                } else {
                    None
                };
                Ok(())
            },
            Some(SourceData::Text { ref text, ref font, text_scale, text_width, virtual_pixel_text_width, cache_size, ref vertex_shader, ref fragment_shader }) => {
                let dpi_factor = factory.dpi_factor();
                let scaled_cache_size = (
                    (cache_size.0 as f64 * dpi_factor) as u32,
                    (cache_size.1 as f64 * dpi_factor) as u32,
                    );
                let scaled_text_scale = (text_scale as f64 * dpi_factor) as f32;

                self.diffuse_texture = factory.create_empty_texture(scaled_cache_size)?;
                let text_data = Text::builder()
                    .font(font)
                    .cache(self.diffuse_texture)
                    .scale(scaled_text_scale)
                    .width(virtual_pixel_text_width)
                    .layout(factory, &text)?;
                let mesh = text_data.mesh(text_width);
                self.vertices = factory.create_vertex_buffer(&mesh.vertices)?;
                self.indices = factory.create_index_buffer(&mesh.indices)?;
                self.shader = factory.create_shader(&vertex_shader, &fragment_shader)?;
                self.normal_texture = None;
                Ok(())
            },
            None => Err(format_err!("Cannot reload the renderable because no source data is present: {:?}", self)),
        }
    }

    pub fn vertices(&self) -> &VertexBufferId {
        &self.vertices
    }

    pub fn indices(&self) -> &IndexBufferId {
        &self.indices
    }

    pub fn shader(&self) -> &ShaderId {
        &self.shader
    }

    pub fn diffuse_texture(&self) -> &TextureId {
        &self.diffuse_texture
    }

    pub fn normal_texture(&self) -> Option<&TextureId> {
        self.normal_texture.as_ref()
    }
}

impl Component for Renderable {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct RenderableBuilder {
    mesh: Option<PathBuf>,
    vs: Option<PathBuf>,
    fs: Option<PathBuf>,
    dt: Option<PathBuf>,
    nt: Option<PathBuf>,
    cache_size: (u32, u32),
    font: Option<PathBuf>,
    text_scale: f32,
    text_width: f32,
    virtual_pixel_text_width: u32,
    text: Option<String>,
}

impl Default for RenderableBuilder {
    fn default() -> Self {
        RenderableBuilder {
            mesh: None,
            vs: None,
            fs: None,
            dt: None,
            nt: None,
            cache_size: (512, 512),
            font: None,
            text_scale: 16.0,
            text_width: 1.0,
            virtual_pixel_text_width: 100,
            text: None,
        }
    }
}

impl RenderableBuilder {
    pub fn mesh<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.mesh = Some(path.as_ref().into());
        self
    }

    pub fn vertex_shader<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.vs = Some(path.as_ref().into());
        self
    }

    pub fn fragment_shader<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.fs = Some(path.as_ref().into());
        self
    }

    pub fn diffuse_texture<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.dt = Some(path.as_ref().into());
        self
    }

    pub fn normal_texture<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.nt = Some(path.as_ref().into());
        self
    }

    pub fn cache_size(&mut self, dims: (u32, u32)) -> &mut Self {
        self.cache_size = dims;
        self
    }

    pub fn font<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.font = Some(path.as_ref().into());
        self
    }

    pub fn text_scale(&mut self, scale: f32) -> &mut Self {
        self.text_scale = scale;
        self
    }

    pub fn text_width(&mut self, model_width: f32, pixel_width: u32) -> &mut Self {
        self.text_width = model_width;
        self.virtual_pixel_text_width = pixel_width;
        self
    }

    pub fn text(&mut self, text: &str) -> &mut Self {
        self.text = Some(text.into());
        self
    }

    pub fn build_mesh<B: BackendTrait>(&self, factory: &mut BackendResource<B>) -> Result<Renderable, Error> {
        let mesh_path = self.mesh.as_ref().ok_or(RenderableError::MissingMesh)?;
        let vs_path = self.vs.as_ref().ok_or(RenderableError::MissingVertexShader)?;
        let fs_path = self.fs.as_ref().ok_or(RenderableError::MissingFragmentShader)?;
        let dt_path = self.dt.as_ref().ok_or(RenderableError::MissingDiffuseTexture)?;

        let mesh = Mesh::from_path(&mesh_path)?;

        let vertices = factory.create_vertex_buffer(&mesh.vertices)?;
        let indices = factory.create_index_buffer(&mesh.indices)?;
        let shader = factory.create_shader(&vs_path, &fs_path)?;
        let diffuse_texture = factory.create_texture(&dt_path)?;
        let normal_texture = if let Some(ref p) = self.nt {
            Some(factory.create_texture(p)?)
        } else {
            None
        };

        Ok(Renderable {
            source: Some(SourceData::Mesh {
                file: mesh_path.to_path_buf(),
                vertex_shader: vs_path.to_path_buf(),
                fragment_shader: fs_path.to_path_buf(),
                diffuse_texture: dt_path.to_path_buf(),
                normal_texture: self.nt.clone(),
            }),
            vertices,
            indices,
            shader,
            diffuse_texture,
            normal_texture,
        })
    }

    pub fn build_text<B: BackendTrait>(&self, factory: &mut BackendResource<B>) -> Result<Renderable, Error> {
        let dpi_factor = factory.dpi_factor();

        let scaled_cache_size = (
            (self.cache_size.0 as f64 * dpi_factor) as u32,
            (self.cache_size.1 as f64 * dpi_factor) as u32,
        );
        let font_path = self.font.as_ref().ok_or(RenderableError::MissingFont)?;
        let scaled_text_scale = (self.text_scale as f64 * dpi_factor) as f32;
        let text_width = self.text_width;
        let pixel_width = self.virtual_pixel_text_width;
        let text = self.text.as_ref().ok_or(RenderableError::MissingText)?;
        let vs_path = self.vs.as_ref().ok_or(RenderableError::MissingVertexShader)?;
        let fs_path = self.fs.as_ref().ok_or(RenderableError::MissingFragmentShader)?;

        let diffuse_texture = factory.create_empty_texture(scaled_cache_size)?;

        let text_data = Text::builder()
            .font(font_path)
            .cache(diffuse_texture)
            .scale(scaled_text_scale)
            .width(pixel_width)
            .layout(factory, &text)?;

        let mesh = text_data.mesh(text_width);
        let vertices = factory.create_vertex_buffer(&mesh.vertices)?;
        let indices = factory.create_index_buffer(&mesh.indices)?;

        let shader = factory.create_shader(&vs_path, &fs_path)?;

        Ok(Renderable {
            source: Some(SourceData::Text {
                text: text.to_string(),
                font: font_path.to_path_buf(),
                text_scale: self.text_scale,
                text_width,
                virtual_pixel_text_width: pixel_width,
                cache_size: self.cache_size,
                vertex_shader: vs_path.to_path_buf(),
                fragment_shader: fs_path.to_path_buf(),
            }),
            vertices,
            indices,
            shader,
            diffuse_texture,
            normal_texture: None,
        })
    }
}

#[derive(Debug, Fail)]
pub enum RenderableError {
    #[fail(display = "You must provide a mesh to build a Renderable")]
    MissingMesh,
    #[fail(display = "You must provide a vertex shader to build a Renderable")]
    MissingVertexShader,
    #[fail(display = "You must provide a fragment shader to build a Renderable")]
    MissingFragmentShader,
    #[fail(display = "You must provide a diffuse texture to build a Renderable")]
    MissingDiffuseTexture,
    #[fail(display = "You must provide a font to build a Renderable")]
    MissingFont,
    #[fail(display = "You must provide text to build a Renderable")]
    MissingText,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::headless::HeadlessBackend;

    #[test]
    fn headless_builder_mesh() {
        let mut f = BackendResource::<HeadlessBackend>::new("Title", (800, 600), false, 0).unwrap();
        let base_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
        let r: Result<Renderable, Error> = Renderable::builder()
            .mesh(&base_path.join("cube.ply"))
            .vertex_shader(&base_path.join("vertex.glsl"))
            .fragment_shader(&base_path.join("fragment.glsl"))
            .diffuse_texture(&base_path.join("tv-test-image.png"))
            .build_mesh(&mut f);

        assert!(r.is_ok(), "{:?}", r);
    }

    #[test]
    fn headless_builder_text() {
        let mut f = BackendResource::<HeadlessBackend>::new("Title", (800, 600), false, 0).unwrap();
        let base_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
        let r: Result<Renderable, Error> = Renderable::builder()
            .font(&base_path.join("SourceSansPro-Regular.ttf"))
            .vertex_shader(&base_path.join("vertex.glsl"))
            .fragment_shader(&base_path.join("fragment.glsl"))
            .text("Hello, World!")
            .build_text(&mut f);

        assert!(r.is_ok(), "{:?}", r);
    }
}
