use ecs::{Component, EventQueue, ReceiverId, Resources, System, ZstStorage, MaybeDefault, WithResources};
use engine::{
    components::Model,
    event::{KeyState, VirtualKeyCode},
    EngineEvent,
};
use log::trace;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct PlayerCharacterMarker;

impl Component for PlayerCharacterMarker {
    type Storage = ZstStorage<Self>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerCharacter {
    receiver: ReceiverId<EngineEvent>,
}

impl WithResources for PlayerCharacter {
    fn with_resources(res: &Resources) -> Self {
        let receiver = res.borrow_mut::<EventQueue<EngineEvent>>()
            .subscribe::<Self>();

        PlayerCharacter {
            receiver,
        }
    }
}

impl PlayerCharacter {
    fn move_char(&self, res: &Resources, dt: &Duration, dir: MoveDirection) {
        let delta = match dir {
            MoveDirection::Up => nalgebra::Vector3::new(0.0, 1.0, 0.0),
            MoveDirection::Down => nalgebra::Vector3::new(0.0, -1.0, 0.0),
            MoveDirection::Left => nalgebra::Vector3::new(-1.0, 0.0, 0.0),
            MoveDirection::Right => nalgebra::Vector3::new(1.0, 0.0, 0.0),
        } * dt.as_secs_f32();

        for (_, m) in res.iter_rw::<PlayerCharacterMarker, Model>() {
            m.set_position(m.position() + delta);
        }
    }
}

impl System for PlayerCharacter {
    fn run(&mut self, res: &Resources, _t: &Duration, dt: &Duration) {
        let events = res
            .borrow_mut::<EventQueue<EngineEvent>>()
            .receive(&self.receiver);

        for event in events {
            match event {
                EngineEvent::KeyboardInput {
                    state: KeyState::Pressed,
                    virtual_keycode: Some(vkc),
                    ..
                } => match vkc {
                    VirtualKeyCode::Up => self.move_char(res, dt, MoveDirection::Up),
                    VirtualKeyCode::Down => self.move_char(res, dt, MoveDirection::Down),
                    VirtualKeyCode::Left => self.move_char(res, dt, MoveDirection::Left),
                    VirtualKeyCode::Right => self.move_char(res, dt, MoveDirection::Right),
                    _ => (),
                },
                _ => (),
            }
        }
    }
}
