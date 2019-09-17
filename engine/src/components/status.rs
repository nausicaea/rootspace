use ecs::{Component, VecStorage};
use serde::{Deserialize, Serialize};
use typename::TypeName;

#[derive(Debug, Clone, PartialEq, Eq, TypeName, Serialize, Deserialize)]
pub struct Status {
    enabled: bool,
    visible: bool,
}

impl Status {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }
}

impl Default for Status {
    fn default() -> Self {
        Status {
            enabled: true,
            visible: true,
        }
    }
}

impl Component for Status {
    type Storage = VecStorage<Self>;
}
