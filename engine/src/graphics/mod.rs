pub mod glium;
pub mod headless;

mod private {
    pub trait Sealed {}
}

use ecs::EventTrait;
use failure::Error;
use resources::Image;
use geometry::Rect;
use std::borrow::{Borrow, Cow};

pub trait BackendTrait: Sized + private::Sealed {
    type Loop;
    type Data: DataTrait;
    type Frame: FrameTrait<Self>;
    type Texture: TextureTrait<Self>;

    fn new(events_loop: &Self::Loop, title: &str, dimensions: [u32; 2], vsync: bool, msaa: u16) -> Result<Self, Error>;
    fn create_frame(&self) -> Self::Frame;
    fn dpi_factor(&self) -> f64;
    fn dimensions(&self) -> [u32; 2];
}

pub trait EventsLoopTrait<O: EventTrait>: private::Sealed {
    type InputEvent: Into<Option<O>>;

    fn poll<F: FnMut(Self::InputEvent)>(&mut self, f: F);
}

pub trait DataTrait: private::Sealed {}

pub trait FrameTrait<B: BackendTrait>: private::Sealed {
    fn initialize(&mut self, color: [f32; 4], depth: f32);
    fn render<T: AsRef<[[f32; 4]; 4]>, R: Borrow<B::Data>>(&mut self, transform: &T, data: &R) -> Result<(), Error>;
    fn finalize(self) -> Result<(), Error>;
}

pub trait TextureTrait<B: BackendTrait>: Sized + private::Sealed {
    fn empty(backend: &B, dimensions: [u32; 2]) -> Result<Self, Error>;
    fn from_image(backend: &B, image: Image) -> Result<Self, Error>;
    fn dimensions(&self) -> [u32; 2];
    fn write<'a, R: Into<Rect<u32>>>(&self, rect: R, data: Cow<'a, [u8]>);
}
