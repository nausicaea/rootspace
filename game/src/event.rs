use ecs::event::EventTrait;

#[derive(Clone)]
pub enum Event {
    TestEvent,
    AnotherTestEvent(u8),
}

impl Event {
    fn as_flag(&self) -> EventFlag {
        match *self {
            Event::TestEvent => EventFlag::TEST_EVENT,
            Event::AnotherTestEvent(_) => EventFlag::ANOTHER_TEST_EVENT,
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct EventFlag: u64 {
        const TEST_EVENT = 0x01;
        const ANOTHER_TEST_EVENT = 0x02;
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
    fn flag_conversion() {
        assert_eq!(Event::TestEvent.as_flag(), EventFlag::TEST_EVENT);
    }
    #[test]
    fn filter_matching() {
        assert!(Event::TestEvent.matches_filter(EventFlag::TEST_EVENT | EventFlag::ANOTHER_TEST_EVENT));
        assert!(!Event::TestEvent.matches_filter(EventFlag::ANOTHER_TEST_EVENT));
    }
}
