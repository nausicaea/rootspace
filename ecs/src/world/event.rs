use downcast_rs::__std::path::PathBuf;

use crate::resources::ConflictResolution;
use serde::{Deserialize, Serialize};

/// Events defined and processed by the world itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldEvent {
    /// Causes the WorldTrait::maintain() method to serialize the entire world state to the given
    /// file.
    Serialize(PathBuf),
    /// Signals the completion of serialization.
    SerializationComplete,
    /// Causes the WorldTrait::maintain() method to deserialize the entire world state from the
    /// given file.
    Deserialize(PathBuf),
    /// Causes the WorldTrait::maintain() method to deserialize a world state additively from a
    /// file into the currently loaded state.
    DeserializeAdditive(PathBuf, ConflictResolution),
    /// Signals the completion of deserialization.
    DeserializationComplete,
    /// Causes the WorldTrait::maintain() method to return `false`, which should result in the game
    /// engine to abort.
    Abort,
}
