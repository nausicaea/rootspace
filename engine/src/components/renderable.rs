use crate::{
    assets::{Mesh, Text},
    graphics::{
        BackendTrait,
    },
    resources::RenderData,
};
use ecs::{Component, VecStorage};
use failure::Error;
use std::{
    path::{Path, PathBuf},
};
use crate::resources::render_data::{VertexBufferId, IndexBufferId, ShaderId, TextureId};

#[cfg_attr(feature = "diagnostics", derive(TypeName))]
#[derive(Debug)]
pub struct Renderable {
    vertices: VertexBufferId,
    indices: IndexBufferId,
    shader: ShaderId,
    diffuse_texture: TextureId,
    normal_texture: Option<TextureId>,
}

impl Renderable {
    pub fn builder() -> RenderableBuilder {
        RenderableBuilder::default()
    }

    pub fn new(v: VertexBufferId, i: IndexBufferId, s: ShaderId, dt: TextureId, nt: Option<TextureId>) -> Self {
        Renderable {
            vertices: v,
            indices: i,
            shader: s,
            diffuse_texture: dt,
            normal_texture: nt,
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

    pub fn build_mesh<B: BackendTrait>(&self, backend: &B, factory: &mut RenderData<B>) -> Result<Renderable, Error> {
        let mesh_path = self.mesh.as_ref().ok_or(RenderableError::MissingMesh)?;
        let vs_path = self.vs.as_ref().ok_or(RenderableError::MissingVertexShader)?;
        let fs_path = self.fs.as_ref().ok_or(RenderableError::MissingFragmentShader)?;
        let dt_path = self.dt.as_ref().ok_or(RenderableError::MissingDiffuseTexture)?;

        let mesh = Mesh::from_path(mesh_path)?;

        let vertices = factory.create_vertex_buffer(backend, &mesh.vertices)?;
        let indices = factory.create_index_buffer(backend, &mesh.indices)?;
        let shader = factory.create_shader(backend, &vs_path, &fs_path)?;
        let diffuse_texture = factory.create_texture(backend, &dt_path)?;
        let normal_texture = if let Some(ref p) = self.nt {
            Some(factory.create_texture(backend, p)?)
        } else {
            None
        };

        Ok(Renderable {
            vertices,
            indices,
            shader,
            diffuse_texture,
            normal_texture,
        })
    }

    pub fn build_text<B: BackendTrait>(&self, backend: &B, factory: &mut RenderData<B>) -> Result<Renderable, Error> {
        let dpi_factor = backend.dpi_factor();

        let cache_size = (
            (self.cache_size.0 as f64 * dpi_factor) as u32,
            (self.cache_size.1 as f64 * dpi_factor) as u32,
        );
        let font_path = self.font.as_ref().ok_or(RenderableError::MissingFont)?;
        let text_scale = (self.text_scale as f64 * dpi_factor) as f32;
        let text_width = self.text_width;
        let pixel_width = self.virtual_pixel_text_width;
        let text = self.text.as_ref().ok_or(RenderableError::MissingText)?;
        let vs_path = self.vs.as_ref().ok_or(RenderableError::MissingVertexShader)?;
        let fs_path = self.fs.as_ref().ok_or(RenderableError::MissingFragmentShader)?;

        let diffuse_texture = factory.create_empty_texture(backend, cache_size)?;

        let text = Text::builder()
            .font(font_path)
            .cache(diffuse_texture)
            .scale(text_scale)
            .width(pixel_width)
            .layout(factory, &text)?;

        let mesh = text.mesh(text_width);
        let vertices = factory.create_vertex_buffer(backend, &mesh.vertices)?;
        let indices = factory.create_index_buffer(backend, &mesh.indices)?;

        let shader = factory.create_shader(backend, &vs_path, &fs_path)?;

        Ok(Renderable {
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
    use crate::graphics::headless::{HeadlessBackend, HeadlessEventsLoop};

    #[test]
    fn headless_builder_mesh() {
        let b = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", (800, 600), false, 0).unwrap();
        let mut f = RenderData::<HeadlessBackend>::default();
        let base_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
        let r: Result<Renderable, Error> = Renderable::builder()
            .mesh(&base_path.join("cube.ply"))
            .vertex_shader(&base_path.join("vertex.glsl"))
            .fragment_shader(&base_path.join("fragment.glsl"))
            .diffuse_texture(&base_path.join("tv-test-image.png"))
            .build_mesh(&b, &mut f);

        assert!(r.is_ok(), "{:?}", r);
    }

    #[test]
    fn headless_builder_text() {
        let b = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", (800, 600), false, 0).unwrap();
        let mut f = RenderData::<HeadlessBackend>::default();
        let base_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
        let r: Result<Renderable, Error> = Renderable::builder()
            .font(&base_path.join("SourceSansPro-Regular.ttf"))
            .vertex_shader(&base_path.join("vertex.glsl"))
            .fragment_shader(&base_path.join("fragment.glsl"))
            .text("Hello, World!")
            .build_text(&b, &mut f);

        assert!(r.is_ok(), "{:?}", r);
    }
}
