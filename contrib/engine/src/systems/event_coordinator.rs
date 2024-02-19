use std::time::Duration;

use ecs::{
    event_queue::receiver_id::ReceiverId, world::event::WorldEvent, EventQueue, Resources, SerializationName, System,
    WithResources,
};
use serde::{Deserialize, Serialize};

use crate::event::EngineEvent;

#[derive(Debug, Serialize, Deserialize)]
pub struct EventCoordinator {
    receiver: ReceiverId<EngineEvent>,
}

impl WithResources for EventCoordinator {
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();

        EventCoordinator { receiver }
    }
}

impl SerializationName for EventCoordinator {}

impl System for EventCoordinator {
    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res.borrow_mut::<EventQueue<EngineEvent>>().receive(&self.receiver);
        for event in events {
            match event {
                EngineEvent::PhaseOneShutdown => res
                    .borrow_mut::<EventQueue<EngineEvent>>()
                    .send(EngineEvent::PhaseTwoShutdown),
                EngineEvent::PhaseTwoShutdown => res.borrow_mut::<EventQueue<WorldEvent>>().send(WorldEvent::Abort),
                _ => (),
            }
        }
    }
}
