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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_event_flag() {
        assert_eq!(EventFlag::default(), EventFlag::all());
    }
}
