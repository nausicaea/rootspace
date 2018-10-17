pub mod glium;
pub mod headless;

use ecs::EventTrait;
use failure::Error;
use std::{borrow::Borrow, convert::TryInto};

pub trait BackendTrait<E, F>
where
    Self: Sized,
{
    fn new(events_loop: &E, title: &str, dimensions: [u32; 2], vsync: bool, msaa: u16) -> Result<Self, Error>;
    fn create_frame(&self) -> F;
    fn dpi_factor(&self) -> f64;
}

pub trait FrameTrait<D> {
    fn initialize(&mut self, color: [f32; 4], depth: f32);
    fn render<T: AsRef<[[f32; 4]; 4]>, R: Borrow<D>>(&mut self, transform: &T, data: &R) -> Result<(), Error>;
    fn finalize(self) -> Result<(), Error>;
}

pub trait EventsLoopTrait<O, I>
where
    O: EventTrait,
    I: TryInto<O>,
{
    fn poll<F: FnMut(I)>(&mut self, f: F);
}
