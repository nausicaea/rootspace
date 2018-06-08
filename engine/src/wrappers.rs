use std::convert::{TryInto, TryFrom};
use glium::glutin::{Event as WinitEvent, EventsLoop as WinitEventsLoop};
use glium::{Surface as GliumSurface, Frame as GliumFrame, Display as GliumDisplay, SwapBuffersError, backend::glutin::DisplayCreationError};
use ecs::event::EventTrait;
use event::Event;

pub trait EventsLoopTrait<E>
where
    E: EventTrait,
{
    type OsEvent: TryInto<E>;

    fn poll<F>(&mut self, f: F) where F: FnMut(Self::OsEvent);
}

impl TryFrom<WinitEvent> for Event {
    type Error = ();

    fn try_from(value: WinitEvent) -> Result<Event, Self::Error> {
        if let WinitEvent::WindowEvent { event: _we, .. } = value {
            unimplemented!()
        } else {
            Err(())
        }
    }
}

impl EventsLoopTrait<Event> for WinitEventsLoop {
    type OsEvent = WinitEvent;

    fn poll<F>(&mut self, f: F) where F: FnMut(Self::OsEvent) {
        self.poll_events(f)
    }
}

pub trait FrameTrait {
    type Error;

    fn clear(&mut self, color: &[f32; 4], depth: f32);
    fn finalize(self) -> Result<(), failure::Error>;
}

impl FrameTrait for GliumFrame {
    type Error = SwapBuffersError;

    fn clear(&mut self, color: &[f32; 4], depth: f32) {
        self.clear_color_and_depth((color[0], color[1], color[2], color[3]), depth)
    }

    fn finalize(self) -> Result<(), failure::Error> {
        self.finish()?
    }
}

pub trait DisplayTrait
where
    Self: Sized,
{
    type Error;
    type EventsLoop;
    type Frame: FrameTrait;

    fn create(events_loop: &Self::EventsLoop, title: &str, dimensions: &[u32; 2], vsync: bool, msaa: u16) -> Result<Self, Self::Error>;
    fn create_frame(&self) -> Self::Frame;
}

impl DisplayTrait for GliumDisplay {
    type Error = DisplayCreationError;
    type EventsLoop = WinitEventsLoop;
    type Frame = GliumFrame;

    fn create(events_loop: &Self::EventsLoop, title: &str, dimensions: &[u32; 2], vsync: bool, msaa: u16) -> Result<Self, Self::Error> {
        unimplemented!()
    }

    fn create_frame(&self) -> Self::Frame {
        self.draw()
    }
}
