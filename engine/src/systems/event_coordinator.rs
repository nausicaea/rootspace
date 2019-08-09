use crate::event::EngineEvent;
use ecs::{EventQueue, ReceiverId, Resources, System, WorldEvent};
use std::time::Duration;

pub struct EventCoordinator {
    receiver: ReceiverId<EngineEvent>,
}

impl EventCoordinator {
    pub fn new(res: &mut Resources) -> Self {
        let events = res.get_mut::<EventQueue<EngineEvent>>();
        let receiver = events.subscribe();

        EventCoordinator { receiver }
    }
}

impl System for EventCoordinator {
    fn name(&self) -> &'static str {
        "EventCoordinator"
    }

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
