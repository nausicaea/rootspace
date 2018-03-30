use std::collections::VecDeque;
use failure::Error;
use ecs::event::{EventTrait, EventManagerTrait};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_event_flag() {
        assert_eq!(EventFlag::default(), EventFlag::all());
    }

    quickcheck! {
        fn event_manager(num_events: usize) -> bool {
            let mut mgr = EventManager::default();

            for _ in 0..num_events {
                mgr.dispatch_later(Event::Ready);
            }

            let mut call_count = 0;
            let running = mgr.handle_events(|_, _| {
                call_count += 1;
                Ok(true)
            }).unwrap();

            running && call_count == num_events
        }
    }
}
