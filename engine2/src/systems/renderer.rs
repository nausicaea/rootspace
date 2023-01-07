use ecs::{EventQueue, ReceiverId, System, WithResources, Resources};

use crate::{
    events::window_event::WindowEvent,
    resources::{
        graphics::{ids::PipelineId, Graphics},
    },
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
    fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        let receiver_id = res.borrow_mut::<EventQueue<WindowEvent>>().subscribe::<Self>();
        Ok(Renderer(receiver_id, None))
    }
}

impl System for Renderer {
    fn run(&mut self, res: &ecs::Resources, _t: &std::time::Duration, _dt: &std::time::Duration) {
        self.handle_events(res);

        res.borrow::<Graphics>().render(|_rp| ()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use ecs::{SystemRegistry, Reg, End, World};

    use super::*;

    #[test]
    fn renderer_reg_macro() {
        type _SR = Reg![Renderer];
    }

    #[test]
    fn renderer_system_registry() {
        let res = Resources::with_dependencies::<Reg![EventQueue<WindowEvent>], _>(&()).unwrap();
        let _rr = SystemRegistry::push(End, Renderer::with_res(&res).unwrap());
    }

    #[test]
    fn renderer_world() {
        let _w = World::with_dependencies::<Reg![EventQueue<WindowEvent>], Reg![], Reg![Renderer], Reg![], _>(&()).unwrap();
    }
}
