use super::{private::Sealed, BackendTrait, DataTrait, EventsLoopTrait, FrameTrait, TextureTrait};
use event::Event;
use failure::Error;
use geometry::Rect;
use resources::{Image, Mesh};
use std::borrow::{Borrow, Cow};

#[derive(Debug, Clone, Default, Copy)]
pub struct HeadlessEvent;

#[derive(Debug, Clone, Default)]
pub struct HeadlessEventsLoop;

impl Sealed for HeadlessEventsLoop {}

impl EventsLoopTrait<Event> for HeadlessEventsLoop {
    type InputEvent = HeadlessEvent;

    fn poll<F: FnMut(HeadlessEvent)>(&mut self, _f: F) {}
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessTexture {
    dimensions: [u32; 2],
}

impl Sealed for HeadlessTexture {}

impl TextureTrait<HeadlessBackend> for HeadlessTexture {
    fn empty(_backend: &HeadlessBackend, dimensions: [u32; 2]) -> Result<Self, Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Created an empty texture (dims={:?})", dimensions);

        Ok(HeadlessTexture { dimensions })
    }

    fn from_image(_backend: &HeadlessBackend, image: Image) -> Result<Self, Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Created a texture from an image (dims={:?})", image.dimensions());

        Ok(HeadlessTexture {
            dimensions: image.dimensions(),
        })
    }

    fn dimensions(&self) -> [u32; 2] {
        self.dimensions
    }

    fn write<'a, R: Into<Rect<u32>>>(&self, rect: R, _data: Cow<'a, [u8]>) {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Wrote to the texture at {}", rect.into());
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessRenderData;

impl HeadlessRenderData {
    #[allow(unused_variables)]
    pub fn new(_backend: &HeadlessBackend, mesh: &Mesh) -> Result<Self, Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!(
            "Created render data ({} vertices, {} triangles)",
            mesh.vertices.len(),
            mesh.indices.len() as f32 / 3.0
        );

        Ok(HeadlessRenderData::default())
    }
}

impl Sealed for HeadlessRenderData {}

impl DataTrait for HeadlessRenderData {}

#[derive(Debug, Clone, Default)]
pub struct HeadlessFrame;

impl Sealed for HeadlessFrame {}

impl FrameTrait<HeadlessBackend> for HeadlessFrame {
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
pub struct HeadlessBackend {
    dimensions: [u32; 2],
}

impl Sealed for HeadlessBackend {}

impl BackendTrait for HeadlessBackend {
    type Loop = HeadlessEventsLoop;
    type Data = HeadlessRenderData;
    type Frame = HeadlessFrame;
    type Texture = HeadlessTexture;

    #[allow(unused_variables)]
    fn new(
        _events_loop: &HeadlessEventsLoop,
        title: &str,
        dimensions: [u32; 2],
        _vsync: bool,
        _msaa: u16,
    ) -> Result<Self, Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Created a headless backend (title='{}', dims={:?})", title, dimensions);

        Ok(HeadlessBackend {
            dimensions
        })
    }

    fn create_frame(&self) -> HeadlessFrame {
        HeadlessFrame::default()
    }

    fn dpi_factor(&self) -> f64 {
        1.0
    }

    fn dimensions(&self) -> [u32; 2] {
        self.dimensions
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
