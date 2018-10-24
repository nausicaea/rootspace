pub mod glium;
pub mod headless;

use ecs::EventTrait;
use failure::Error;
use std::borrow::{Borrow, Cow};

pub trait BackendTrait: Sized {
    type Loop;
    type Frame: FrameTrait;

    fn new(events_loop: &Self::Loop, title: &str, dimensions: [u32; 2], vsync: bool, msaa: u16) -> Result<Self, Error>;
    fn create_frame(&self) -> Self::Frame;
    fn dpi_factor(&self) -> f64;
}

pub trait FrameTrait {
    type Data;

    fn initialize(&mut self, color: [f32; 4], depth: f32);
    fn render<T: AsRef<[[f32; 4]; 4]>, R: Borrow<Self::Data>>(&mut self, transform: &T, data: &R) -> Result<(), Error>;
    fn finalize(self) -> Result<(), Error>;
}

pub trait EventsLoopTrait<O, I>
where
    O: EventTrait,
    I: Into<Option<O>>,
{
    fn poll<F: FnMut(I)>(&mut self, f: F);
}

pub trait TextureTrait: Sized {
    type Backend;

    fn empty(backend: &Self::Backend, width: u32, height: u32) -> Result<Self, Error>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn write<'a>(&self, x: u32, y: u32, width: u32, height: u32, data: Cow<'a, [u8]>);
}
