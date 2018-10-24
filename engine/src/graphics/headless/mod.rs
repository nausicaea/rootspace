use super::{BackendTrait, EventsLoopTrait, FrameTrait, TextureTrait};
use event::Event;
use failure::Error;
use std::borrow::{Borrow, Cow};

#[derive(Debug, Clone, Default, Copy)]
pub struct HeadlessEvent;

#[derive(Debug, Clone, Default)]
pub struct HeadlessEventsLoop;

impl EventsLoopTrait<Event, HeadlessEvent> for HeadlessEventsLoop {
    fn poll<F: FnMut(HeadlessEvent)>(&mut self, _f: F) {}
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessTexture {
    width: u32,
    height: u32,
}

impl TextureTrait for HeadlessTexture {
    type Backend = HeadlessBackend;

    fn empty(_backend: &HeadlessBackend, width: u32, height: u32) -> Result<Self, Error> {
        Ok(HeadlessTexture { width, height })
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn write<'a>(&self, _x: u32, _y: u32, _width: u32, _height: u32, _data: Cow<'a, [u8]>) {}
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessRenderData;

impl HeadlessRenderData {
    pub fn new(_backend: &HeadlessBackend) -> Result<Self, Error> {
        Ok(HeadlessRenderData::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessFrame;

impl FrameTrait for HeadlessFrame {
    type Data = HeadlessRenderData;

    fn initialize(&mut self, _color: [f32; 4], _depth: f32) {}

    fn render<T: AsRef<[[f32; 4]; 4]>, R: Borrow<HeadlessRenderData>>(
        &mut self,
        _transform: &T,
        _data: &R,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn finalize(self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessBackend;

impl BackendTrait for HeadlessBackend {
    type Loop = HeadlessEventsLoop;
    type Frame = HeadlessFrame;

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

    fn dpi_factor(&self) -> f64 {
        1.0
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
    fn data() {
        let _ = HeadlessRenderData::default();

        let b = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();

        assert_ok!(HeadlessRenderData::new(&b));
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
    fn dpi_factor() {
        let b = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();

        assert_eq!(b.dpi_factor(), 1.0f64);
    }

    #[test]
    fn frame() {
        let b = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();

        let mut f: HeadlessFrame = b.create_frame();
        f.initialize([1.0, 0.0, 0.5, 1.0], 1.0);
        assert_ok!(f.render(&MockLocation::default(), &HeadlessRenderData::default()));
        let r = f.finalize();
        assert_ok!(r);
    }
}
