use super::{DisplayTrait, EventsLoopTrait, FrameTrait};
use event::Event;
use failure::Error as FailureError;
use std::convert::TryFrom;

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
pub struct HeadlessFrame {
    pub draw_calls: usize,
}

impl HeadlessFrame {
    pub fn draw(&mut self) -> Result<(), FailureError> {
        self.draw_calls += 1;
        Ok(())
    }
}

impl FrameTrait for HeadlessFrame {
    fn clear_frame(&mut self, _color: [f32; 4], _depth: f32) {}

    fn finalize(self) -> Result<(), FailureError> {
        Ok(())
    }
}

#[derive(Default, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let r = HeadlessDisplay::create(&HeadlessEventsLoop::default(), "", [800, 600], false, 0);

        assert_ok!(r);

        let _f: HeadlessFrame = r.unwrap().create_frame();
    }

    #[test]
    fn frame() {
        let mut f = HeadlessFrame::default();

        f.clear_frame([0.0, 0.0, 0.0, 0.0], 0.0);

        assert_ok!(f.draw());

        let r = f.finalize();

        assert_ok!(r);
    }
}
