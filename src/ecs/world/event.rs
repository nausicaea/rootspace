use serde::{Deserialize, Serialize};

use super::super::entity::Entity;

/// Events defined and processed by the world itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldEvent {
    CreateEntity,
    EntityCreated(Entity),
    DestroyEntity(Entity),
    EntityDestroyed(Entity),
    Exiting,
}
