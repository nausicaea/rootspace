use std::marker::PhantomData;
use failure::Error;
use ecs::database::DatabaseTrait;
use ecs::event::{EventTrait, EventManagerTrait};
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;

pub struct EventMonitor<H, A, D, E>
where
    H: EventManagerTrait<E>,
    D: DatabaseTrait,
    E: EventTrait,
{
    phantom_h: PhantomData<H>,
    phantom_a: PhantomData<A>,
    phantom_d: PhantomData<D>,
    phantom_e: PhantomData<E>,
}

impl<H, A, D, E> Default for EventMonitor<H, A, D, E>
where
    H: EventManagerTrait<E>,
    D: DatabaseTrait,
    E: EventTrait,
{
    fn default() -> Self {
        EventMonitor {
            phantom_h: Default::default(),
            phantom_a: Default::default(),
            phantom_d: Default::default(),
            phantom_e: Default::default(),
        }
    }
}

impl<H, A, D, E> SystemTrait<H, A, D, E> for EventMonitor<H, A, D, E>
where
    H: EventManagerTrait<E>,
    D: DatabaseTrait,
    E: EventTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::HANDLE_EVENTS
    }
    fn get_event_filter(&self) -> E::EventFlag {
        Default::default()
    }
    fn handle_event(&mut self, _db: &mut D, _evt_mgr: &mut H, _aux: &mut A, event: &E) -> Result<(), Error> {
        trace!("Received event {:?}", event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ecs::mock::{MockEvt, MockEvtFlag, MockEvtMgr, MockAux, MockDb};
    use super::*;

    #[test]
    fn default() {
        let _s = EventMonitor::<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>::default();
    }

    #[test]
    fn stage_filter() {
        let s = EventMonitor::<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>::default();

        assert_eq!(s.get_stage_filter(), LoopStage::HANDLE_EVENTS);
    }

    #[test]
    fn event_filter() {
        let s = EventMonitor::<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>::default();

        assert_eq!(s.get_event_filter(), MockEvtFlag::all());
    }

    #[test]
    fn handle_event() {
        let mut s = EventMonitor::<MockEvtMgr<MockEvt>, MockAux, MockDb, MockEvt>::default();
        assert!(s.handle_event(&mut Default::default(), &mut Default::default(), &mut Default::default(), &MockEvt::TestEventB(0)).is_ok());
    }
}
