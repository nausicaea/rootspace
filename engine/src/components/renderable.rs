use crate::{
    assets::{AssetTrait, Mesh},
    graphics::{BackendTrait, text::Text},
    resources::{BackendResource, IndexBufferId, ShaderId, TextureId, VertexBufferId},
};
use ecs::{Component, VecStorage};
use anyhow::Result;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderableType {
    Mesh,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Default, Serialize, Deserialize)]
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

    pub fn reload<B: BackendTrait>(&mut self, factory: &mut BackendResource<B>) -> Result<()> {
        match self.source {
            Some(SourceData::Mesh {
                ref file,
                ref vertex_shader,
                ref fragment_shader,
                ref diffuse_texture,
                ref normal_texture,
            }) => {
                let mesh_path = factory.find_asset(file)?;
                let mesh = Mesh::from_path(mesh_path)?;

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
            }
            Some(SourceData::Text {
                ref text,
                ref font,
                text_scale,
                text_width,
                virtual_pixel_text_width,
                cache_size,
                ref vertex_shader,
                ref fragment_shader,
            }) => {
                let dpi_factor = factory.dpi_factor();
                let scaled_cache_size = (
                    (cache_size.0 as f64 * dpi_factor) as u32,
                    (cache_size.1 as f64 * dpi_factor) as u32,
                );
                let scaled_text_scale = (text_scale as f64 * dpi_factor) as f32;

                self.diffuse_texture = factory.create_empty_texture(scaled_cache_size)?;
                let font_path = factory.find_asset(font)?;
                let text_data = Text::builder()
                    .font(font_path)
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
            }
            None => Err(From::from(RenderableError::NoSourceDataPresent)),
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
    ty: RenderableType,
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
            ty: RenderableType::Mesh,
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
    pub fn with_type(&mut self, ty: RenderableType) -> &mut Self {
        self.ty = ty;
        self
    }

    pub fn with_mesh<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.mesh = Some(path.as_ref().into());
        self
    }

    pub fn with_vertex_shader<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.vs = Some(path.as_ref().into());
        self
    }

    pub fn with_fragment_shader<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.fs = Some(path.as_ref().into());
        self
    }

    pub fn with_diffuse_texture<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.dt = Some(path.as_ref().into());
        self
    }

    pub fn with_normal_texture<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.nt = Some(path.as_ref().into());
        self
    }

    pub fn with_cache_size(&mut self, dims: (u32, u32)) -> &mut Self {
        self.cache_size = dims;
        self
    }

    pub fn with_font<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
        self.font = Some(path.as_ref().into());
        self
    }

    pub fn with_text_scale(&mut self, scale: f32) -> &mut Self {
        self.text_scale = scale;
        self
    }

    pub fn with_text_width(&mut self, model_width: f32, pixel_width: u32) -> &mut Self {
        self.text_width = model_width;
        self.virtual_pixel_text_width = pixel_width;
        self
    }

    pub fn with_text(&mut self, text: &str) -> &mut Self {
        self.text = Some(text.into());
        self
    }

    pub fn build<B: BackendTrait>(&self, factory: &mut BackendResource<B>) -> Result<Renderable> {
        match self.ty {
            RenderableType::Mesh => {
                let mesh_path = self.mesh.as_ref().ok_or(RenderableError::MissingMesh)?;
                let vs_path = self.vs.as_ref().ok_or(RenderableError::MissingVertexShader)?;
                let fs_path = self.fs.as_ref().ok_or(RenderableError::MissingFragmentShader)?;
                let dt_path = self.dt.as_ref().ok_or(RenderableError::MissingDiffuseTexture)?;

                let mut renderable = Renderable {
                    source: Some(SourceData::Mesh {
                        file: mesh_path.to_path_buf(),
                        vertex_shader: vs_path.to_path_buf(),
                        fragment_shader: fs_path.to_path_buf(),
                        diffuse_texture: dt_path.to_path_buf(),
                        normal_texture: self.nt.clone(),
                    }),
                    ..Default::default()
                };

                renderable.reload(factory)?;

                Ok(renderable)
            },
            RenderableType::Text => {
                let text = self.text.as_ref().ok_or(RenderableError::MissingText)?;
                let font_path = self.font.as_ref().ok_or(RenderableError::MissingFont)?;
                let vs_path = self.vs.as_ref().ok_or(RenderableError::MissingVertexShader)?;
                let fs_path = self.fs.as_ref().ok_or(RenderableError::MissingFragmentShader)?;

                let mut renderable = Renderable {
                    source: Some(SourceData::Text {
                        text: text.to_string(),
                        font: font_path.to_path_buf(),
                        text_scale: self.text_scale,
                        text_width: self.text_width,
                        virtual_pixel_text_width: self.virtual_pixel_text_width,
                        cache_size: self.cache_size,
                        vertex_shader: vs_path.to_path_buf(),
                        fragment_shader: fs_path.to_path_buf(),
                    }),
                    ..Default::default()
                };

                renderable.reload(factory)?;

                Ok(renderable)
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum RenderableError {
    #[error("You must provide a mesh to build a Renderable")]
    MissingMesh,
    #[error("You must provide a vertex shader to build a Renderable")]
    MissingVertexShader,
    #[error("You must provide a fragment shader to build a Renderable")]
    MissingFragmentShader,
    #[error("You must provide a diffuse texture to build a Renderable")]
    MissingDiffuseTexture,
    #[error("You must provide a font to build a Renderable")]
    MissingFont,
    #[error("You must provide text to build a Renderable")]
    MissingText,
    #[error("Cannot reload the renderable because no source data is present")]
    NoSourceDataPresent,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graphics::headless::HeadlessBackend, resources::BackendSettings};

    #[test]
    fn headless_builder_mesh() {
        let resource_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/rootspace");
        let mut f = BackendSettings::new("Title", (800, 600), false, 0, resource_path)
            .build::<HeadlessBackend>()
            .unwrap();
        let r: Result<Renderable> = Renderable::builder()
            .with_mesh("meshes/cube.ply")
            .with_vertex_shader("shaders/test-vertex.glsl")
            .with_fragment_shader("shaders/test-fragment.glsl")
            .with_diffuse_texture("textures/tv-test-image.png")
            .with_type(RenderableType::Mesh)
            .build(&mut f);

        r.unwrap();
    }

    #[test]
    fn headless_builder_text() {
        let resource_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/rootspace");
        let mut f = BackendSettings::new("Title", (800, 600), false, 0, resource_path)
            .build::<HeadlessBackend>()
            .unwrap();
        let r: Result<Renderable> = Renderable::builder()
            .with_font("fonts/SourceSansPro-Regular.ttf")
            .with_vertex_shader("shaders/test-vertex.glsl")
            .with_fragment_shader("shaders/test-fragment.glsl")
            .with_text("Hello, World!")
            .with_type(RenderableType::Text)
            .build(&mut f);

        r.unwrap();
    }
}
