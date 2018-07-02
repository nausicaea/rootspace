use failure::Error;
use glium::Frame as GliumFrame;
use graphics::headless::HeadlessFrame;
use graphics::FrameTrait;

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

pub struct Renderable<D> {
    _data: D,
}

impl Renderable<HeadlessRenderData> {
    pub fn new() -> Self {
        Renderable {
            _data: HeadlessRenderData::default(),
        }
    }
}

impl Renderable<GliumRenderData> {
    pub fn new() -> Self {
        Renderable {
            _data: GliumRenderData::default(),
        }
    }
}

impl<M> RenderTrait<HeadlessFrame, M> for Renderable<HeadlessRenderData> {
    fn render(&self, target: &mut HeadlessFrame, _model: &M) -> Result<(), Error> {
        target.draw()
    }
}

impl<M> RenderTrait<GliumFrame, M> for Renderable<GliumRenderData> {
    fn render(&self, _target: &mut GliumFrame, _model: &M) -> Result<(), Error> {
        // if let Err(e) = target.draw(vertices, indices, program, uniforms, &Default::default()) {
        //     Err(Into::into(e))
        // } else {
        //     Ok(())
        // }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock::model::MockModel;

    #[test]
    fn headless() {
        let r = Renderable::<HeadlessRenderData>::new();

        let model = MockModel::default();
        let mut frame = HeadlessFrame::default();

        assert_ok!(r.render(&mut frame, &model));
        assert_eq!(frame.draw_calls, 1);

        let r = frame.finalize();
        assert_ok!(r);
    }
}
