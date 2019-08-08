use crate::{
    event::EngineEvent,
    graphics::{BackendTrait, EventsLoopTrait},
};
use ecs::{EventQueue, Resources, System};
use std::{convert::TryInto, marker::PhantomData, time::Duration};

pub struct EventInterface<B: BackendTrait> {
    pub events_loop: B::EventsLoop,
    _b: PhantomData<B>,
}

impl<B> Default for EventInterface<B>
where
    B: BackendTrait,
{
    fn default() -> Self {
        EventInterface {
            events_loop: B::EventsLoop::default(),
            _b: PhantomData::default(),
        }
    }
}

impl<B> System for EventInterface<B>
where
    B: BackendTrait,
{
    fn name(&self) -> &'static str {
        "EventInterface"
    }

    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        self.events_loop.poll(|input_event: B::Event| {
            if let Ok(event) = input_event.try_into() {
                res.borrow_mut::<EventQueue<EngineEvent>>().send(event);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::headless::HeadlessBackend;

    #[test]
    fn new_headless() {
        let _: EventInterface<HeadlessBackend> = EventInterface::default();
    }
}
