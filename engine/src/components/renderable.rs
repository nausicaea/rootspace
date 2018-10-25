use std::path::PathBuf;
use std::borrow::Borrow;
use graphics::glium::{GliumBackend, GliumRenderData};
use resources::Text;

#[derive(Debug)]
pub struct Renderable<D> {
    data: D,
}

impl Renderable<GliumRenderData> {
    pub fn new(backend: &GliumBackend) -> Renderable<GliumRenderData> {
        let text = Text::builder()
            .font(&PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf")))
            .cache([512; 2])
            .scale(14.0)
            .width(100)
            .layout(backend, "Hello, World!")
            .unwrap();

        let dimensions = backend.dimensions();
        let mesh = text.mesh(dimensions);

        Renderable {
            data: GliumRenderData {
                vertices,
                indices,
                program,
                diffuse_texture,
                normal_texture,
            },
        }
    }
}

impl<D> Borrow<D> for Renderable<D> {
    fn borrow(&self) -> &D {
        &self.data
    }
}
