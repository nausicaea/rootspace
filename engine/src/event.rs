use ecs::EventTrait;
use glium::glutin::{Event as GlutinEvent, WindowEvent};
use graphics::{glium::GliumEvent, headless::HeadlessEvent};

bitflags! {
    pub struct EventFlag: u64 {
        const STARTUP = 0x01;
        const SHUTDOWN = 0x02;
        const HARD_SHUTDOWN = 0x04;
        const COMMAND = 0x08;
    }
}

impl Default for EventFlag {
    fn default() -> Self {
        EventFlag::all()
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    flag: EventFlag,
    data: EventData,
}

impl Event {
    pub fn startup() -> Self {
        Event {
            flag: EventFlag::STARTUP,
            data: EventData::Empty,
        }
    }

    pub fn shutdown() -> Self {
        Event {
            flag: EventFlag::SHUTDOWN,
            data: EventData::Empty,
        }
    }

    pub fn hard_shutdown() -> Self {
        Event {
            flag: EventFlag::HARD_SHUTDOWN,
            data: EventData::Empty,
        }
    }

    pub fn command(args: Vec<String>) -> Self {
        Event {
            flag: EventFlag::COMMAND,
            data: EventData::Command(args),
        }
    }

    pub fn flag(&self) -> EventFlag {
        self.flag
    }

    pub fn data(&self) -> &EventData {
        &self.data
    }
}

impl EventTrait for Event {
    type EventFlag = EventFlag;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool {
        flag.contains(self.flag)
    }
}

impl From<HeadlessEvent> for Option<Event> {
    fn from(_value: HeadlessEvent) -> Option<Event> {
        None
    }
}

impl From<GliumEvent> for Option<Event> {
    fn from(value: GliumEvent) -> Option<Event> {
        if let GliumEvent(GlutinEvent::WindowEvent { event: we, .. }) = value {
            match we {
                WindowEvent::CloseRequested => Some(Event::shutdown()),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventData {
    Empty,
    Command(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_event_flag() {
        assert_eq!(EventFlag::default(), EventFlag::all());
    }

    #[test]
    fn accessors() {
        let e = Event::startup();
        let _: EventFlag = e.flag();
        let _: &EventData = e.data();
    }

    #[test]
    fn ready_event() {
        let e = Event::startup();
        assert_eq!(e.flag, EventFlag::STARTUP);
        assert_eq!(e.data, EventData::Empty);
    }

    #[test]
    fn shutdown_event() {
        let e = Event::shutdown();
        assert_eq!(e.flag, EventFlag::SHUTDOWN);
        assert_eq!(e.data, EventData::Empty);
    }

    #[test]
    fn hard_shutdown_event() {
        let e = Event::hard_shutdown();
        assert_eq!(e.flag, EventFlag::HARD_SHUTDOWN);
        assert_eq!(e.data, EventData::Empty);
    }

    #[test]
    fn command_event() {
        let e = Event::command(Vec::new());
        assert_eq!(e.flag, EventFlag::COMMAND);
        assert_eq!(e.data, EventData::Command(Vec::new()));
    }
}
