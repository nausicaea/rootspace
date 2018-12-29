#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LoopStage {
    FixedUpdate,
    Update,
    Render,
}
