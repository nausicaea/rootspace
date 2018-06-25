use super::model::MockModel;
use components::renderable::RenderTrait;
use failure::Error as FailureError;
use std::sync::RwLock;
use wrappers::glium::FrameTrait;

#[derive(Default)]
pub struct MockRenderable {
    pub dc: RwLock<usize>,
}

impl MockRenderable {
    pub fn draw_calls(&self) -> usize {
        *self.dc.read().unwrap()
    }
}

impl RenderTrait for MockRenderable {
    type Model = MockModel;

    fn draw<F: FrameTrait>(&self, _target: &mut F, _model: &MockModel) -> Result<(), FailureError> {
        let mut calls = self.dc.write().unwrap();
        *calls += 1;

        Ok(())
    }
}
