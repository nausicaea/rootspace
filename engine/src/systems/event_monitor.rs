use ecs::{EventHandlerSystem, EventTrait, Resources};
use std::marker::PhantomData;

pub struct EventMonitor<Evt> {
    _evt: PhantomData<Evt>,
}

impl<Evt> Default for EventMonitor<Evt> {
    fn default() -> Self {
        EventMonitor {
            _evt: PhantomData::default(),
        }
    }
}

impl<Evt> EventHandlerSystem<Evt> for EventMonitor<Evt>
where
    Evt: EventTrait,
{
    fn get_event_filter(&self) -> Evt::EventFlag {
        Default::default()
    }

    fn run(&mut self, _res: &mut Resources, event: &Evt) -> bool {
        trace!("Received {:?}", event);
        true
    }
}
