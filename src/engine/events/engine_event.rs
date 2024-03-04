#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum EngineEvent {
    Command(Vec<String>),
    Exit,
}
