#[cfg(not(test))]
use ctrlc;
use ecs::{EventManagerTrait, LoopStage, SystemTrait};
use event::EngineEventTrait;
use failure::Error;
#[cfg(not(test))]
use std::process;
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

pub struct EventCoordinator<Ctx, Evt> {
    ctrlc_triggered: Arc<AtomicUsize>,
    _ctx: PhantomData<Ctx>,
    _evt: PhantomData<Evt>,
}

impl<Ctx, Evt> Default for EventCoordinator<Ctx, Evt> {
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
            _evt: PhantomData::default(),
        }
    }
}

impl<Ctx, Evt> SystemTrait<Ctx, Evt> for EventCoordinator<Ctx, Evt>
where
    Ctx: EventManagerTrait<Evt>,
    Evt: EngineEventTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::HANDLE_EVENTS | LoopStage::UPDATE
    }

    fn get_event_filter(&self) -> Evt::EventFlag {
        Evt::shutdown() | Evt::hard_shutdown()
    }

    fn handle_event(&mut self, ctx: &mut Ctx, event: &Evt) -> Result<bool, Error> {
        if event.matches_filter(Evt::shutdown()) {
            ctx.dispatch_later(Evt::new_hard_shutdown());
            Ok(true)
        } else if event.matches_filter(Evt::hard_shutdown()) {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    fn update(&mut self, ctx: &mut Ctx, _: &Duration, _: &Duration) -> Result<(), Error> {
        if self.ctrlc_triggered.load(Ordering::SeqCst) > 0 {
            trace!("Recently caught a termination signal");
            ctx.dispatch_later(Evt::new_shutdown());
            self.ctrlc_triggered.store(0, Ordering::SeqCst);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context::Context;
    use mock::{MockEvt, MockEvtFlag};

    #[test]
    fn new() {
        let _: EventCoordinator<Context<MockEvt>, MockEvt> = EventCoordinator::default();
    }

    #[test]
    fn get_stage_filter() {
        let c: EventCoordinator<Context<MockEvt>, MockEvt> = EventCoordinator::default();

        assert_eq!(c.get_stage_filter(), LoopStage::HANDLE_EVENTS | LoopStage::UPDATE);
    }

    #[test]
    fn get_event_filter() {
        let c: EventCoordinator<Context<MockEvt>, MockEvt> = EventCoordinator::default();

        assert_eq!(c.get_event_filter(), MockEvtFlag::SHUTDOWN | MockEvtFlag::HARD_SHUTDOWN);
    }

    #[test]
    fn handle_event() {
        let mut c: EventCoordinator<Context<MockEvt>, MockEvt> = EventCoordinator::default();

        let r = c.handle_event(&mut Default::default(), &MockEvt::new_startup());
        assert_ok!(r);
        assert!(r.unwrap());

        let r = c.handle_event(&mut Default::default(), &MockEvt::new_shutdown());
        assert_ok!(r);
        assert!(r.unwrap());

        let r = c.handle_event(&mut Default::default(), &MockEvt::new_hard_shutdown());
        assert_ok!(r);
        assert!(!r.unwrap());
    }
}
