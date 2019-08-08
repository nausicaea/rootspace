#[cfg_attr(feature = "diagnostics", derive(TypeName))]
#[derive(Debug, Clone, PartialEq)]
pub enum EngineEvent {
    Startup,
    Shutdown,
    HardShutdown,
    Command(Vec<String>),
    Resize((u32, u32)),
    ChangeDpi(f64),
}
