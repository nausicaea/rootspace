use ecs::{Component, VecStorage};
use serde::{Deserialize, Serialize};
use typename::TypeName;

#[derive(Debug, Clone, PartialEq, Eq, TypeName, Serialize, Deserialize)]
pub struct Status(bool);

impl Status {
    pub fn enabled(&self) -> bool {
        self.0
    }

    pub fn enable(&mut self) {
        self.0 = true;
    }

    pub fn disable(&mut self) {
        self.0 = false;
    }
}

impl Default for Status {
    fn default() -> Self {
        Status(true)
    }
}

impl Component for Status {
    type Storage = VecStorage<Self>;
}
