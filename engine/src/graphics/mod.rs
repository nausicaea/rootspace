pub mod glium;
pub mod headless;

use ecs::event::EventTrait;
use failure::Error as FailureError;
use nalgebra::Matrix4;
use std::convert::TryInto;

pub trait EventsLoopTrait<E>
where
    E: EventTrait,
{
    type OsEvent: TryInto<E>;

    fn poll<F>(&mut self, f: F)
    where
        F: FnMut(Self::OsEvent);
}

pub trait FrameTrait {
    fn clear_frame(&mut self, color: [f32; 4], depth: f32);
    fn finalize(self) -> Result<(), FailureError>;
}

pub trait DisplayTrait
where
    Self: Sized,
{
    type EventsLoop;
    type Frame: FrameTrait;

    fn create(
        events_loop: &Self::EventsLoop,
        title: &str,
        dimensions: [u32; 2],
        vsync: bool,
        msaa: u16,
    ) -> Result<Self, FailureError>;
    fn create_frame(&self) -> Self::Frame;
}

pub struct Uniforms {
    pub pvm_matrix: Matrix4<f32>,
}
