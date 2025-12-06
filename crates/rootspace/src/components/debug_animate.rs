use ecs::{Component, ZstStorage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DebugAnimate;

impl Component for DebugAnimate {
    type Storage = ZstStorage<Self>;
}
