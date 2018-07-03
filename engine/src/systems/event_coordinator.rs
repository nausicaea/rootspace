use event::{Event, EventFlag};
use ecs::EventManagerTrait;
use ecs::LoopStage;
use ecs::SystemTrait;
use failure::Error;
use std::marker::PhantomData;

#[derive(Default)]
pub struct EventCoordinator<Ctx> {
    _ctx: PhantomData<Ctx>,
}

impl<Ctx> SystemTrait<Ctx, Event> for EventCoordinator<Ctx>
where
    Ctx: EventManagerTrait<Event>,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::HANDLE_EVENTS
    }

    fn get_event_filter(&self) -> EventFlag {
        EventFlag::SHUTDOWN | EventFlag::HARD_SHUTDOWN
    }

    fn handle_event(&mut self, ctx: &mut Ctx, event: &Event) -> Result<bool, Error> {
        match event.flag() {
            EventFlag::SHUTDOWN => {
                ctx.dispatch_later(Event::hard_shutdown());
                Ok(true)
            },
            EventFlag::HARD_SHUTDOWN => {
                Ok(false)
            },
            _ => Ok(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use components::model::Model;
    use mock::MockCtx;

    #[test]
    fn new() {
        let _: EventCoordinator<MockCtx<Event, Model>> = EventCoordinator::default();
    }

    #[test]
    fn get_stage_filter() {
        let c: EventCoordinator<MockCtx<Event, Model>> = EventCoordinator::default();

        assert_eq!(c.get_stage_filter(), LoopStage::HANDLE_EVENTS);
    }

    #[test]
    fn get_event_filter() {
        let c: EventCoordinator<MockCtx<Event, Model>> = EventCoordinator::default();

        assert_eq!(c.get_event_filter(), EventFlag::SHUTDOWN | EventFlag::HARD_SHUTDOWN);
    }

    #[test]
    fn handle_event() {
        let mut c: EventCoordinator<MockCtx<Event, Model>> = EventCoordinator::default();

        let r = c.handle_event(&mut Default::default(), &Event::startup());
        assert_ok!(r);
        assert!(r.unwrap());

        let r = c.handle_event(&mut Default::default(), &Event::shutdown());
        assert_ok!(r);
        assert!(r.unwrap());

        let r = c.handle_event(&mut Default::default(), &Event::hard_shutdown());
        assert_ok!(r);
        assert!(!r.unwrap());
    }
}
