use ecs::{Component, VecStorage};

#[cfg_attr(feature = "diagnostics", derive(TypeName))]
#[derive(Debug, Clone, PartialEq, Eq)]
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
