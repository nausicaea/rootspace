use anyhow::Context;
use log::{debug, error, warn};
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;

use crate::ecs::event_queue::receiver_id::ReceiverId;
use crate::ecs::event_queue::EventQueue;
use crate::ecs::resources::Resources;
use crate::ecs::system::System;
use crate::ecs::with_resources::WithResources;
use crate::engine::components::camera::Camera;
use crate::engine::components::renderable::Renderable;
use crate::engine::components::transform::Transform;
use crate::engine::events::engine_event::EngineEvent;
use crate::engine::events::window_event::WindowEvent;
use crate::engine::resources::asset_database::AssetDatabase;
use crate::engine::resources::graphics::encoder::RenderPass;
use crate::engine::resources::graphics::ids::{BindGroupId, BufferId, PipelineId};
use crate::engine::resources::graphics::vertex::Vertex;
use crate::engine::resources::graphics::Graphics;
use crate::glamour::mat::Mat4;

#[derive(Debug)]
pub struct Renderer {
    window_receiver: ReceiverId<WindowEvent>,
    engine_receiver: ReceiverId<EngineEvent>,
    renderer_enabled: bool,
    transform_buffer: BufferId,
    transform_bind_group: BindGroupId,
    pipeline_wt: PipelineId,
    pipeline_wtm: PipelineId,
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

    fn render(&self, res: &Resources, mut rp: RenderPass) {
        let gfx = res.borrow::<Graphics>();

        for (c, ct) in res.iter_rr::<Camera, Transform>() {
            let c_mat = c.as_matrix() * ct.to_matrix();

            for (r, t) in res.iter_rr::<Renderable, Transform>() {
                let t_mat = c_mat * t.to_matrix();

                // panic!(
                //     "\np={:#}\nv={:#}\nm={:#}\npvm={:#}\nt1={:}\nt2={:}\nt3={:}",
                //     c.as_matrix(), ct.to_matrix(), t.to_matrix(), t_mat,
                //     t_mat * glamour::Vec4::new(0.0, 0.5, 0.5, 1.0),
                //     t_mat * glamour::Vec4::new(-0.5, -0.5, 0.0, 1.0),
                //     t_mat * glamour::Vec4::new(0.5, -0.5, 0.0, 1.0),
                // );

                gfx.write_buffer(self.transform_buffer, t_mat.as_ref());

                if r.0.materials.is_empty() {
                    rp.set_pipeline(self.pipeline_wt)
                        .set_bind_group(0, self.transform_bind_group)
                        .set_vertex_buffer(0, r.0.mesh.vertex_buffer)
                        .set_index_buffer(r.0.mesh.index_buffer)
                        .draw_indexed(0..r.0.mesh.num_indices, 0, 0..1);
                } else {
                    rp.set_pipeline(self.pipeline_wtm)
                        .set_bind_group(0, self.transform_bind_group)
                        .set_bind_group(1, r.0.materials[0].bind_group)
                        .set_vertex_buffer(0, r.0.mesh.vertex_buffer)
                        .set_index_buffer(r.0.mesh.index_buffer)
                        .draw_indexed(0..r.0.mesh.num_indices, 0, 0..1);
                }
            }
        }
    }

    fn on_window_resized(&self, res: &Resources, ps: PhysicalSize<u32>) {
        res.borrow_mut::<Graphics>().resize(ps)
    }

    fn on_surface_outdated(&self, res: &Resources) {
        res.borrow_mut::<Graphics>().reconfigure()
    }

    fn on_out_of_memory(&self, res: &Resources) {
        error!("WGPU surface is out of memory");
        res.borrow_mut::<EventQueue<EngineEvent>>()
            .send(EngineEvent::AbortRequested)
    }

    fn on_timeout(&self) {
        log::warn!("WGPU surface timed out")
    }

    fn crp_with_transform(
        adb: &AssetDatabase,
        gfx: &mut Graphics,
        label: &'static str,
    ) -> Result<PipelineId, anyhow::Error> {
        let shader_path = adb.find_asset("shaders", "transformed.wgsl")?;
        let shader_data = std::fs::read_to_string(&shader_path)
            .with_context(|| format!("trying to load a shader source from '{}'", shader_path.display()))?;
        let vertex_shader_module = gfx.create_shader_module(Some("vertex-shader"), shader_data);

        let shader_path = adb.find_asset("shaders", "with_static_color.wgsl")?;
        let shader_data = std::fs::read_to_string(&shader_path)
            .with_context(|| format!("trying to load a shader source from '{}'", shader_path.display()))?;
        let fragment_shader_module = gfx.create_shader_module(Some("fragment-shader"), shader_data);

        let tl = gfx.transform_layout();

        let pipeline = gfx
            .create_render_pipeline()
            .with_label(label)
            .add_bind_group_layout(tl)
            .with_vertex_shader_module(vertex_shader_module, "main")
            .with_fragment_shader_module(fragment_shader_module, "main")
            .add_vertex_buffer_layout::<Vertex>()
            .submit();

        Ok(pipeline)
    }

    fn crp_with_transform_and_material(
        adb: &AssetDatabase,
        gfx: &mut Graphics,
        label: &'static str,
    ) -> Result<PipelineId, anyhow::Error> {
        let shader_path = adb.find_asset("shaders", "transformed.wgsl")?;
        let shader_data = std::fs::read_to_string(&shader_path)
            .with_context(|| format!("trying to load a shader source from '{}'", shader_path.display()))?;
        let vertex_shader_module = gfx.create_shader_module(Some("vertex-shader"), shader_data);

        let shader_path = adb.find_asset("shaders", "textured.wgsl")?;
        let shader_data = std::fs::read_to_string(&shader_path)
            .with_context(|| format!("trying to load a shader source from '{}'", shader_path.display()))?;
        let fragment_shader_module = gfx.create_shader_module(Some("fragment-shader"), shader_data);

        let tl = gfx.transform_layout();
        let ml = gfx.material_layout();

        let pipeline = gfx
            .create_render_pipeline()
            .with_label(label)
            .add_bind_group_layout(tl)
            .add_bind_group_layout(ml)
            .with_vertex_shader_module(vertex_shader_module, "main")
            .with_fragment_shader_module(fragment_shader_module, "main")
            .add_vertex_buffer_layout::<Vertex>()
            .submit();

        Ok(pipeline)
    }
}

impl WithResources for Renderer {
    fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        let window_receiver = res.borrow_mut::<EventQueue<WindowEvent>>().subscribe::<Self>();
        let engine_receiver = res.borrow_mut::<EventQueue<EngineEvent>>().subscribe::<Self>();

        let adb = res.borrow::<AssetDatabase>();
        let mut gfx = res.borrow_mut::<Graphics>();

        let pipeline_wtm = Self::crp_with_transform_and_material(&adb, &mut gfx, "wtm")
            .context("trying to create the render pipeline 'wtm'")?;
        let pipeline_wt =
            Self::crp_with_transform(&adb, &mut gfx, "wt").context("trying to create the render pipeline 'wt'")?;

        let transform_buffer = gfx.create_buffer(
            Some("transform-buffer"),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            Mat4::<f32>::identity().as_ref(),
        );

        let tl = gfx.transform_layout();
        let transform_bind_group = gfx
            .create_bind_group(tl)
            .with_label("transform-bind-group")
            .add_buffer(0, transform_buffer)
            .submit();

        Ok(Renderer {
            window_receiver,
            engine_receiver,
            renderer_enabled: true,
            transform_buffer,
            transform_bind_group,
            pipeline_wtm,
            pipeline_wt,
        })
    }
}

impl System for Renderer {
    fn run(&mut self, res: &Resources, _t: &std::time::Duration, _dt: &std::time::Duration) {
        self.handle_events(res);

        if !self.renderer_enabled {
            return;
        }

        let gfx = res.borrow::<Graphics>();

        let encoder = gfx.create_encoder(Some("main-encoder"));

        match encoder {
            Err(SurfaceError::Lost | SurfaceError::Outdated) => self.on_surface_outdated(res),
            Err(SurfaceError::OutOfMemory) => self.on_out_of_memory(res),
            Err(SurfaceError::Timeout) => self.on_timeout(),
            Ok(mut enc) => {
                self.render(&res, enc.begin(Some("main-render-pass")));
                enc.submit();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::resources::asset_database::AssetDatabaseDeps;
    use crate::Reg;

    use super::*;

    struct TDeps<'a> {
        name: &'a str,
        force_init: bool,
        within_repo: bool,
    }

    impl Default for TDeps<'static> {
        fn default() -> Self {
            TDeps {
                name: "test",
                force_init: false,
                within_repo: false,
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

        fn within_repo(&self) -> bool {
            self.within_repo
        }
    }

    #[test]
    fn renderer_reg_macro() {
        type _SR = Reg![Renderer];
    }
}
