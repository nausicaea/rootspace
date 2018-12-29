use ecs::{EventTrait, EventHandlerSystem};
use std::marker::PhantomData;

pub struct EventMonitor<Ctx, Evt> {
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
}

impl<Ctx, Evt> Default for EventMonitor<Ctx, Evt> {
    fn default() -> Self {
        EventMonitor {
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
        }
    }
}

impl<Ctx, Evt> EventHandlerSystem<Ctx, Evt> for EventMonitor<Ctx, Evt>
where
    Evt: EventTrait,
{
    fn get_event_filter(&self) -> Evt::EventFlag {
        Default::default()
    }

    fn run(&mut self, _ctx: &mut Ctx, event: &Evt) -> bool {
        trace!("Received {:?}", event);
        true
    }
}
