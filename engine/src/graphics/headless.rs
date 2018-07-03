use super::{BackendTrait, EventsLoopTrait, FrameTrait, RenderDataTrait};
use event::Event;
use failure::Error;

pub struct HeadlessEvent;

#[derive(Debug, Clone, Default)]
pub struct HeadlessEventsLoop;

impl EventsLoopTrait<Event, HeadlessEvent> for HeadlessEventsLoop {
    fn poll<F: FnMut(HeadlessEvent)>(&mut self, _f: F) {}
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessRenderData;

impl RenderDataTrait<HeadlessBackend> for HeadlessRenderData {
    fn triangle(_backend: &HeadlessBackend) -> Result<Self, Error> {
        Ok(HeadlessRenderData::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessFrame;

impl FrameTrait<HeadlessRenderData> for HeadlessFrame {
    fn initialize(&mut self, _color: [f32; 4], _depth: f32) {}

    fn render<L: AsRef<[[f32; 4]; 4]>>(
        &mut self,
        _location: &L,
        _data: &HeadlessRenderData,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn finalize(self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessBackend;

impl BackendTrait<HeadlessEventsLoop, HeadlessFrame> for HeadlessBackend {
    fn new(
        _events_loop: &HeadlessEventsLoop,
        _title: &str,
        _dimensions: [u32; 2],
        _vsync: bool,
        _msaa: u16,
    ) -> Result<Self, Error> {
        Ok(HeadlessBackend::default())
    }

    fn create_frame(&self) -> HeadlessFrame {
        HeadlessFrame::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Default)]
    struct MockLocation([[f32; 4]; 4]);

    impl AsRef<[[f32; 4]; 4]> for MockLocation {
        fn as_ref(&self) -> &[[f32; 4]; 4] {
            &self.0
        }
    }

    #[test]
    fn backend() {
        assert_ok!(HeadlessBackend::new(
            &HeadlessEventsLoop::default(),
            "Title",
            [800, 600],
            false,
            0
        ));
    }

    #[test]
    fn render_data() {
        let b = HeadlessBackend::new(
            &HeadlessEventsLoop::default(),
            "Title",
            [800, 600],
            false,
            0,
        ).unwrap();

        assert_ok!(HeadlessRenderData::triangle(&b));
    }

    #[test]
    fn frame() {
        let b = HeadlessBackend::new(
            &HeadlessEventsLoop::default(),
            "Title",
            [800, 600],
            false,
            0,
        ).unwrap();

        let mut f: HeadlessFrame = b.create_frame();
        f.initialize([1.0, 0.0, 0.5, 1.0], 1.0);
        assert_ok!(f.render(&MockLocation::default(), &HeadlessRenderData::default()));
        let r = f.finalize();
        assert_ok!(r);
    }
}
