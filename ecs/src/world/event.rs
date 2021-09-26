use file_manipulation::{FilePathBuf, NewOrExFilePathBuf};
use serde::{Deserialize, Serialize};

use crate::Entity;

/// Events defined and processed by the world itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldEvent {
    CreateEntity,
    EntityCreated(Entity),
    DestroyEntity(Entity),
    EntityDestroyed(Entity),
    /// Causes the World::maintain() method to serialize the entire world state to the given
    /// file.
    Serialize(NewOrExFilePathBuf),
    /// Causes the World::maintain() method to deserialize the entire world state from the
    /// given file.
    Deserialize(FilePathBuf),
    /// Causes the World::maintain() method to deserialize the most recently loaded world state.
    DeserializeLastState,
    /// Signals the completion of deserialization.
    DeserializationComplete,
    /// Causes the World::maintain() method to return `false`, which should result in the game
    /// engine to abort.
    Abort,
}
