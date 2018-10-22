use ecs::EventTrait;
use glium::glutin::{Event as GlutinEvent, WindowEvent};
use graphics::{glium::GliumEvent, headless::HeadlessEvent};

bitflags! {
    pub struct EventFlag: u64 {
        const STARTUP = 0x01;
        const SHUTDOWN = 0x02;
        const HARD_SHUTDOWN = 0x04;
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
}

impl Event {
    pub fn startup() -> Self {
        Event {
            flag: EventFlag::STARTUP,
        }
    }

    pub fn shutdown() -> Self {
        Event {
            flag: EventFlag::SHUTDOWN,
        }
    }

    pub fn hard_shutdown() -> Self {
        Event {
            flag: EventFlag::HARD_SHUTDOWN,
        }
    }

    pub fn flag(&self) -> EventFlag {
        self.flag
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_event_flag() {
        assert_eq!(EventFlag::default(), EventFlag::all());
    }

    #[test]
    fn ready_event() {
        assert!(Event::startup().matches_filter(EventFlag::STARTUP));
    }

    #[test]
    fn shutdown_event() {
        assert!(Event::shutdown().matches_filter(EventFlag::SHUTDOWN));
    }

    #[test]
    fn hard_shutdown_event() {
        assert!(Event::hard_shutdown().matches_filter(EventFlag::HARD_SHUTDOWN));
    }
}
