use graphics::headless::HeadlessEvent;
use graphics::glium::GliumEvent;
use ecs::event::EventTrait;
use std::convert::TryFrom;

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

#[derive(Clone, Debug)]
pub struct Event {
    flag: EventFlag,
}

impl Event {
    pub fn ready() -> Self {
        Event {
            flag: EventFlag::READY,
        }
    }
}

impl EventTrait for Event {
    type EventFlag = EventFlag;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool {
        flag.contains(self.flag)
    }
}

impl TryFrom<HeadlessEvent> for Event {
    type Error = ();

    fn try_from(_value: HeadlessEvent) -> Result<Event, ()> {
        Err(())
    }
}

impl TryFrom<GliumEvent> for Event {
    type Error = ();

    fn try_from(value: GliumEvent) -> Result<Event, ()> {
        if let GliumEvent::WindowEvent { event: _we, .. } = value {
            unimplemented!()
        } else {
            Err(())
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
        assert!(Event::ready().matches_filter(EventFlag::READY));
    }
}
