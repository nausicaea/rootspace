use ecs::{EventManagerTrait, EventTrait, LoopStage, SystemTrait};
use event::MaybeInto;
use failure::Error;
use graphics::{glium::GliumEventsLoop as GEL, headless::HeadlessEventsLoop as HEL, EventsLoopTrait};
use std::{marker::PhantomData, time::Duration};

pub type HeadlessEventInterface<Ctx, Evt> = EventInterface<Ctx, Evt, HEL>;
pub type GliumEventInterface<Ctx, Evt> = EventInterface<Ctx, Evt, GEL>;

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

impl<Ctx, Evt, L> SystemTrait<Ctx, Evt> for EventInterface<Ctx, Evt, L>
where
    Ctx: EventManagerTrait<Evt>,
    L: EventsLoopTrait<Evt>,
    Evt: EventTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::UPDATE
    }

    fn update(&mut self, ctx: &mut Ctx, _t: &Duration, _dt: &Duration) -> Result<(), Error> {
        self.events_loop.poll(|input_event| {
            if let Some(event) = input_event.maybe_into() {
                ctx.dispatch_later(event);
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context::Context;
    use mock::MockEvt;

    #[test]
    fn new_headless() {
        let _: HeadlessEventInterface<Context<MockEvt>, MockEvt> = EventInterface::default();
    }

    #[test]
    fn get_stage_filter_headless() {
        let e: HeadlessEventInterface<Context<MockEvt>, MockEvt> = EventInterface::default();

        assert_eq!(e.get_stage_filter(), LoopStage::UPDATE);
    }

    #[test]
    fn update_headless() {
        let mut e: HeadlessEventInterface<Context<MockEvt>, MockEvt> = EventInterface::default();
        let mut c = Context::default();

        assert_ok!(e.update(&mut c, &Default::default(), &Default::default()));
        // assert_eq!(c.dispatch_later_calls, 0);
        // assert!(c.events.is_empty());
    }
}
