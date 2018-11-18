use ecs::{EventTrait, LoopStage, SystemTrait};
use failure::Error;
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

impl<Ctx, Evt> SystemTrait<Ctx, Evt> for EventMonitor<Ctx, Evt>
where
    Evt: EventTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::HANDLE_EVENTS
    }
    fn get_event_filter(&self) -> Evt::EventFlag {
        Default::default()
    }
    fn handle_event(&mut self, _ctx: &mut Ctx, event: &Evt) -> Result<bool, Error> {
        trace!("Received {:?}", event);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context::Context;
    use event::EngineEventTrait;
    use mock::{MockEvt, MockEvtFlag};

    #[test]
    fn default() {
        let _s = EventMonitor::<Context<MockEvt>, MockEvt>::default();
    }

    #[test]
    fn stage_filter() {
        let s = EventMonitor::<Context<MockEvt>, MockEvt>::default();

        assert_eq!(s.get_stage_filter(), LoopStage::HANDLE_EVENTS);
    }

    #[test]
    fn event_filter() {
        let s = EventMonitor::<Context<MockEvt>, MockEvt>::default();

        assert_eq!(s.get_event_filter(), MockEvtFlag::all());
    }

    #[test]
    fn handle_event() {
        let mut s = EventMonitor::<Context<MockEvt>, MockEvt>::default();
        let r = s.handle_event(&mut Default::default(), &MockEvt::new_startup());
        assert_ok!(r);
        assert!(r.unwrap());
    }
}
