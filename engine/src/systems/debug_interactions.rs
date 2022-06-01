use std::time::Duration;

use ecs::{EventQueue, ReceiverId, Resources, SerializationName, System, WithResources, WorldEvent};
use serde::{Deserialize, Serialize};
use log::info;

use crate::event::{ElementState, EngineEvent, ModifiersState, VirtualKeyCode};

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugInteractions {
    receiver: ReceiverId<EngineEvent>,
    edit_mode_enabled: bool,
}

impl DebugInteractions {
    fn normal_mode(&mut self, res: &Resources, event: EngineEvent) {
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
                virtual_keycode: Some(VirtualKeyCode::L),
                modifiers: ModifiersState { logo: true, shift: true, .. },
                ..
            } => {
                res.borrow_mut::<EventQueue<WorldEvent>>()
                    .send(WorldEvent::DeserializeLastState);
            }
            EngineEvent::KeyboardInput {
                state: ElementState::Released,
                virtual_keycode: Some(VirtualKeyCode::S),
                modifiers: ModifiersState { logo: true, shift: true, .. },
                ..
            } => {
                res.borrow_mut::<EventQueue<WorldEvent>>()
                    .send(WorldEvent::SerializeLastState);
            }
            EngineEvent::KeyboardInput {
                state: ElementState::Released,
                virtual_keycode: Some(VirtualKeyCode::E),
                modifiers: ModifiersState { logo: true, shift: true, .. },
                ..
            } => {
                info!("Edit mode enabled");
                self.edit_mode_enabled = true;
            }
            _ => (),
        }
    }

    fn edit_mode(&mut self, _res: &Resources, event: EngineEvent) {
        match event {
            EngineEvent::KeyboardInput {
                state: ElementState::Released,
                virtual_keycode: Some(VirtualKeyCode::Escape),
                ..
            } => {
                info!("Edit mode disabled");
                self.edit_mode_enabled = false;
            }
            _ => (),
        }
    }
}

impl WithResources for DebugInteractions {
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();

        DebugInteractions { receiver, edit_mode_enabled: false }
    }
}

impl SerializationName for DebugInteractions {}

impl System for DebugInteractions {
    fn run(&mut self, res: &Resources, _: &Duration, _: &Duration) {
        let events = res.borrow_mut::<EventQueue<EngineEvent>>().receive(&self.receiver);
        for event in events {
            if self.edit_mode_enabled {
                self.edit_mode(res, event);
            } else {
                self.normal_mode(res, event);
            }
        }
    }
}
