use crate::event::EngineEvent;
use crate::event::{ElementState, ModifiersState, VirtualKeyCode};
use ecs::{EventQueue, ReceiverId, Resources, SerializationName, System, WithResources, WorldEvent};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugInteractions {
    receiver: ReceiverId<EngineEvent>,
}

impl WithResources for DebugInteractions {
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();

        DebugInteractions { receiver }
    }
}

impl SerializationName for DebugInteractions {}

impl System for DebugInteractions {
    fn run(&mut self, res: &Resources, _: &Duration, _: &Duration) {
        let events = res.borrow_mut::<EventQueue<EngineEvent>>().receive(&self.receiver);
        for event in events {
            match event {
                EngineEvent::KeyboardInput {
                    state: ElementState::Released,
                    virtual_keycode: Some(VirtualKeyCode::Q),
                    modifiers: ModifiersState { logo: true, .. },
                    ..
                } => {
                    res.borrow_mut::<EventQueue<EngineEvent>>()
                        .send(EngineEvent::PhaseOneShutdown);
                }
                EngineEvent::KeyboardInput {
                    state: ElementState::Released,
                    virtual_keycode: Some(VirtualKeyCode::R),
                    modifiers: ModifiersState { logo: true, .. },
                    ..
                } => {
                    res.borrow_mut::<EventQueue<WorldEvent>>()
                        .send(WorldEvent::DeserializeLastState);
                }
                _ => (),
            }
        }
    }
}
