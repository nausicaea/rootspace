use ecs::{Resource, SerializationName};
use winit::window::Window;

#[derive(Debug, Default)]
pub struct Graphics {
}

impl Graphics {
    pub async fn initialize(&mut self, window: Window) {
    }
}

impl Resource for Graphics {}

impl SerializationName for Graphics {}
