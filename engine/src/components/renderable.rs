use std::path::PathBuf;
use std::fmt;
use file_manipulation::ReadPath;
use std::borrow::Borrow;
use graphics::{BackendTrait, TextureTrait};
use graphics::glium::{GliumBackend, GliumRenderData, GliumTexture};
use graphics::headless::{HeadlessBackend, HeadlessRenderData};
use resources::Text;
use std::marker::PhantomData;
use glium::{vertex::VertexBuffer, index::{IndexBuffer, PrimitiveType}, program::Program};

pub struct Renderable<B: BackendTrait> {
    data: B::Data,
    _b: PhantomData<B>,
}

impl Renderable<GliumBackend> {
    pub fn new(backend: &GliumBackend) -> Renderable<GliumBackend> {
        let diffuse_texture = GliumTexture::empty(backend, 512, 512)
            .unwrap();

        let text: Text<GliumBackend> = Text::builder()
            .font(&PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf")))
            .cache(diffuse_texture.clone())
            .scale(14.0)
            .width(100)
            .layout("Hello, World!")
            .unwrap();

        let dimensions = backend.dimensions();
        let mesh = text.mesh(dimensions);
        let vertices = VertexBuffer::new(&backend.display, &mesh.vertices).unwrap();
        let indices = IndexBuffer::new(&backend.display, PrimitiveType::TrianglesList, &mesh.indices).unwrap();

        let vs = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/vertex.glsl"))
            .read_to_string()
            .unwrap();
        let fs = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fragment.glsl"))
            .read_to_string()
            .unwrap();
        let program = Program::from_source(
            &backend.display,
            &vs,
            &fs,
            None,
        ).unwrap();

        Renderable {
            data: GliumRenderData {
                vertices,
                indices,
                program,
                diffuse_texture: diffuse_texture,
                normal_texture: None,
            },
            _b: PhantomData::default(),
        }
    }
}

impl<B: BackendTrait> fmt::Debug for Renderable<B> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Renderable")
    }
}

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
