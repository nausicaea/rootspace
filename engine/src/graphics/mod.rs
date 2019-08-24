pub mod glium;
pub mod headless;

mod private {
    pub trait Sealed {}
}

use crate::{
    assets::{AssetTrait, Image, Vertex},
    components::Renderable,
    event::EngineEvent,
    file_manipulation::ReadPath,
    geometry::rect::Rect,
    resources::Backend,
};
use failure::Error;
use std::{borrow::Cow, convert::TryInto, path::Path};
#[cfg(feature = "diagnostics")]
use typename::TypeName;

#[cfg(not(feature = "diagnostics"))]
pub trait BackendTrait: Sized + private::Sealed + 'static {
    type Event: EventTrait;
    type Frame: FrameTrait<Self>;
    type EventsLoop: EventsLoopTrait<Self>;
    type Texture: TextureTrait<Self>;
    type Shader: ShaderTrait<Self>;
    type VertexBuffer: VertexBufferTrait<Self>;
    type IndexBuffer: IndexBufferTrait<Self>;

    fn new(
        events_loop: &Self::EventsLoop,
        title: &str,
        dimensions: (u32, u32),
        vsync: bool,
        msaa: u16,
    ) -> Result<Self, Error>;
    fn create_frame(&self) -> Self::Frame;
    fn dpi_factor(&self) -> f64;
    fn physical_dimensions(&self) -> (u32, u32);
}

#[cfg(feature = "diagnostics")]
pub trait BackendTrait: Sized + private::Sealed + 'static + TypeName {
    type Event: EventTrait;
    type Frame: FrameTrait<Self>;
    type EventsLoop: EventsLoopTrait<Self>;
    type Texture: TextureTrait<Self>;
    type Shader: ShaderTrait<Self>;
    type VertexBuffer: VertexBufferTrait<Self>;
    type IndexBuffer: IndexBufferTrait<Self>;

    fn new(
        events_loop: &Self::EventsLoop,
        title: &str,
        dimensions: (u32, u32),
        vsync: bool,
        msaa: u16,
    ) -> Result<Self, Error>;
    fn create_frame(&self) -> Self::Frame;
    fn dpi_factor(&self) -> f64;
    fn physical_dimensions(&self) -> (u32, u32);
}

pub trait EventTrait: TryInto<EngineEvent> + private::Sealed {}

pub trait EventsLoopTrait<B: BackendTrait>: Default + private::Sealed + 'static {
    fn poll<F: FnMut(B::Event)>(&mut self, f: F);
}

pub trait FrameTrait<B: BackendTrait>: private::Sealed {
    fn initialize(&mut self, color: [f32; 4], depth: f32);
    fn render<T: AsRef<[[f32; 4]; 4]>>(
        &mut self,
        transform: &T,
        factory: &Backend<B>,
        data: &Renderable,
    ) -> Result<(), Error>;
    fn finalize(self) -> Result<(), Error>;
}

pub trait TextureTrait<B: BackendTrait>: Sized + private::Sealed {
    fn empty(backend: &B, dimensions: (u32, u32)) -> Result<Self, Error>;
    fn from_image(backend: &B, image: Image) -> Result<Self, Error>;
    fn dimensions(&self) -> (u32, u32);
    fn write<'a, R: Into<Rect<u32>>>(&self, rect: R, data: Cow<'a, [u8]>);
    fn from_path<P: AsRef<Path>>(backend: &B, image: P) -> Result<Self, Error> {
        let img = Image::from_path(image)?;

        Self::from_image(backend, img)
    }
}

pub trait ShaderTrait<B: BackendTrait>: Sized + private::Sealed {
    fn from_source<S: AsRef<str>>(backend: &B, vs: S, fs: S) -> Result<Self, Error>;
    fn from_paths<P: AsRef<Path>>(backend: &B, vs: P, fs: P) -> Result<Self, Error> {
        let v = vs.read_to_string()?;
        let f = fs.read_to_string()?;

        Self::from_source(backend, v, f)
    }
}

pub trait VertexBufferTrait<B: BackendTrait>: Sized + private::Sealed {
    fn from_vertices(backend: &B, vertices: &[Vertex]) -> Result<Self, Error>;
}

pub trait IndexBufferTrait<B: BackendTrait>: Sized + private::Sealed {
    fn from_indices(backend: &B, indices: &[u16]) -> Result<Self, Error>;
}
