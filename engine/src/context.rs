use std::collections::VecDeque;
use failure::Error;
use ecs::event::EventManagerTrait;
use ecs::entity::Entity;
use hierarchy::Hierarchy;
use event::Event;
use components::model::Model;

pub struct Context {
    scene_graph: Hierarchy<Entity, Model>,
    events: VecDeque<Event>,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            scene_graph: Default::default(),
            events: Default::default(),
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
