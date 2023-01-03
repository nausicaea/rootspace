use ecs::{System, SerializationName, EventQueue, ReceiverId, WithResources};

use crate::{resources::{statistics::Statistics, graphics::Graphics}, events::window_event::WindowEvent};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Renderer(ReceiverId<WindowEvent>);

impl WithResources for Renderer {
    fn with_resources(res: &ecs::Resources) -> Self {
        let receiver_id = res.borrow_mut::<EventQueue<WindowEvent>>().subscribe::<Self>();
        Renderer(receiver_id)
    }
}

impl SerializationName for Renderer {}

impl System for Renderer {
    fn run(&mut self, res: &ecs::Resources, t: &std::time::Duration, dt: &std::time::Duration) {
        let events = res.borrow_mut::<EventQueue<WindowEvent>>().receive(&self.0);
        for event in events {
            match event {
                WindowEvent::Resized(ps) => res.borrow_mut::<Graphics>().resize(ps),
                _ => (),
            }
        }
        // TODO
    }
}
