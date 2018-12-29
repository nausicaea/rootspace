use ecs::{EventManagerTrait, EventTrait, System};
use crate::event::MaybeInto;
use crate::graphics::EventsLoopTrait;
use std::{marker::PhantomData, time::Duration};

pub struct EventInterface<Ctx, Evt, L> {
    pub events_loop: L,
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
}

impl<Ctx, Evt, L> EventInterface<Ctx, Evt, L> {
    pub fn new(events_loop: L) -> Self {
        EventInterface {
            events_loop,
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
        }
    }
}

impl<Ctx, Evt, L> Default for EventInterface<Ctx, Evt, L>
where
    L: Default,
{
    fn default() -> Self {
        EventInterface::new(Default::default())
    }
}

impl<Ctx, Evt, L> System<Ctx> for EventInterface<Ctx, Evt, L>
where
    Ctx: EventManagerTrait<Evt>,
    L: EventsLoopTrait<Evt>,
    Evt: EventTrait,
{
    fn run(&mut self, ctx: &mut Ctx, _t: &Duration, _dt: &Duration) {
        self.events_loop.poll(|input_event| {
            if let Some(event) = input_event.maybe_into() {
                ctx.dispatch_later(event);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::graphics::headless::HeadlessEventsLoop;
    use crate::mock::MockEvt;

    #[test]
    fn new_headless() {
        let _: EventInterface<Context<MockEvt>, MockEvt, HeadlessEventsLoop> = EventInterface::default();
    }
}
