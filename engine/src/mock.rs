use context::SceneGraphTrait;
use ecs::entity::Entity;
use ecs::mock::{MockCtx, MockEvt};
use failure::Error as FailureError;
use math::DepthOrderingTrait;
use std::collections::VecDeque;
use std::convert::TryInto;
use std::f32;
use std::ops::Mul;
use wrappers::glium::EventsLoopTrait;
use components::renderable::RenderTrait;

impl SceneGraphTrait<Entity, MockModel> for MockCtx<MockEvt> {
    fn update_graph(&mut self) -> Result<(), FailureError> {
        Ok(())
    }

    fn get_nodes(
        &self,
        _sort_nodes: bool,
    ) -> Vec<(&Entity, &MockModel)> {
        Vec::new()
    }

    fn sort_nodes(&self, _nodes: &mut [(&Entity, &MockModel)]) {}
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

pub struct MockRenderable;

impl RenderTrait for MockRenderable {
    fn draw(&self) {
        unimplemented!()
    }
}
