use ecs::{EventQueue, ReceiverId, SerializationName, System, WithResources};

use crate::{
    events::window_event::WindowEvent,
    resources::{graphics::{Graphics, ids::PipelineId}, statistics::Statistics},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Renderer(ReceiverId<WindowEvent>, #[serde(skip)] Option<PipelineId>);

impl Renderer {
    fn handle_events(&self, res: &ecs::Resources) {
        let events = res.borrow_mut::<EventQueue<WindowEvent>>().receive(&self.0);
        for event in events {
            match event {
                WindowEvent::Resized(ps) => res.borrow_mut::<Graphics>().resize(ps),
                _ => (),
            }
        }
    }
}

impl WithResources for Renderer {
    fn with_resources(res: &ecs::Resources) -> Self {
        let receiver_id = res.borrow_mut::<EventQueue<WindowEvent>>().subscribe::<Self>();
        Renderer(receiver_id, None)
    }
}

impl SerializationName for Renderer {}

impl System for Renderer {
    fn run(&mut self, res: &ecs::Resources, _t: &std::time::Duration, _dt: &std::time::Duration) {
        self.handle_events(res);
        
        res.borrow::<Graphics>().render(|_rp| ()).unwrap();
    }
}
