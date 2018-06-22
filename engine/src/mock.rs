use context::SceneGraphTrait;
use ecs::database::{Database, DatabaseTrait, Error as DatabaseError};
use ecs::event::{EventTrait, EventManagerTrait};
use ecs::entity::Entity;
use ecs::mock::MockEvt;
use hierarchy::Hierarchy;
use failure::Error as FailureError;
use math::DepthOrderingTrait;
use std::any::Any;
use std::collections::VecDeque;
use std::convert::TryInto;
use std::f32;
use std::ops::Mul;
use std::sync::RwLock;
use wrappers::glium::EventsLoopTrait;
use components::renderable::RenderTrait;

pub struct MockCtx<E>
where
    E: EventTrait,
{
    pub events: VecDeque<E>,
    pub handle_events_calls: usize,
    pub database: Database,
    pub scene_graph: Hierarchy<Entity, MockModel>,
}

impl<E> Default for MockCtx<E>
where
    E: EventTrait,
{
    fn default() -> Self {
        MockCtx {
            events: Default::default(),
            handle_events_calls: 0,
            database: Default::default(),
            scene_graph: Default::default(),
        }
    }
}

impl<E> EventManagerTrait<E> for MockCtx<E>
where
    E: EventTrait,
{
    fn dispatch_later(&mut self, event: E) {
        self.events.push_back(event)
    }
    fn handle_events<F>(&mut self, mut handler: F) -> Result<bool, FailureError>
    where
        F: FnMut(&mut Self, &E) -> Result<bool, FailureError>,
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

impl<E> DatabaseTrait for MockCtx<E>
where
    E: EventTrait,
{
    fn create_entity(&mut self) -> Entity {
        self.database.create_entity()
    }

    fn destroy_entity(&mut self, entity: &Entity) -> Result<(), DatabaseError> {
        self.database.destroy_entity(entity)
    }

    fn has_entity(&self, entity: &Entity) -> bool {
        self.database.has_entity(entity)
    }

    fn entities(&self) -> usize {
        self.database.entities()
    }

    fn add<C: Any>(&mut self, entity: Entity, component: C) -> Result<(), DatabaseError> {
        self.database.add::<C>(entity, component)
    }

    fn remove<C: Any>(&mut self, entity: &Entity) -> Result<C, DatabaseError> {
        self.database.remove(entity)
    }

    fn has<C: Any>(&self, entity: &Entity) -> bool {
        self.database.has::<C>(entity)
    }

    fn components(&self, entity: &Entity) -> usize {
        self.database.components(entity)
    }

    fn borrow<C: Any>(&self, entity: &Entity) -> Result<&C, DatabaseError> {
        self.database.borrow::<C>(entity)
    }

    fn borrow_mut<C: Any>(&mut self, entity: &Entity) -> Result<&mut C, DatabaseError> {
        self.database.borrow_mut::<C>(entity)
    }
}

impl<E> SceneGraphTrait<Entity, MockModel> for MockCtx<E>
where
    E: EventTrait,
{
    fn update_graph(&mut self) -> Result<(), FailureError> {
        let db = &self.database;
        self.scene_graph.update(&|entity, _, parent_model| {
            let current_model = db.borrow(entity).ok()?;
            Some(parent_model * current_model)
        })?;
        Ok(())
    }

    fn insert_node(&mut self, entity: Entity) {
        self.scene_graph.insert(entity, Default::default())
    }

    fn get_nodes(&self, sort_nodes: bool) -> Vec<(&Entity, &MockModel)> {
        let mut nodes = self.scene_graph.iter().collect::<Vec<_>>();

        if sort_nodes {
            self.sort_nodes(&mut nodes);
        }

        nodes
    }

    fn sort_nodes(&self, nodes: &mut [(&Entity, &MockModel)]) {
        nodes.sort_unstable_by_key(|(_, v)| v.depth_index());
    }
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

impl MockModel {
    pub fn new(z: f32) -> MockModel {
        MockModel(z)
    }
}

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
pub struct MockRenderable {
    pub draw_calls: RwLock<usize>,
}

impl RenderTrait for MockRenderable {
    fn draw(&self) {
        let mut calls = self.draw_calls.write().unwrap();
        *calls += 1;
    }
}
