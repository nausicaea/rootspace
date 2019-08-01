//! Provides a way to select the persistence of an object.

/// Determines how persistent a particular objec should be. This allows selectively deleting and
/// retaining objects upon multiple re-initialisations of the world.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Persistence {
    /// The respective object will be deleted when resetting the world.
    None,
    /// The respective object should be present for the entire runtime of the program.
    Runtime,
}
