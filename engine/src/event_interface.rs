use std::marker::PhantomData;
use std::time::Duration;
use failure::Error;
use ecs::event::{EventTrait, EventManagerTrait};
use ecs::loop_stage::LoopStage;
use ecs::system::SystemTrait;

pub trait EventsLoopTrait<E>
where
    E: EventTrait,
{
    type OsEvent: Into<Option<E>>;

    fn poll<F>(&mut self, handler: F) where F: FnMut(Self::OsEvent);
}

pub struct EventInterface<C, E, Z>
where
    C: EventManagerTrait<E>,
    E: EventTrait,
    Z: Default + EventsLoopTrait<E>,
{
    pub events_loop: Z,
    phantom_c: PhantomData<C>,
    phantom_e: PhantomData<E>,
}

impl<C, E, Z> Default for EventInterface<C, E, Z>
where
    C: EventManagerTrait<E>,
    E: EventTrait,
    Z: Default + EventsLoopTrait<E>,
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
    C: EventManagerTrait<E>,
    E: EventTrait,
    Z: Default + EventsLoopTrait<E>,
{
    fn get_stage_filter(&self) -> LoopStage {
        LoopStage::UPDATE
    }
    fn update(&mut self, ctx: &mut C, _time: &Duration, _delta_time: &Duration) -> Result<(), Error> {
        self.events_loop.poll(|os_event| {
            if let Some(event) = os_event.into() {
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

    impl Into<Option<MockEvt>> for MockOsEvent {
        fn into(self) -> Option<MockEvt> {
            match self {
                MockOsEvent::TestEventA(s) => Some(MockEvt::TestEventA(s)),
                MockOsEvent::TestEventB(d) => Some(MockEvt::TestEventB(d)),
                MockOsEvent::TestEventC(_) => None,
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
        let _s = EventInterface::<MockCtx<MockEvt>, MockEvt, MockEventsLoop>::default();
    }

    #[test]
    fn stage_filter() {
        let s = EventInterface::<MockCtx<MockEvt>, MockEvt, MockEventsLoop>::default();
        assert_eq!(s.get_stage_filter(), LoopStage::UPDATE);
    }

    #[test]
    fn update() {
        let mut ctx = MockCtx::<MockEvt>::default();
        let mut s = EventInterface::<MockCtx<MockEvt>, MockEvt, MockEventsLoop>::default();

        s.events_loop.events.push_back(MockOsEvent::TestEventA("hello".into()));
        s.events_loop.events.push_back(MockOsEvent::TestEventB(100));
        s.events_loop.events.push_back(MockOsEvent::TestEventC(1.0));

        assert!(s.update(&mut ctx, &Default::default(), &Default::default()).is_ok());
        assert!(s.events_loop.events.is_empty());
        assert_eq!(ctx.events.len(), 2);
    }
}
