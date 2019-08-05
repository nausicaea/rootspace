use ecs::EventTrait;

pub trait EngineEventTrait: EventTrait + Sized + 'static {
    fn startup() -> Self::EventFlag;
    fn shutdown() -> Self::EventFlag;
    fn hard_shutdown() -> Self::EventFlag;
    fn command() -> Self::EventFlag;
    fn resize() -> Self::EventFlag;
    fn change_dpi() -> Self::EventFlag;

    fn new_startup() -> Self;
    fn new_shutdown() -> Self;
    fn new_hard_shutdown() -> Self;
    fn new_command(args: Vec<String>) -> Self;
    fn new_resize(dims: (u32, u32)) -> Self;
    fn new_change_dpi(factor: f64) -> Self;

    fn flag(&self) -> Self::EventFlag;
    fn command_data(&self) -> Option<&[String]>;
    fn resize_data(&self) -> Option<(u32, u32)>;
    fn change_dpi_data(&self) -> Option<f64>;
}
