use components::model::Model;
use failure::Error;
use wrappers::glium::{FrameTrait, HeadlessFrame};
use glium::Frame as GliumFrame;

pub trait RenderTrait<F, M>
where
    F: FrameTrait,
{
    fn render(&self, target: &mut F, model: &M) -> Result<(), Error>;
}

pub struct Renderable;

impl Renderable {
    pub fn new() -> Self {
        Renderable { }
    }
}

impl RenderTrait<HeadlessFrame, Model> for Renderable {
    fn render(&self, _target: &mut HeadlessFrame, _model: &Model) -> Result<(), Error> {
        // target.render(vertices, indices, program, uniforms, params);
        Ok(())
    }
}

impl RenderTrait<GliumFrame, Model> for Renderable {
    fn render(&self, _target: &mut GliumFrame, _model: &Model) -> Result<(), Error> {
        // target.render(vertices, indices, program, uniforms, params);
        Ok(())
    }
}
