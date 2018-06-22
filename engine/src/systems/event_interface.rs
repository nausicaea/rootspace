use ecs::event::{EventManagerTrait, EventTrait};
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;
use failure::Error;
use std::convert::TryInto;
use std::marker::PhantomData;
use std::time::Duration;
use wrappers::glium::EventsLoopTrait;

pub struct EventInterface<E, C, Z>
where
    E: EventTrait,
    C: EventManagerTrait<E>,
    Z: EventsLoopTrait<E>,
{
    pub events_loop: Z,
    phantom_e: PhantomData<E>,
    phantom_c: PhantomData<C>,
}

impl<E, C, Z> EventInterface<E, C, Z>
where
    E: EventTrait,
    C: EventManagerTrait<E>,
    Z: EventsLoopTrait<E>,
{
    pub fn new(events_loop: Z) -> Self {
        EventInterface {
            events_loop: events_loop,
            phantom_e: Default::default(),
            phantom_c: Default::default(),
        }
    }
}

impl<E, C, Z> SystemTrait<C, E> for EventInterface<E, C, Z>
where
    E: EventTrait,
    C: EventManagerTrait<E>,
    Z: EventsLoopTrait<E>,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::UPDATE
    }
    fn update(
        &mut self,
        ctx: &mut C,
        _time: &Duration,
        _delta_time: &Duration,
    ) -> Result<(), Error> {
        self.events_loop.poll(|os_event| {
            if let Ok(event) = os_event.try_into() {
                ctx.dispatch_later(event);
            }
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecs::mock::{MockCtx, MockEvt};
    use mock::{MockEventsLoop, MockOsEvent};

    #[test]
    fn default() {
        let _s = EventInterface::<MockEvt, MockCtx<MockEvt>, MockEventsLoop>::new(
            MockEventsLoop::default(),
        );
    }

    #[test]
    fn stage_filter() {
        let s = EventInterface::<MockEvt, MockCtx<MockEvt>, MockEventsLoop>::new(
            MockEventsLoop::default(),
        );
        assert_eq!(s.get_stage_filter(), LoopStage::UPDATE);
    }

    #[test]
    fn update() {
        let mut ctx = MockCtx::<MockEvt>::default();
        let mut s = EventInterface::<MockEvt, MockCtx<MockEvt>, MockEventsLoop>::new(
            MockEventsLoop::default(),
        );

        s.events_loop
            .enqueue(MockOsEvent::TestEventA("hello".into()));
        s.events_loop.enqueue(MockOsEvent::TestEventB(100));
        s.events_loop.enqueue(MockOsEvent::TestEventC(1.0));

        assert_ok!(s.update(&mut ctx, &Default::default(), &Default::default()));
        assert!(s.events_loop.is_empty());
        assert_eq!(ctx.events.len(), 2);
    }
}
