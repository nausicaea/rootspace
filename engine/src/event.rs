use typename::TypeName;

#[derive(Debug, Clone, PartialEq, TypeName)]
pub enum EngineEvent {
    Startup,
    Shutdown,
    HardShutdown,
    Command(Vec<String>),
    Resize((u32, u32)),
    ChangeDpi(f64),
}
