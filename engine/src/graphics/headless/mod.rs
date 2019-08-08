use super::{
    private::Sealed, BackendTrait, DataTrait, EventTrait, EventsLoopTrait, FrameTrait, IndexBufferTrait, ShaderTrait,
    TextureTrait, VertexBufferTrait,
};
use crate::{
    assets::{Image, Mesh, Vertex},
    event::EngineEvent,
    geometry::rect::Rect,
};
use failure::Error;
use std::borrow::{Borrow, Cow};
use std::convert::TryInto;

#[derive(Debug, Clone, Default, Copy)]
pub struct HeadlessEvent;

impl Sealed for HeadlessEvent {}

impl EventTrait for HeadlessEvent {}

impl TryInto<EngineEvent> for HeadlessEvent {
    type Error = ();

    fn try_into(self) -> Result<EngineEvent, Self::Error> {
        Err(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessEventsLoop;

impl Sealed for HeadlessEventsLoop {}

impl EventsLoopTrait<HeadlessBackend> for HeadlessEventsLoop {
    fn poll<F: FnMut(HeadlessEvent)>(&mut self, _f: F) {}
}

#[derive(Debug, Clone, Default)]
pub struct HeadlessTexture {
    dimensions: (u32, u32),
}

impl Sealed for HeadlessTexture {}

impl TextureTrait<HeadlessBackend> for HeadlessTexture {
    fn empty(_backend: &HeadlessBackend, dimensions: (u32, u32)) -> Result<Self, Error> {
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

    fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    #[cfg_attr(not(test), allow(unused_variables))]
    fn write<'a, R: Into<Rect<u32>>>(&self, rect: R, _data: Cow<'a, [u8]>) {
        #[cfg(any(test, feature = "diagnostics"))]
        {
            let rect = rect.into();
            assert!(rect.max().x() < self.dimensions.0);
            assert!(rect.max().y() < self.dimensions.1);

            trace!("Wrote to the texture at {}", rect);
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeadlessShader;

impl Sealed for HeadlessShader {}

impl ShaderTrait<HeadlessBackend> for HeadlessShader {
    fn from_source<S: AsRef<str>>(_backend: &HeadlessBackend, _vs: S, _fs: S) -> Result<Self, Error> {
        Ok(HeadlessShader)
    }
}

#[derive(Debug, Clone)]
pub struct HeadlessVertexBuffer;

impl Sealed for HeadlessVertexBuffer {}

impl VertexBufferTrait<HeadlessBackend> for HeadlessVertexBuffer {
    fn from_vertices(_backend: &HeadlessBackend, _vertices: &[Vertex]) -> Result<Self, Error> {
        Ok(HeadlessVertexBuffer)
    }
}

#[derive(Debug, Clone)]
pub struct HeadlessIndexBuffer;

impl Sealed for HeadlessIndexBuffer {}

impl IndexBufferTrait<HeadlessBackend> for HeadlessIndexBuffer {
    fn from_indices(_backend: &HeadlessBackend, _indices: &[u16]) -> Result<Self, Error> {
        Ok(HeadlessIndexBuffer)
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

#[cfg_attr(feature = "diagnostics", derive(TypeName))]
#[derive(Debug, Clone, Default)]
pub struct HeadlessBackend {
    dimensions: (u32, u32),
}

impl Sealed for HeadlessBackend {}

impl BackendTrait for HeadlessBackend {
    type Data = HeadlessRenderData;
    type Event = HeadlessEvent;
    type EventsLoop = HeadlessEventsLoop;
    type Frame = HeadlessFrame;
    type IndexBuffer = HeadlessIndexBuffer;
    type Shader = HeadlessShader;
    type Texture = HeadlessTexture;
    type VertexBuffer = HeadlessVertexBuffer;

    #[allow(unused_variables)]
    fn new(
        _events_loop: &HeadlessEventsLoop,
        title: &str,
        dimensions: (u32, u32),
        _vsync: bool,
        _msaa: u16,
    ) -> Result<Self, Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        trace!("Created a headless backend (title='{}', dims={:?})", title, dimensions);

        Ok(HeadlessBackend { dimensions })
    }

    fn create_frame(&self) -> HeadlessFrame {
        HeadlessFrame::default()
    }

    fn dpi_factor(&self) -> f64 {
        1.0
    }

    fn physical_dimensions(&self) -> (u32, u32) {
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
        assert!(HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", (800, 600), false, 0).is_ok());
    }

    #[test]
    fn dpi_factor() {
        let b = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", (800, 600), false, 0).unwrap();

        assert_eq!(b.dpi_factor(), 1.0f64);
    }

    #[test]
    fn frame() {
        let b = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", (800, 600), false, 0).unwrap();

        let mut f: HeadlessFrame = b.create_frame();
        f.initialize([1.0, 0.0, 0.5, 1.0], 1.0);
        assert!(f
            .render(&MockLocation::default(), &HeadlessRenderData::default())
            .is_ok());
        let r = f.finalize();
        assert!(r.is_ok());
    }
}
