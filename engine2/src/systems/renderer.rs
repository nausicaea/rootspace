use ecs::{EventQueue, ReceiverId, Resources, System, WithResources};
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;

use crate::{
    events::{engine_event::EngineEvent, window_event::WindowEvent},
    resources::{
        asset_database::AssetDatabase,
        graphics::{ids::PipelineId, Graphics},
    },
};

#[derive(Debug)]
pub struct Renderer {
    window_receiver: ReceiverId<WindowEvent>,
    engine_receiver: ReceiverId<EngineEvent>,
    renderer_enabled: bool,
    pipeline: PipelineId,
}

impl Renderer {
    fn handle_events(&mut self, res: &Resources) {
        res
            .borrow_mut::<EventQueue<WindowEvent>>()
            .receive_cb(&self.window_receiver, |e| {
                match e {
                    WindowEvent::Resized(ps) => self.on_window_resized(res, *ps),
                    _ => (),
                }
            });

        res
            .borrow_mut::<EventQueue<EngineEvent>>()
            .receive_cb(&self.engine_receiver, |e| {
                match e {
                    EngineEvent::AbortRequested => self.renderer_enabled = true,
                    _ => (),
                }
            });
    }

    fn on_window_resized(&self, res: &Resources, ps: PhysicalSize<u32>) {
        res.borrow_mut::<Graphics>().resize(ps)
    }

    fn on_surface_outdated(&self, res: &Resources) {
        res.borrow_mut::<Graphics>().reconfigure()
    }

    fn on_out_of_memory(&self, res: &Resources) {
        log::error!("WGPU surface is out of memory");
        res.borrow_mut::<EventQueue<EngineEvent>>()
            .send(EngineEvent::AbortRequested)
    }

    fn on_timeout(&self) {
        log::warn!("WGPU surface timed out")
    }
}

impl WithResources for Renderer {
    fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        let window_receiver = res.borrow_mut::<EventQueue<WindowEvent>>().subscribe::<Self>();
        let engine_receiver = res.borrow_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();

        let shader_path = res.borrow::<AssetDatabase>().find_asset("shaders/triangle.wgsl")?;
        let shader_data = shader_path.read_to_string()?;

        let mut gfx = res.borrow_mut::<Graphics>();
        // let bind_group_layout = gfx.create_bind_group_layout(None, &[]);
        let pipeline = gfx
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

        let r = res
            .borrow::<Graphics>()
            .create_render_pass()
            .with_pipeline(&self.pipeline)
            .submit(None, None);

        match r {
            Err(SurfaceError::Lost | SurfaceError::Outdated) => self.on_surface_outdated(res),
            Err(SurfaceError::OutOfMemory) => self.on_out_of_memory(res),
            Err(SurfaceError::Timeout) => self.on_timeout(),
            Ok(_) => (),
        }
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
