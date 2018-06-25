use components::model::Model;
use failure::Error;
use wrappers::glium::FrameTrait;

pub trait RenderTrait {
    type Model;

    fn draw<F: FrameTrait>(&self, target: &mut F, model: &Self::Model) -> Result<(), Error>;
}

pub struct Renderable;

impl RenderTrait for Renderable {
    type Model = Model;

    fn draw<F: FrameTrait>(&self, _target: &mut F, _model: &Model) -> Result<(), Error> {
        unimplemented!()
    }
}
