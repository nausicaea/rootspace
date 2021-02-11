use std::time::Duration;

use log::trace;

use ecs::{world::event::WorldEvent, EventQueue, ReceiverId, Resources, System, MaybeDefault, WithResources};

use crate::event::EngineEvent;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventCoordinator {
    receiver: ReceiverId<EngineEvent>,
}

impl WithResources for EventCoordinator {
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<EngineEvent>>()
            .subscribe::<Self>();

        EventCoordinator {
            receiver,
        }
    }
}

impl MaybeDefault for EventCoordinator {}

impl System for EventCoordinator {
    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let mut queue = res.borrow_mut::<EventQueue<EngineEvent>>();
        let events = queue.receive(&self.receiver);
        for event in events {
            match event {
                EngineEvent::Shutdown => queue.send(EngineEvent::HardShutdown),
                EngineEvent::HardShutdown => {
                    let mut queue = res.borrow_mut::<EventQueue<WorldEvent>>();
                    queue.send(WorldEvent::Abort)
                }
                _ => (),
            }
        }
    }
}
