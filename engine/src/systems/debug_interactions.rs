use crate::event::{KeyModifiers, KeyState, VirtualKeyCode};
use crate::EngineEvent;
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
                    state: KeyState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::R),
                    modifiers: KeyModifiers { logo: true, .. },
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
