use ecs::{EventManagerTrait, EventHandlerSystem};
use crate::event::EngineEventTrait;
use std::marker::PhantomData;

pub struct EventCoordinator<Ctx, Evt> {
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
}

impl<Ctx, Evt> Default for EventCoordinator<Ctx, Evt> {
    fn default() -> Self {
        EventCoordinator {
            _ctx: PhantomData::default(),
            _evt: PhantomData::default(),
        }
    }
}

impl<Ctx, Evt> EventHandlerSystem<Ctx, Evt> for EventCoordinator<Ctx, Evt>
where
    Ctx: EventManagerTrait<Evt>,
    Evt: EngineEventTrait,
{
    fn get_event_filter(&self) -> Evt::EventFlag {
        Evt::shutdown() | Evt::hard_shutdown()
    }

    fn run(&mut self, ctx: &mut Ctx, event: &Evt) -> bool {
        if event.matches_filter(Evt::shutdown()) {
            ctx.dispatch_later(Evt::new_hard_shutdown());
            true
        } else if event.matches_filter(Evt::hard_shutdown()) {
            false
        } else {
            true
        }
    }
}
