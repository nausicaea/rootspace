use std::marker::PhantomData;
use std::time::Duration;
use failure::Error;
use ecs::event::EventTrait;
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;

pub trait EventsLoopTrait: Default {
}

pub struct EventInterface<C, E, Z>
where
    E: EventTrait,
    Z: EventsLoopTrait,
{
    pub events_loop: Z,
    phantom_c: PhantomData<C>,
    phantom_e: PhantomData<E>,
}

impl<C, E, Z> Default for EventInterface<C, E, Z>
where
    E: EventTrait,
    Z: EventsLoopTrait,
{
    fn default() -> Self {
        EventInterface {
            events_loop: Default::default(),
            phantom_c: Default::default(),
            phantom_e: Default::default(),
        }
    }
}

impl<C, E, Z> SystemTrait<C, E> for EventInterface<C, E, Z>
where
    E: EventTrait,
    Z: EventsLoopTrait,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::UPDATE
    }
    fn update(&mut self, _ctx: &mut C, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ecs::mock::{MockEvt, MockEvtFlag, MockCtx};
    use super::*;

    #[test]
    fn default() {
        let _s = EventInterface::<MockCtx<MockEvt>, MockEvt>::default();
    }

    #[test]
    fn stage_filter() {
        let s = EventInterface::<MockCtx<MockEvt>, MockEvt>::default();
        assert_eq!(s.get_stage_filter(), LoopStage::UPDATE);
    }

    #[test]
    fn update() {
        let mut s = EventInterface::<MockCtx<MockEvt>, MockEvt>::default();

        assert!(s.update(&mut Default::default(), &Default::default(), &Default::default()).is_ok());
    }
}
