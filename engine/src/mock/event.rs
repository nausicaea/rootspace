use ecs::mock::MockEvt;
use std::collections::VecDeque;
use std::convert::TryInto;
use wrappers::glium::EventsLoopTrait;

#[derive(Clone)]
pub enum MockOsEvent {
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
pub struct MockEventsLoop {
    events: VecDeque<MockOsEvent>,
}

impl MockEventsLoop {
    pub fn enqueue(&mut self, event: MockOsEvent) {
        self.events.push_back(event);
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl EventsLoopTrait<MockEvt> for MockEventsLoop {
    type OsEvent = MockOsEvent;

    fn poll<F>(&mut self, mut handler: F)
    where
        F: FnMut(Self::OsEvent),
    {
        let tmp = self.events.iter().cloned().collect::<Vec<_>>();
        self.events.clear();

        for event in tmp {
            handler(event);
        }
    }
}
