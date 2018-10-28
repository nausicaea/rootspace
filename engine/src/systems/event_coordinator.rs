use ecs::{EventManagerTrait, LoopStage, SystemTrait};
use event::{Event, EventFlag};
use failure::Error;
#[cfg(not(test))]
use ctrlc;
use std::marker::PhantomData;
#[cfg(not(test))]
use std::process;
use std::time::Duration;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

pub struct EventCoordinator<Ctx> {
    ctrlc_triggered: Arc<AtomicUsize>,
    _ctx: PhantomData<Ctx>,
}

impl<Ctx> Default for EventCoordinator<Ctx> {
    fn default() -> Self {
        let ctrlc_triggered = Arc::new(AtomicUsize::new(0));
        #[cfg(not(test))]
        {
            let r = ctrlc_triggered.clone();
            ctrlc::set_handler(move || {
                let previous = r.fetch_add(1, Ordering::SeqCst);
                if previous > 0 {
                    error!("Force-quitting the application");
                    process::exit(1);
                }
            })
            .expect("Unable to set a termination handler");
        }

        EventCoordinator {
            ctrlc_triggered,
            _ctx: PhantomData::default(),
        }
    }
}

impl<Ctx> SystemTrait<Ctx, Event> for EventCoordinator<Ctx>
where
    Ctx: EventManagerTrait<Event>,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::HANDLE_EVENTS | LoopStage::UPDATE
    }

    fn get_event_filter(&self) -> EventFlag {
        EventFlag::SHUTDOWN | EventFlag::HARD_SHUTDOWN
    }

    fn handle_event(&mut self, ctx: &mut Ctx, event: &Event) -> Result<bool, Error> {
        match event.flag() {
            EventFlag::SHUTDOWN => {
                ctx.dispatch_later(Event::hard_shutdown());
                Ok(true)
            }
            EventFlag::HARD_SHUTDOWN => Ok(false),
            _ => Ok(true),
        }
    }

    fn update(&mut self, ctx: &mut Ctx, _: &Duration, _: &Duration) -> Result<(), Error> {
        if self.ctrlc_triggered.load(Ordering::SeqCst) > 0 {
            trace!("Recently caught a termination signal");
            ctx.dispatch_later(Event::shutdown());
            self.ctrlc_triggered.store(0, Ordering::SeqCst);
        }

        Ok(())
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

        assert_eq!(c.get_stage_filter(), LoopStage::HANDLE_EVENTS | LoopStage::UPDATE);
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
