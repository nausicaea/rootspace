use graphics::{headless::HeadlessRenderData, glium::GliumRenderData};

pub struct Renderable;

impl AsRef<HeadlessRenderData> for Renderable {
    fn as_ref(&self) -> &HeadlessRenderData {
        unimplemented!()
    }
}

impl AsRef<GliumRenderData> for Renderable {
    fn as_ref(&self) -> &GliumRenderData {
        unimplemented!()
    }
}
