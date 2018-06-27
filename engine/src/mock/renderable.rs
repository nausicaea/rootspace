use super::model::MockModel;
use components::renderable::RenderTrait;
use failure::Error as FailureError;
use std::sync::RwLock;
use graphics::headless::HeadlessFrame;

#[derive(Default)]
pub struct MockRenderable {
    rc: RwLock<usize>,
}

impl MockRenderable {
    pub fn render_calls(&self) -> usize {
        *self.rc.read().unwrap()
    }
}

impl RenderTrait<HeadlessFrame, MockModel> for MockRenderable {
    fn render(&self, _target: &mut HeadlessFrame, _model: &MockModel) -> Result<(), FailureError> {
        let mut calls = self.rc.write().unwrap();
        *calls += 1;

        Ok(())
    }
}
