use std::collections::VecDeque;
use failure::Error;
use ecs::event::{EventTrait, EventManagerTrait};

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

pub struct EventManager {
    events: VecDeque<Event>,
}

impl Default for EventManager {
    fn default() -> Self {
        EventManager {
            events: Default::default(),
        }
    }
}

impl EventManagerTrait<Event> for EventManager {
    fn dispatch_later(&mut self, event: Event) {
        self.events.push_back(event)
    }
    fn handle_events<F>(&mut self, mut handler: F) -> Result<bool, Error>
    where
        F: FnMut(&mut Self, &Event) -> Result<bool, Error>,
    {
        let tmp = self.events.iter().cloned().collect::<Vec<_>>();
        self.events.clear();

        for event in tmp {
            handler(self, &event)?;
        }

        Ok(true)
    }
}
