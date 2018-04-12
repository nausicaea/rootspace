use std::convert::{TryInto, TryFrom};
use winit::{Event as WinitEvent, EventsLoop as WinitEventsLoop};
use ecs::event::EventTrait;

#[derive(Clone, Debug)]
pub enum Event {
    Ready,
}

impl Event {
    fn as_flag(&self) -> EventFlag {
        match *self {
            Event::Ready => EventFlag::READY,
        }
    }
}

bitflags! {
    pub struct EventFlag: u64 {
        const READY = 0x01;
    }
}

impl Default for EventFlag {
    fn default() -> Self {
        EventFlag::all()
    }
}

impl EventTrait for Event {
    type EventFlag = EventFlag;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool {
        flag.contains(self.as_flag())
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_event_flag() {
        assert_eq!(EventFlag::default(), EventFlag::all());
    }
}
