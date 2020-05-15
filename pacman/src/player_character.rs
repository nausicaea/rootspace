use ecs::{System, Resources, EventQueue, ReceiverId, Component, ZstStorage};
use engine::{EngineEvent, event::{VirtualKeyCode, KeyState}};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use log::trace;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PlayerCharacterMarker;

impl Component for PlayerCharacterMarker {
    type Storage = ZstStorage<Self>;
}

pub struct PlayerCharacter {
    receiver: ReceiverId<EngineEvent>,
}

impl PlayerCharacter {
    pub fn new(queue: &mut EventQueue<EngineEvent>) -> Self {
        trace!("PlayerCharacter subscribing to EventQueue<EngineEvent>");

        PlayerCharacter {
            receiver: queue.subscribe(),
        }
    }
}

impl System for PlayerCharacter {
    fn name(&self) -> &'static str {
        stringify!(PlayerCharacter)
    }

    fn run(&mut self, res: &Resources, _t: &Duration, _dt: &Duration) {
        let events = res.borrow_mut::<EventQueue<EngineEvent>>().receive(&self.receiver);
        // let pc = res.borrow_components::<

        for event in events {
            match event {
                EngineEvent::KeyboardInput { state: KeyState::Pressed, virtual_keycode: Some(vkc), .. } => {
                    match vkc {
                        VirtualKeyCode::Up => (),
                        VirtualKeyCode::Down => (),
                        VirtualKeyCode::Left => (),
                        VirtualKeyCode::Right => (),
                        _ => (),
                    }
                },
                _ => (),
            }
        }
    }
}
