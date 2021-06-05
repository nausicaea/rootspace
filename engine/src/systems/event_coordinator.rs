use std::time::Duration;

use ecs::{world::event::WorldEvent, EventQueue, ReceiverId, Resources, SerializationName, System, WithResources};

use crate::event::EngineEvent;
use serde::{Deserialize, Serialize};

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
        let mut queue = res.borrow_mut::<EventQueue<EngineEvent>>();
        let events = queue.receive(&self.receiver);
        for event in events {
            match event {
                EngineEvent::PhaseOneShutdown => queue.send(EngineEvent::PhaseTwoShutdown),
                EngineEvent::PhaseTwoShutdown => {
                    let mut queue = res.borrow_mut::<EventQueue<WorldEvent>>();
                    queue.send(WorldEvent::Abort)
                }
                _ => (),
            }
        }
    }
}
