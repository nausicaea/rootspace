//! Provides a way to select the persistence of an object.

use serde::{Deserialize, Serialize};

/// Determines how persistent a particular objec should be. This allows selectively deleting and
/// retaining objects upon multiple re-initialisations of the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Persistence {
    /// The respective object will be deleted when resetting the world.
    None,
    /// The respective object should be present for the entire runtime of the program.
    Runtime,
}
