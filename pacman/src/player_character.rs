use ecs::{System, Resources};
use std::time::Duration;

pub struct PlayerCharacter;

impl System for PlayerCharacter {
    fn name(&self) -> &'static str {
        stringify!(PlayerCharacter)
    }

    fn run(&mut self, _res: &Resources, _t: &Duration, _dt: &Duration) {
    }
}
