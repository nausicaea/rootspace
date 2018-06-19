use context::SceneGraphTrait;
use ecs::entity::Entity;
use ecs::mock::{MockCtx, MockEvt};
use failure::Error as FailureError;
use math::DepthOrderingTrait;
use std::collections::VecDeque;
use std::convert::TryInto;
use std::f32;
use std::ops::Mul;
use wrappers::{DisplayTrait, EventsLoopTrait, FrameTrait};

impl SceneGraphTrait<Entity, MockModel> for MockCtx<MockEvt> {
    fn get_current_nodes(
        &mut self,
        _sort_nodes: bool,
    ) -> Result<Vec<(&Entity, &MockModel)>, FailureError> {
        Ok(Vec::new())
    }

    fn sort_graph_nodes(&self, _nodes: &mut [(&Entity, &MockModel)]) {}
}

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

#[derive(Clone, Default)]
pub struct MockModel(f32);

impl DepthOrderingTrait for MockModel {
    fn depth_index(&self) -> i32 {
        (self.0 / f32::EPSILON).round() as i32
    }
}

impl<'a, 'b> Mul<&'b MockModel> for &'a MockModel {
    type Output = MockModel;

    fn mul(self, rhs: &'b MockModel) -> Self::Output {
        MockModel(self.0 * rhs.0)
    }
}

#[derive(Default)]
pub struct MockFrame {
    pub error_out: bool,
    pub clear_call_count: usize,
}

impl MockFrame {
    pub fn new(error_out: bool) -> Self {
        MockFrame {
            error_out: error_out,
            clear_call_count: 0,
        }
    }
}

impl FrameTrait for MockFrame {
    fn clear(&mut self, _color: &[f32; 4], _depth: f32) {
        self.clear_call_count += 1
    }

    fn finalize(self) -> Result<(), FailureError> {
        if self.error_out {
            Err(format_err!("MockFrame had an error."))
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
    type EventsLoop = ();
    type Frame = MockFrame;

    fn create(
        _events_loop: &Self::EventsLoop,
        _title: &str,
        _dimensions: &[u32; 2],
        _vsync: bool,
        _msaa: u16,
    ) -> Result<Self, FailureError> {
        Ok(MockDisplay::new(false))
    }

    fn create_frame(&self) -> Self::Frame {
        MockFrame::new(self.cause_frame_to_error)
    }
}
