//! Provides a way to select the main loop stage. Used for registering systems with the world.

/// Selects the stage of the main loop (mapped to individual methods of `World`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoopStage {
    /// Select an update stage that is called at regular intervals (e.g. for physics calculations).
    FixedUpdate,
    /// Select an update stage that is called at varying intervals (for non-critical state updates).
    Update,
    /// Select the render stage that should only be used by rendering systems.
    Render,
}
