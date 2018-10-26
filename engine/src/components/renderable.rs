use std::path::PathBuf;
use std::fmt;
use std::borrow::Borrow;
use graphics::BackendTrait;
use graphics::glium::{GliumBackend, GliumRenderData};
use graphics::headless::{HeadlessBackend, HeadlessRenderData};
use resources::Text;
use std::marker::PhantomData;

pub struct Renderable<B: BackendTrait> {
    data: B::Data,
    _b: PhantomData<B>,
}

impl Renderable<GliumBackend> {
    pub fn new(backend: &GliumBackend) -> Renderable<GliumBackend> {
        let text = Text::builder()
            .font(&PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/SourceSansPro-Regular.ttf")))
            .cache([512; 2])
            .scale(14.0)
            .width(100)
            .layout(backend, "Hello, World!")
            .unwrap();

        let dimensions = backend.dimensions();
        let mesh = text.mesh(dimensions);

        unimplemented!()
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
