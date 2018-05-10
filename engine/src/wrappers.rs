use std::convert::{TryInto, TryFrom};
use winit::{Event as WinitEvent, EventsLoop as WinitEventsLoop};
use glium::{Frame as GliumFrame, Display as GliumDisplay, SwapBuffersError};
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

    fn finalize(self) -> Result<(), Self::Error>;
}

impl FrameTrait for GliumFrame {
    type Error = SwapBuffersError;

    fn finalize(self) -> Result<(), Self::Error> {
        self.finish()
    }
}

pub trait DisplayTrait {
    type Frame: FrameTrait;

    fn create_frame(&self) -> Self::Frame;
}

impl DisplayTrait for GliumDisplay {
    type Frame = GliumFrame;

    fn create_frame(&self) -> Self::Frame {
        self.draw()
    }
}
