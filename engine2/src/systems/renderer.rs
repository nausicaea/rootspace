use ecs::{EventQueue, ReceiverId, Resources, System, WithResources};
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;

use crate::{
    events::{engine_event::EngineEvent, window_event::WindowEvent},
    resources::{
        asset_database::AssetDatabase,
        graphics::{
            ids::{BindGroupId, BufferId, PipelineId, ShaderModuleId},
            Graphics,
        },
    },
};

#[derive(Debug)]
pub struct Renderer {
    window_receiver: ReceiverId<WindowEvent>,
    engine_receiver: ReceiverId<EngineEvent>,
    renderer_enabled: bool,
    vertex_shader_module: ShaderModuleId,
    fragment_shader_module: ShaderModuleId,
    transform_buffer_data: glamour::Mat4<f32>,
    transform_buffer: BufferId,
    transform_bind_group: BindGroupId,
    pipeline: PipelineId,
}

impl Renderer {
    fn handle_events(&mut self, res: &Resources) {
        res.borrow_mut::<EventQueue<WindowEvent>>()
            .receive_cb(&self.window_receiver, |e| match e {
                WindowEvent::Resized(ps) => self.on_window_resized(res, *ps),
                _ => (),
            });

        res.borrow_mut::<EventQueue<EngineEvent>>()
            .receive_cb(&self.engine_receiver, |e| match e {
                EngineEvent::AbortRequested => self.renderer_enabled = true,
                _ => (),
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

        let mut gfx = res.borrow_mut::<Graphics>();

        let shader_path = res
            .borrow::<AssetDatabase>()
            .find_asset("shaders", "transformed.wgsl")?;
        let shader_data = shader_path.read_to_string()?;
        let vertex_shader_module = gfx.create_shader_module(None, shader_data);
        let shader_path = res.borrow::<AssetDatabase>().find_asset("shaders", "textured.wgsl")?;
        let shader_data = shader_path.read_to_string()?;
        let fragment_shader_module = gfx.create_shader_module(None, shader_data);

        let tl = gfx.transform_layout();
        let ml = gfx.material_layout();
        let pipeline = gfx
            .create_render_pipeline()
            .add_bind_group_layout(tl)
            .add_bind_group_layout(ml)
            .with_vertex_shader_module(vertex_shader_module, "main")
            .with_fragment_shader_module(fragment_shader_module, "main")
            .submit();

        let transform_buffer_data = glamour::Mat4::<f32>::identity();
        let transform_buffer = gfx.create_buffer(
            None,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            transform_buffer_data.as_slice(),
        );

        let transform_bind_group = gfx.create_bind_group(tl).add_buffer(0, transform_buffer).submit();

        Ok(Renderer {
            window_receiver,
            engine_receiver,
            renderer_enabled: true,
            vertex_shader_module,
            fragment_shader_module,
            transform_buffer_data,
            transform_buffer,
            transform_bind_group,
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
            .with_pipeline(self.pipeline)
            .add_bind_group(0, self.transform_bind_group)
            .draw(0..3, 0..3)
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
    use ecs::Reg;

    use super::*;
    use crate::resources::asset_database::AssetDatabaseDeps;

    struct TDeps<'a> {
        name: &'a str,
        force_init: bool,
    }

    impl Default for TDeps<'static> {
        fn default() -> Self {
            TDeps {
                name: "test",
                force_init: false,
            }
        }
    }

    impl<'a> AssetDatabaseDeps for TDeps<'a> {
        fn name(&self) -> &str {
            self.name
        }

        fn force_init(&self) -> bool {
            self.force_init
        }
    }

    #[test]
    fn renderer_reg_macro() {
        type _SR = Reg![Renderer];
    }
}
