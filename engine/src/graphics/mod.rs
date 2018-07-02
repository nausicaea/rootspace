pub mod glium;
pub mod headless;

use ecs::event::EventTrait;
use failure::Error;
use std::convert::TryInto;

pub trait BackendTrait<E, F>
where
Self: Sized,
{
    fn new(events_loop: &E, title: &str, dimensions: [u32; 2], vsync: bool, msaa: u16) -> Result<Self, Error>;
    fn create_frame(&self) -> F;
}

pub trait FrameTrait<R> {
    fn initialize(&mut self, color: [f32; 4], depth: f32);
    fn render<L: AsRef<[[f32; 4]; 4]>>(&mut self, location: &L, data: &R) -> Result<(), Error>;
    fn finalize(self) -> Result<(), Error>;
}

pub trait RenderDataTrait<B>
where
Self: Sized,
{
    fn triangle(backend: &B) -> Result<Self, Error>;
}

pub trait EventsLoopTrait<E>
where
E: EventTrait,
{
    type OsEvent: TryInto<E>;

    fn poll<F>(&mut self, f: F)
        where
            F: FnMut(Self::OsEvent);
}
