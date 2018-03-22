use ecs::event::EventTrait;

#[derive(Clone)]
pub enum Event {
    UnspecifiedEvent,
}

impl Event {
    fn as_flag(&self) -> EventFlag {
        match *self {
            Event::UnspecifiedEvent => EventFlag::UNSPECIFIED_EVENT,
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct EventFlag: u64 {
        const UNSPECIFIED_EVENT = 0x01;
    }
}

impl EventTrait for Event {
    type EventFlag = EventFlag;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool {
        flag.contains(self.as_flag())
    }
}
