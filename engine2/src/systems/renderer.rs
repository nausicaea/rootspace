use ecs::{EventQueue, ReceiverId, Resources, System, WithResources};

use crate::{
    events::{engine_event::EngineEvent, window_event::WindowEvent},
    resources::{graphics::{ids::PipelineId, Graphics}, asset_database::AssetDatabase},
};

#[derive(Debug)]
pub struct Renderer {
    window_receiver: ReceiverId<WindowEvent>,
    engine_receiver: ReceiverId<EngineEvent>,
    renderer_enabled: bool,
    pipeline: PipelineId,
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

        let shader_path = res.borrow::<AssetDatabase>()
            .find_asset("shaders/triangle.wgsl")?;
        let shader_data = shader_path.read_to_string()?;

        let pipeline = res.borrow_mut::<Graphics>()
            .create_render_pipeline()
            .with_shader_module(None, &shader_data, "vs_main", Some("fs_main"))
            .submit(None)?;

        Ok(Renderer {
            window_receiver,
            engine_receiver,
            renderer_enabled: true,
            pipeline,
        })
    }
}

impl System for Renderer {
    fn run(&mut self, res: &ecs::Resources, _t: &std::time::Duration, _dt: &std::time::Duration) {
        self.handle_events(res);

        if !self.renderer_enabled {
            return;
        }

        res.borrow::<Graphics>().create_render_pass()
            .with_pipeline(&self.pipeline)
            .commit()
            .unwrap();
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
