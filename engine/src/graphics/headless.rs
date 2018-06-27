use event::Event;
use failure::Error as FailureError;
use std::convert::TryFrom;
use super::{EventsLoopTrait, FrameTrait, DisplayTrait};

#[derive(Default)]
pub struct HeadlessEventsLoop;

impl EventsLoopTrait<Event> for HeadlessEventsLoop {
    type OsEvent = ();

    fn poll<F>(&mut self, _f: F)
    where
        F: FnMut(Self::OsEvent),
    {
    }
}

impl TryFrom<()> for Event {
    type Error = ();

    fn try_from(_value: ()) -> Result<Event, Self::Error> {
        Err(())
    }
}

#[derive(Default)]
pub struct HeadlessFrame;

impl HeadlessFrame {
    pub fn draw(&mut self) -> Result<(), FailureError> {
        Ok(())
    }
}

impl FrameTrait for HeadlessFrame {
    fn clear(&mut self, _color: &[f32; 4], _depth: f32) {}

    fn finalize(self) -> Result<(), FailureError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct HeadlessDisplay;

impl DisplayTrait for HeadlessDisplay {
    type EventsLoop = HeadlessEventsLoop;
    type Frame = HeadlessFrame;

    fn create(
        _events_loop: &Self::EventsLoop,
        _title: &str,
        _dimensions: [u32; 2],
        _vsync: bool,
        _msaa: u16,
    ) -> Result<Self, FailureError> {
        Ok(HeadlessDisplay::default())
    }

    fn create_frame(&self) -> Self::Frame {
        HeadlessFrame::default()
    }
}

