use crate::graphics::EventsLoopTrait;
use ecs::{EventManager, EventTrait, Resources, System};
use std::{convert::TryInto, marker::PhantomData, time::Duration};

pub struct EventInterface<Evt, L> {
    pub events_loop: L,
    _evt: PhantomData<Evt>,
}

impl<Evt, L> EventInterface<Evt, L> {
    pub fn new(events_loop: L) -> Self {
        EventInterface {
            events_loop,
            _evt: PhantomData::default(),
        }
    }
}

impl<Evt, L> Default for EventInterface<Evt, L>
where
    L: Default,
{
    fn default() -> Self {
        EventInterface::new(Default::default())
    }
}

impl<Evt, L> System for EventInterface<Evt, L>
where
    L: EventsLoopTrait<Evt>,
    Evt: EventTrait,
{
    fn name(&self) -> &'static str {
        "EventInterface"
    }

    fn run(&mut self, res: &mut Resources, _t: &Duration, _dt: &Duration) {
        self.events_loop.poll(|input_event| {
            if let Ok(event) = input_event.try_into() {
                res.get_mut::<EventManager<Evt>>().dispatch_later(event);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graphics::headless::HeadlessEventsLoop, mock::MockEvt};

    #[test]
    fn new_headless() {
        let _: EventInterface<MockEvt, HeadlessEventsLoop> = EventInterface::default();
    }
}
