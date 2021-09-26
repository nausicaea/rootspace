use std::ops::Mul;

use ecs::{Component, VecStorage};
use serde::{Deserialize, Serialize};
use std::iter::Product;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    enabled: bool,
    visible: bool,
}

impl Status {
    pub fn new(enabled: bool, visible: bool) -> Self {
        Status::builder().with_enabled(enabled).with_visible(visible).build()
    }

    pub fn builder() -> StatusBuilder {
        StatusBuilder::default()
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
        Status::builder().build()
    }
}

impl Mul<Status> for Status {
    type Output = Status;

    fn mul(self, rhs: Status) -> Status {
        &self * &rhs
    }
}

impl<'a, 'b> Mul<&'a Status> for &'b Status {
    type Output = Status;

    fn mul(self, rhs: &'a Status) -> Status {
        Status {
            enabled: self.enabled && rhs.enabled,
            visible: self.visible && rhs.visible,
        }
    }
}

impl<'a> Product<&'a Status> for Status {
    fn product<I: Iterator<Item = &'a Status>>(iter: I) -> Self {
        iter.fold(Status::default(), |state, value| &state * value)
    }
}

impl Product for Status {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Status::default(), |state, value| state * value)
    }
}

impl Component for Status {
    type Storage = VecStorage<Self>;
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let enbl = if self.enabled { "enabled" } else { "disabled" };
        let vsbl = if self.visible { "visible" } else { "hidden" };
        write!(f, "{}, {}", enbl, vsbl)
    }
}

#[derive(Debug)]
pub struct StatusBuilder {
    enabled: bool,
    visible: bool,
}

impl StatusBuilder {
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn build(self) -> Status {
        Status {
            enabled: self.enabled,
            visible: self.visible,
        }
    }
}

impl Default for StatusBuilder {
    fn default() -> Self {
        StatusBuilder {
            enabled: true,
            visible: true,
        }
    }
}
