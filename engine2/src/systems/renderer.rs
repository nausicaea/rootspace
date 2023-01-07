use ecs::{EventQueue, ReceiverId, Resources, System, WithResources};

use crate::{
    events::{engine_event::EngineEvent, window_event::WindowEvent},
    resources::graphics::{ids::PipelineId, Graphics},
};

#[derive(Debug)]
pub struct Renderer {
    window_receiver: ReceiverId<WindowEvent>,
    engine_receiver: ReceiverId<EngineEvent>,
    renderer_enabled: bool,
    pipeline: Option<PipelineId>,
}

impl Renderer {
    fn handle_events(&mut self, res: &ecs::Resources) {
        let events = res
            .borrow_mut::<EventQueue<WindowEvent>>()
            .receive(&self.window_receiver);
        for event in events {
            match event {
                WindowEvent::Resized(ps) => res.borrow_mut::<Graphics>().resize(ps),
                _ => (),
            }
        }

        let events = res
            .borrow_mut::<EventQueue<EngineEvent>>()
            .receive(&self.engine_receiver);
        for event in events {
            match event {
                EngineEvent::AbortRequested => self.renderer_enabled = false,
                _ => (),
            }
        }
    }
}

impl WithResources for Renderer {
    fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        let window_receiver = res.borrow_mut::<EventQueue<WindowEvent>>().subscribe::<Self>();
        let engine_receiver = res.borrow_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();
        Ok(Renderer {
            window_receiver,
            engine_receiver,
            renderer_enabled: true,
            pipeline: None,
        })
    }
}

impl System for Renderer {
    fn run(&mut self, res: &ecs::Resources, _t: &std::time::Duration, _dt: &std::time::Duration) {
        self.handle_events(res);

        if !self.renderer_enabled {
            return;
        }

        log::trace!("Renderer backend called");
        res.borrow::<Graphics>().render(|_rp| ()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use ecs::{End, Reg, SystemRegistry, World};

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
        let _w =
            World::with_dependencies::<Reg![EventQueue<WindowEvent>], Reg![], Reg![Renderer], Reg![], _>(&()).unwrap();
    }
}
