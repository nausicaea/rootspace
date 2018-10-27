use failure::Error;
use file_manipulation::ReadPath;
use glium::{
    index::{IndexBuffer, PrimitiveType},
    program::Program,
    vertex::VertexBuffer,
};
use graphics::glium::{GliumBackend, GliumRenderData, GliumTexture};
use graphics::headless::{HeadlessBackend, HeadlessRenderData, HeadlessTexture};
use graphics::{BackendTrait, DataTrait, TextureTrait};
use resources::{Image, Mesh, Text};
use std::borrow::Borrow;
use std::fmt;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

pub struct Renderable<B: BackendTrait> {
    data: B::Data,
    _b: PhantomData<B>,
}

impl<B: BackendTrait> Renderable<B> {
    pub fn builder() -> RenderableBuilder<B> {
        RenderableBuilder::default()
    }
}

impl<B: BackendTrait> fmt::Debug for Renderable<B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Renderable {{ ... }}")
    }
}

impl<B, D> Borrow<D> for Renderable<B>
where
    B: BackendTrait<Data = D>,
    D: DataTrait,
{
    fn borrow(&self) -> &D {
        &self.data
    }
}

#[derive(Debug)]
pub struct RenderableBuilder<B: BackendTrait> {
    mesh: Option<PathBuf>,
    vs: Option<PathBuf>,
    fs: Option<PathBuf>,
    dt: Option<PathBuf>,
    nt: Option<PathBuf>,
    cache_size: [u32; 2],
    font: Option<PathBuf>,
    text_scale: f32,
    text_width: u32,
    text: Option<String>,
    _b: PhantomData<B>,
}

impl<B: BackendTrait> Default for RenderableBuilder<B> {
    fn default() -> Self {
        RenderableBuilder {
            mesh: None,
            vs: None,
            fs: None,
            dt: None,
            nt: None,
            cache_size: [512; 2],
            font: None,
            text_scale: 16.0,
            text_width: 100,
            text: None,
            _b: PhantomData::default(),
        }
    }
}

impl<B: BackendTrait> RenderableBuilder<B> {
    pub fn mesh<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.mesh = Some(path.as_ref().into());
        self
    }

    pub fn vertex_shader<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.vs = Some(path.as_ref().into());
        self
    }

    pub fn fragment_shader<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.fs = Some(path.as_ref().into());
        self
    }

    pub fn diffuse_texture<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.dt = Some(path.as_ref().into());
        self
    }

    pub fn normal_texture<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.nt = Some(path.as_ref().into());
        self
    }

    pub fn cache_size(mut self, width: u32, height: u32) -> Self {
        self.cache_size = [width, height];
        self
    }

    pub fn font<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.font = Some(path.as_ref().into());
        self
    }

    pub fn text_scale(mut self, scale: f32) -> Self {
        self.text_scale = scale;
        self
    }

    pub fn text_width(mut self, width: u32) -> Self {
        self.text_width = width;
        self
    }

    pub fn text(mut self, text: &str) -> Self {
        self.text = Some(text.into());
        self
    }
}

impl RenderableBuilder<HeadlessBackend> {
    pub fn build_mesh_headless(self, _backend: &HeadlessBackend) -> Result<Renderable<HeadlessBackend>, Error> {
        let mesh_path = self.mesh.ok_or(RenderableError::MissingMesh)?;
        let vs_path = self.vs.ok_or(RenderableError::MissingVertexShader)?;
        let fs_path = self.fs.ok_or(RenderableError::MissingFragmentShader)?;
        let dt_path = self.dt.ok_or(RenderableError::MissingDiffuseTexture)?;

        let _mesh = Mesh::from_path(&mesh_path)?;
        let _dt_image = Image::from_path(&dt_path)?;
        let _nt_image = if let Some(ref p) = self.nt {
            Some(Image::from_path(p)?)
        } else {
            None
        };
        let _vs = vs_path.read_to_string()?;
        let _fs = fs_path.read_to_string()?;

        Ok(Renderable {
            data: HeadlessRenderData::default(),
            _b: PhantomData::default(),
        })
    }

    pub fn build_text_headless(self, backend: &HeadlessBackend) -> Result<Renderable<HeadlessBackend>, Error> {
        let cache_size = self.cache_size;
        let font_path = self.font.ok_or(RenderableError::MissingFont)?;
        let text_scale = self.text_scale;
        let text_width = self.text_width;
        let text = self.text.ok_or(RenderableError::MissingText)?;
        let vs_path = self.vs.ok_or(RenderableError::MissingVertexShader)?;
        let fs_path = self.fs.ok_or(RenderableError::MissingFragmentShader)?;

        let diffuse_texture = HeadlessTexture::empty(backend, cache_size)?;

        let text: Text<HeadlessBackend> = Text::builder()
            .font(&font_path)
            .cache(diffuse_texture.clone())
            .scale(text_scale)
            .width(text_width)
            .layout(&text)?;

        let dimensions = backend.dimensions();
        let _mesh = text.mesh(dimensions);

        let _vs = vs_path.read_to_string()?;
        let _fs = fs_path.read_to_string()?;

        Ok(Renderable {
            data: HeadlessRenderData::default(),
            _b: PhantomData::default(),
        })
    }
}

impl RenderableBuilder<GliumBackend> {
    pub fn build_mesh_glium(self, backend: &GliumBackend) -> Result<Renderable<GliumBackend>, Error> {
        let mesh_path = self.mesh.ok_or(RenderableError::MissingMesh)?;
        let vs_path = self.vs.ok_or(RenderableError::MissingVertexShader)?;
        let fs_path = self.fs.ok_or(RenderableError::MissingFragmentShader)?;
        let dt_path = self.dt.ok_or(RenderableError::MissingDiffuseTexture)?;

        let mesh = Mesh::from_path(&mesh_path)?;
        let dt_image = Image::from_path(&dt_path)?;
        let vs = vs_path.read_to_string()?;
        let fs = fs_path.read_to_string()?;

        let vertices = VertexBuffer::new(&backend.display, &mesh.vertices)?;
        let indices = IndexBuffer::new(&backend.display, PrimitiveType::TrianglesList, &mesh.indices)?;
        let program = Program::from_source(&backend.display, &vs, &fs, None)?;
        let diffuse_texture = GliumTexture::from_image(backend, dt_image)?;
        let normal_texture = if let Some(ref p) = self.nt {
            let nt_image = Image::from_path(p)?;
            Some(GliumTexture::from_image(backend, nt_image)?)
        } else {
            None
        };

        Ok(Renderable {
            data: GliumRenderData {
                vertices,
                indices,
                program,
                diffuse_texture,
                normal_texture,
            },
            _b: PhantomData::default(),
        })
    }

    pub fn build_text_glium(self, backend: &GliumBackend) -> Result<Renderable<GliumBackend>, Error> {
        let cache_size = self.cache_size;
        let font_path = self.font.ok_or(RenderableError::MissingFont)?;
        let text_scale = self.text_scale;
        let text_width = self.text_width;
        let text = self.text.ok_or(RenderableError::MissingText)?;
        let vs_path = self.vs.ok_or(RenderableError::MissingVertexShader)?;
        let fs_path = self.fs.ok_or(RenderableError::MissingFragmentShader)?;

        let diffuse_texture = GliumTexture::empty(backend, cache_size)?;

        let text: Text<GliumBackend> = Text::builder()
            .font(&font_path)
            .cache(diffuse_texture.clone())
            .scale(text_scale)
            .width(text_width)
            .layout(&text)?;

        let dimensions = backend.dimensions();
        let mesh = text.mesh(dimensions);
        let vertices = VertexBuffer::new(&backend.display, &mesh.vertices)?;
        let indices = IndexBuffer::new(&backend.display, PrimitiveType::TrianglesList, &mesh.indices)?;

        let vs = vs_path.read_to_string()?;
        let fs = fs_path.read_to_string()?;
        let program = Program::from_source(&backend.display, &vs, &fs, None)?;

        Ok(Renderable {
            data: GliumRenderData {
                vertices,
                indices,
                program,
                diffuse_texture: diffuse_texture,
                normal_texture: None,
            },
            _b: PhantomData::default(),
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
    use graphics::headless::HeadlessEventsLoop;

    #[test]
    fn headless_builder_mesh() {
        let b = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();
        let base_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
        let r: Result<Renderable<HeadlessBackend>, Error> = Renderable::builder()
            .mesh(&base_path.join("cube.ply"))
            .vertex_shader(&base_path.join("vertex.glsl"))
            .fragment_shader(&base_path.join("fragment.glsl"))
            .diffuse_texture(&base_path.join("tv-test-image.png"))
            .build_mesh_headless(&b);

        assert_ok!(r);
    }

    #[test]
    fn headless_builder_text() {
        let b = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();
        let base_path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"));
        let r: Result<Renderable<HeadlessBackend>, Error> = Renderable::builder()
            .font(&base_path.join("SourceSansPro-Regular.ttf"))
            .vertex_shader(&base_path.join("vertex.glsl"))
            .fragment_shader(&base_path.join("fragment.glsl"))
            .text("Hello, World!")
            .build_text_headless(&b);

        assert_ok!(r);
    }
}
