use std::marker::PhantomData;
use failure::Error;
use ecs::event::EventTrait;
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;

pub struct EventMonitor<E, C>
where
    E: EventTrait,
{
    phantom_e: PhantomData<E>,
    phantom_c: PhantomData<C>,
}

impl<E, C> Default for EventMonitor<E, C>
where
    E: EventTrait,
{
    fn default() -> Self {
        EventMonitor {
            phantom_e: Default::default(),
            phantom_c: Default::default(),
        }
    }
}

impl<E, C> SystemTrait<C, E> for EventMonitor<E, C>
where
    E: EventTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::HANDLE_EVENTS
    }
    fn get_event_filter(&self) -> E::EventFlag {
        Default::default()
    }
    fn handle_event(&mut self, _ctx: &mut C, event: &E) -> Result<(), Error> {
        trace!("Received event {:?}", event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ecs::mock::{MockEvt, MockEvtFlag, MockCtx};
    use super::*;

    #[test]
    fn default() {
        let _s = EventMonitor::<MockEvt, MockCtx<MockEvt>>::default();
    }

    #[test]
    fn stage_filter() {
        let s = EventMonitor::<MockEvt, MockCtx<MockEvt>>::default();

        assert_eq!(s.get_stage_filter(), LoopStage::HANDLE_EVENTS);
    }

    #[test]
    fn event_filter() {
        let s = EventMonitor::<MockEvt, MockCtx<MockEvt>>::default();

        assert_eq!(s.get_event_filter(), MockEvtFlag::all());
    }

    #[test]
    fn handle_event() {
        let mut s = EventMonitor::<MockEvt, MockCtx<MockEvt>>::default();
        assert!(s.handle_event(&mut Default::default(), &MockEvt::TestEventB(0)).is_ok());
    }
}
