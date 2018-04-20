use std::convert::TryInto;
use std::marker::PhantomData;
use std::time::Duration;
use failure::Error;
use ecs::event::{EventTrait, EventManagerTrait};
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;
use event::EventsLoopTrait;

pub struct EventInterface<E, C, Z>
where
    E: EventTrait,
    C: EventManagerTrait<E>,
    Z: EventsLoopTrait<E>,
{
    events_loop: Z,
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
    fn update(&mut self, ctx: &mut C, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
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
    use std::collections::VecDeque;
    use ecs::mock::{MockEvt, MockCtx};
    use super::*;

    #[derive(Clone)]
    enum MockOsEvent {
        TestEventA(String),
        TestEventB(u32),
        TestEventC(f32),
    }

    impl TryInto<MockEvt> for MockOsEvent {
        type Error = ();

        fn try_into(self) -> Result<MockEvt, Self::Error> {
            match self {
                MockOsEvent::TestEventA(s) => Ok(MockEvt::TestEventA(s)),
                MockOsEvent::TestEventB(d) => Ok(MockEvt::TestEventB(d)),
                MockOsEvent::TestEventC(_) => Err(()),
            }
        }
    }

    #[derive(Default)]
    struct MockEventsLoop {
        events: VecDeque<MockOsEvent>,
    }

    impl EventsLoopTrait<MockEvt> for MockEventsLoop {
        type OsEvent = MockOsEvent;

        fn poll<F>(&mut self, mut handler: F) where F: FnMut(Self::OsEvent) {
            let tmp = self.events.iter().cloned().collect::<Vec<_>>();
            self.events.clear();

            for event in tmp {
                handler(event);
            }
        }
    }

    #[test]
    fn default() {
        let _s = EventInterface::<MockEvt, MockCtx<MockEvt>, MockEventsLoop>::new(MockEventsLoop::default());
    }

    #[test]
    fn stage_filter() {
        let s = EventInterface::<MockEvt, MockCtx<MockEvt>, MockEventsLoop>::new(MockEventsLoop::default());
        assert_eq!(s.get_stage_filter(), LoopStage::UPDATE);
    }

    #[test]
    fn update() {
        let mut ctx = MockCtx::<MockEvt>::default();
        let mut s = EventInterface::<MockEvt, MockCtx<MockEvt>, MockEventsLoop>::new(MockEventsLoop::default());

        s.events_loop.events.push_back(MockOsEvent::TestEventA("hello".into()));
        s.events_loop.events.push_back(MockOsEvent::TestEventB(100));
        s.events_loop.events.push_back(MockOsEvent::TestEventC(1.0));

        assert!(s.update(&mut ctx, &Default::default(), &Default::default()).is_ok());
        assert!(s.events_loop.events.is_empty());
        assert_eq!(ctx.events.len(), 2);
    }
}
