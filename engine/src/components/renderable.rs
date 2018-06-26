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

#[derive(Default)]
pub struct HeadlessRenderData;

#[derive(Default)]
pub struct GliumRenderData;

pub struct Renderable<D>
{
    data: D,
}

impl Renderable<HeadlessRenderData> {
    pub fn new() -> Self {
        Renderable {
            data: Default::default(),
        }
    }
}

impl Renderable<GliumRenderData> {
    pub fn new() -> Self {
        Renderable {
            data: Default::default(),
        }
    }
}

impl RenderTrait<HeadlessFrame, Model> for Renderable<HeadlessRenderData> {
    fn render(&self, target: &mut HeadlessFrame, _model: &Model) -> Result<(), Error> {
        target.draw()
    }
}

impl RenderTrait<GliumFrame, Model> for Renderable<GliumRenderData> {
    fn render(&self, _target: &mut GliumFrame, _model: &Model) -> Result<(), Error> {
        // if let Err(e) = target.draw(vertices, indices, program, uniforms, &Default::default()) {
        //     Err(Into::into(e))
        // } else {
        //     Ok(())
        // }
        Ok(())
    }
}
