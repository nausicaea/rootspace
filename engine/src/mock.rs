use std::convert::TryInto;
use std::collections::VecDeque;
use ecs::mock::MockEvt;
use wrappers::{FrameTrait, DisplayTrait, EventsLoopTrait};

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

#[derive(Default)]
pub struct MockFrame {
    pub error_out: bool,
}

impl MockFrame {
    pub fn new(error_out: bool) -> Self {
        MockFrame {
            error_out: error_out,
        }
    }
}

impl FrameTrait for MockFrame {
    type Error = ();

    fn finalize(self) -> Result<(), Self::Error> {
        if self.error_out {
            Err(())
        } else {
            Ok(())
        }
    }
}

pub struct MockDisplay {
    pub cause_frame_to_error: bool,
}

impl MockDisplay {
    pub fn new(cause_frame_to_error: bool) -> Self {
        MockDisplay {
            cause_frame_to_error: cause_frame_to_error,
        }
    }
}

impl DisplayTrait for MockDisplay {
    type Error = ();
    type EventsLoop = ();
    type Frame = MockFrame;

    fn create(_events_loop: &Self::EventsLoop, _title: &str, _dimensions: &[u32; 2], _vsync: bool, _msaa: u16) -> Result<Self, Self::Error> {
        Ok(MockDisplay::new(false))
    }

    fn create_frame(&self) -> Self::Frame {
        MockFrame::new(self.cause_frame_to_error)
    }
}

