use event::{EventTrait, EventManagerTrait};
use failure::Error;
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq)]
pub enum MockEvt {
    TestEventA(String),
    TestEventB(u32),
}

impl MockEvt {
    pub fn as_flag(&self) -> MockEvtFlag {
        match *self {
            MockEvt::TestEventA(_) => MockEvtFlag::TEST_EVENT_A,
            MockEvt::TestEventB(_) => MockEvtFlag::TEST_EVENT_B,
        }
    }
}

bitflags! {
    pub struct MockEvtFlag: u8 {
        const TEST_EVENT_A = 0x01;
        const TEST_EVENT_B = 0x02;
    }
}

impl Default for MockEvtFlag {
    fn default() -> Self {
        MockEvtFlag::all()
    }
}

impl EventTrait for MockEvt {
    type EventFlag = MockEvtFlag;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool {
        flag.contains(self.as_flag())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MockEvtMgr<E>
where
    E: EventTrait,
{
    pub events: VecDeque<E>,
    pub handle_events_calls: usize,
}

impl<E> Default for MockEvtMgr<E>
where
    E: EventTrait,
{
    fn default() -> Self {
        MockEvtMgr {
            events: Default::default(),
            handle_events_calls: 0,
        }
    }
}

impl<E> EventManagerTrait<E> for MockEvtMgr<E>
where
    E: EventTrait,
{
    fn dispatch_later(&mut self, event: E) {
        self.events.push_back(event)
    }
    fn handle_events<F>(&mut self, mut handler: F) -> Result<bool, Error>
    where
        F: FnMut(&mut Self, &E) -> Result<bool, Error>,
    {
        self.handle_events_calls += 1;

        let tmp = self.events.iter().cloned().collect::<Vec<_>>();
        self.events.clear();

        for event in tmp {
            handler(self, &event)?;
        }

        Ok(true)
    }
}
