use std::collections::VecDeque;
use std::hash::Hash;
use failure::Error;
use hierarchy::Hierarchy;
use ecs::event::EventManagerTrait;
use ecs::entity::Entity;
use event::Event;
use components::model::Model;

pub struct Context {
    events: VecDeque<Event>,
    scene_graph: Hierarchy<Entity, Model>,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            events: Default::default(),
            scene_graph: Default::default(),
        }
    }
}

impl EventManagerTrait<Event> for Context {
    fn dispatch_later(&mut self, event: Event) {
        self.events.push_back(event)
    }
    fn handle_events<F>(&mut self, mut handler: F) -> Result<bool, Error>
    where
        F: FnMut(&mut Self, &Event) -> Result<bool, Error>,
    {
        let tmp = self.events.iter().cloned().collect::<Vec<_>>();
        self.events.clear();

        for event in tmp {
            handler(self, &event)?;
        }

        Ok(true)
    }
}

pub trait SceneGraphTrait<K, V>
where
    K: Clone + Default + Eq + Hash,
    V: Clone + Default,
{
    fn scene_graph(&self) -> &Hierarchy<K, V>;
    fn scene_graph_mut(&mut self) -> &mut Hierarchy<K, V>;
}

impl SceneGraphTrait<Entity, Model> for Context {
    fn scene_graph(&self) -> &Hierarchy<Entity, Model> {
        &self.scene_graph
    }

    fn scene_graph_mut(&mut self) -> &mut Hierarchy<Entity, Model> {
        &mut self.scene_graph
    }
}
