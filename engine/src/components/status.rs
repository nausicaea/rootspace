use ecs::{Component, VecStorage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    enabled: bool,
    visible: bool,
}

impl Status {
    pub fn new(enabled: bool, visible: bool) -> Self {
        Status {
            enabled,
            visible,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
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

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
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
