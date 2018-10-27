use std::path::{PathBuf, Path};
use std::fmt;
use failure::Error;
use file_manipulation::ReadPath;
use std::borrow::Borrow;
use graphics::{BackendTrait, TextureTrait};
use graphics::glium::{GliumBackend, GliumRenderData, GliumTexture};
use graphics::headless::{HeadlessBackend, HeadlessRenderData};
use resources::{Text, Image, Mesh};
use std::marker::PhantomData;
use glium::{vertex::VertexBuffer, index::{IndexBuffer, PrimitiveType}, program::Program};

pub struct Renderable<B: BackendTrait> {
    data: B::Data,
    _b: PhantomData<B>,
}

impl<B: BackendTrait> fmt::Debug for Renderable<B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Renderable {{ ... }}")
    }
}

// impl<B: BackendTrait> Borrow<B::Data> for Renderable<B> {
//     fn borrow(&self) -> &B::Data {
//         &self.data
//     }
// }

impl Borrow<GliumRenderData> for Renderable<GliumBackend> {
    fn borrow(&self) -> &GliumRenderData {
        &self.data
    }
}

impl Borrow<HeadlessRenderData> for Renderable<HeadlessBackend> {
    fn borrow(&self) -> &HeadlessRenderData {
        &self.data
    }
}

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
    pub fn mesh(mut self, path: &Path) -> Self {
        self.mesh = Some(path.into());
        self
    }

    pub fn vertex_shader(mut self, path: &Path) -> Self {
        self.vs = Some(path.into());
        self
    }

    pub fn fragment_shader(mut self, path: &Path) -> Self {
        self.fs = Some(path.into());
        self
    }

    pub fn diffuse_texture(mut self, path: &Path) -> Self {
        self.dt = Some(path.into());
        self
    }

    pub fn normal_texture(mut self, path: &Path) -> Self {
        self.nt = Some(path.into());
        self
    }

    pub fn cache_size(mut self, width: u32, height: u32) -> Self {
        self.cache_size = [width, height];
        self
    }

    pub fn font(mut self, path: &Path) -> Self {
        self.font = Some(path.into());
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

impl RenderableBuilder<GliumBackend> {
    pub fn build_mesh(self, backend: &GliumBackend) -> Result<Renderable<GliumBackend>, Error> {
        let mesh_path = self.mesh
            .ok_or(RenderableError::MissingMesh)?;
        let vs_path = self.vs
            .ok_or(RenderableError::MissingVertexShader)?;
        let fs_path = self.fs
            .ok_or(RenderableError::MissingFragmentShader)?;
        let dt_path = self.dt
            .ok_or(RenderableError::MissingDiffuseTexture)?;

        let mesh = Mesh::from_path(&mesh_path)?;
        let dt_image = Image::from_path(&dt_path)?;

        let vertices = VertexBuffer::new(&backend.display, &mesh.vertices)?;
        let indices = IndexBuffer::new(&backend.display, PrimitiveType::TrianglesList, &mesh.indices)?;
        let vs = vs_path
            .read_to_string()?;
        let fs = fs_path
            .read_to_string()?;
        let program = Program::from_source(
            &backend.display,
            &vs,
            &fs,
            None,
        )?;
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

    pub fn build_text(self, backend: &GliumBackend) -> Result<Renderable<GliumBackend>, Error> {
        let cache_size = self.cache_size;
        let font_path = self.font
            .ok_or(RenderableError::MissingFont)?;
        let text_scale = self.text_scale;
        let text_width = self.text_width;
        let text = self.text
            .ok_or(RenderableError::MissingText)?;
        let vs_path = self.vs
            .ok_or(RenderableError::MissingVertexShader)?;
        let fs_path = self.fs
            .ok_or(RenderableError::MissingFragmentShader)?;

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

        let vs = vs_path
            .read_to_string()?;
        let fs = fs_path
            .read_to_string()?;
        let program = Program::from_source(
            &backend.display,
            &vs,
            &fs,
            None,
        )?;

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
