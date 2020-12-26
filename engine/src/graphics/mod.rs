pub mod glium;
pub mod headless;
pub mod vertex;
pub mod text;

pub use self::vertex::Vertex;
pub use self::text::Text;

use crate::{
    assets::{AssetTrait, Image},
    components::Renderable,
    event::EngineEvent,
    file_manipulation::FilePathBuf,
    geometry::rect::Rect,
    resources::BackendResource,
};
use anyhow::Result;
use std::{borrow::Cow, convert::TryInto, path::Path};
use std::convert::TryFrom;

pub trait BackendTrait: Sized + 'static {
    type Event: EventTrait;
    type Frame: FrameTrait<Self>;
    type Texture: TextureTrait<Self>;
    type Shader: ShaderTrait<Self>;
    type VertexBuffer: VertexBufferTrait<Self>;
    type IndexBuffer: IndexBufferTrait<Self>;

    fn new<S: AsRef<str>>(title: S, dimensions: (u32, u32), vsync: bool, msaa: u16) -> Result<Self>;
    fn poll_events<F: FnMut(Self::Event)>(&mut self, f: F);
    fn create_frame(&self) -> Self::Frame;
    fn dpi_factor(&self) -> f64;
    fn physical_dimensions(&self) -> (u32, u32);
}

pub trait EventTrait: TryInto<EngineEvent> {}

pub trait FrameTrait<B: BackendTrait> {
    fn initialize(&mut self, color: [f32; 4], depth: f32);
    fn render<T: AsRef<[[f32; 4]; 4]>>(
        &mut self,
        transform: &T,
        factory: &BackendResource<B>,
        data: &Renderable,
    ) -> Result<()>;
    fn finalize(self) -> Result<()>;
}

pub trait TextureTrait<B: BackendTrait>: Sized {
    fn empty(backend: &B, dimensions: (u32, u32)) -> Result<Self>;
    fn from_image(backend: &B, image: Image) -> Result<Self>;
    fn dimensions(&self) -> (u32, u32);
    fn write<'a, R: Into<Rect<u32>>>(&self, rect: R, data: Cow<'a, [u8]>);
    fn from_path<P: AsRef<Path>>(backend: &B, image: P) -> Result<Self> {
        let img = Image::from_path(image)?;

        Self::from_image(backend, img)
    }
}

pub trait ShaderTrait<B: BackendTrait>: Sized {
    fn from_source<S: AsRef<str>>(backend: &B, vs: S, fs: S) -> Result<Self>;
    fn from_paths<P: AsRef<Path>>(backend: &B, vs: P, fs: P) -> Result<Self> {
        let vs = FilePathBuf::try_from(vs.as_ref())?;
        let v = vs.read_to_string()?;

        let fs = FilePathBuf::try_from(fs.as_ref())?;
        let f = fs.read_to_string()?;

        Self::from_source(backend, v, f)
    }
}

pub trait VertexBufferTrait<B: BackendTrait>: Sized {
    fn from_vertices(backend: &B, vertices: &[Vertex]) -> Result<Self>;
}

pub trait IndexBufferTrait<B: BackendTrait>: Sized {
    fn from_indices(backend: &B, indices: &[u16]) -> Result<Self>;
}
