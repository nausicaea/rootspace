use file_manipulation::{FilePathBuf, NewOrExFilePathBuf};
use serde::{Deserialize, Serialize};

/// Events defined and processed by the world itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldEvent {
    /// Causes the WorldTrait::maintain() method to serialize the entire world state to the given
    /// file.
    Serialize(NewOrExFilePathBuf),
    /// Causes the WorldTrait::maintain() method to deserialize the entire world state from the
    /// given file.
    Deserialize(FilePathBuf),
    /// Signals the completion of deserialization.
    DeserializationComplete,
    /// Causes the WorldTrait::maintain() method to return `false`, which should result in the game
    /// engine to abort.
    Abort,
}
