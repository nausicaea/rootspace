use crate::ecs::component::Component;
use crate::ecs::entity::index::Index;
use crate::engine::components::ui_transform::UiTransform;
use anyhow::Context;
use async_trait::async_trait;
use log::{error, trace, warn};
use std::time::{Duration, Instant};
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;

use crate::ecs::event_queue::receiver_id::ReceiverId;
use crate::ecs::event_queue::EventQueue;
use crate::ecs::resources::Resources;
use crate::ecs::storage::Storage;
use crate::ecs::system::System;
use crate::ecs::with_resources::WithResources;
use crate::engine::components::camera::Camera;
use crate::engine::components::renderable::Renderable;
use crate::engine::components::transform::Transform;
use crate::engine::events::engine_event::EngineEvent;
use crate::engine::resources::asset_database::AssetDatabase;
use crate::engine::resources::graphics::encoder::RenderPass;
use crate::engine::resources::graphics::ids::{BindGroupId, BufferId, PipelineId};
use crate::engine::resources::graphics::vertex::Vertex;
use crate::engine::resources::graphics::{Graphics, TransformWrapper};
use crate::engine::resources::statistics::Statistics;
use crate::glamour::mat::Mat4;
use crate::glamour::num::ToMatrix;
use crate::rose_tree::hierarchy::Hierarchy;

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
        res.write::<EventQueue<WindowEvent>>()
            .receive_cb(&self.window_receiver, |e| {
                if let WindowEvent::Resized(ps) = e {
                    self.on_window_resized(res, *ps)
                }
            });

        res.write::<EventQueue<EngineEvent>>()
            .receive_cb(&self.engine_receiver, |e| {
                #[allow(irrefutable_let_patterns)]
                if let EngineEvent::Exit = e {
                    self.renderer_enabled = false
                }
            });
    }

    fn render(&mut self, res: &Resources, mut rp: RenderPass) {
        fn hier_transform<C: Component + ToMatrix<f32>>(
            idx: Index,
            hier: &Hierarchy<Index>,
            transforms: &C::Storage,
        ) -> Mat4<f32> {
            hier.ancestors(idx)
                .filter_map(|a| transforms.get(a).map(|at| at.to_matrix()))
                .product::<Mat4<f32>>()
        }

        let gfx = res.read::<Graphics>();
        let hier = res.read::<Hierarchy<Index>>();
        let transforms = res.read_components::<Transform>();
        let ui_transforms = res.read_components::<UiTransform>();

        let (cam_ortho, cam_persp) = res
            .iter_r::<Camera>()
            .map(|(idx, c)| {
                (
                    hier_transform::<UiTransform>(idx, &hier, &ui_transforms),
                    c.as_matrix() * hier_transform::<Transform>(idx, &hier, &transforms),
                )
            })
            .fold((Vec::new(), Vec::new()), |(mut ortho, mut persp), (o, p)| {
                ortho.push(o);
                persp.push(p);
                (ortho, persp)
            });

        let (renderables, transforms) = cam_persp
            .iter()
            .flat_map(|cm| {
                res.iter_rr::<Renderable, Transform>()
                    .map(|(idx, r, _)| (idx, r, *cm * hier_transform::<Transform>(idx, &hier, &transforms)))
            })
            .chain(cam_ortho.iter().flat_map(|cm| {
                res.iter_rr::<Renderable, UiTransform>()
                    .map(|(idx, r, _)| (idx, r, *cm * hier_transform::<UiTransform>(idx, &hier, &ui_transforms)))
            }))
            .fold(
                (Vec::new(), Vec::new()),
                |(mut renderables, mut transforms), (idx, r, t)| {
                    renderables.push((idx, r));
                    transforms.push(TransformWrapper(t));
                    (renderables, transforms)
                },
            );

        let uniform_alignment = gfx.limits().min_uniform_buffer_offset_alignment; // 256 bytes
        gfx.write_buffer(self.transform_buffer, unsafe {
            std::slice::from_raw_parts(
                transforms.as_ptr() as *const u8,
                transforms.len() * uniform_alignment as usize,
            )
        });

        let world_draw_calls = renderables.len();
        for (i, (_, r)) in renderables.into_iter().enumerate() {
            let transform_offset = (i as wgpu::DynamicOffset) * (uniform_alignment as wgpu::DynamicOffset); // first 0x0, then 0x100
            if r.model.materials.is_empty() {
                rp.set_pipeline(self.pipeline_wt)
                    .set_bind_group(0, self.transform_bind_group, &[transform_offset])
                    .set_vertex_buffer(0, r.model.mesh.vertex_buffer)
                    .set_index_buffer(r.model.mesh.index_buffer)
                    .draw_indexed(0..r.model.mesh.num_indices, 0, 0..1);
            } else {
                rp.set_pipeline(self.pipeline_wtm)
                    .set_bind_group(0, self.transform_bind_group, &[transform_offset])
                    .set_bind_group(1, r.model.materials[0].bind_group, &[])
                    .set_vertex_buffer(0, r.model.mesh.vertex_buffer)
                    .set_index_buffer(r.model.mesh.index_buffer)
                    .draw_indexed(0..r.model.mesh.num_indices, 0, 0..1);
            }
        }
        res.write::<Statistics>().update_draw_calls(world_draw_calls, 0);
    }

    fn on_window_resized(&self, res: &Resources, ps: PhysicalSize<u32>) {
        trace!("Resizing surface");
        res.write::<Graphics>().resize(ps)
    }

    fn on_surface_outdated(&self, res: &Resources) {
        trace!("Surface is outdated");
        res.write::<Graphics>().reconfigure()
    }

    fn on_out_of_memory(&self, res: &Resources) {
        error!("surface is out of memory");
        res.write::<EventQueue<EngineEvent>>().send(EngineEvent::Exit)
    }

    fn on_timeout(&self) {
        warn!("Surface timed out")
    }

    fn crp_with_transform(
        adb: &AssetDatabase,
        gfx: &mut Graphics,
        label: &'static str,
    ) -> Result<PipelineId, anyhow::Error> {
        let shader_path = adb.find_asset("shaders", "transformed.wgsl")?;
        let shader_data = std::fs::read_to_string(&shader_path)
            .with_context(|| format!("Loading a shader source from '{}'", shader_path.display()))?;
        let vertex_shader_module = gfx.create_shader_module(Some("vertex-shader"), shader_data);

        let shader_path = adb.find_asset("shaders", "with_static_color.wgsl")?;
        let shader_data = std::fs::read_to_string(&shader_path)
            .with_context(|| format!("Loading a shader source from '{}'", shader_path.display()))?;
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
            .with_context(|| format!("Loading a shader source from '{}'", shader_path.display()))?;
        let vertex_shader_module = gfx.create_shader_module(Some("vertex-shader"), shader_data);

        let shader_path = adb.find_asset("shaders", "textured.wgsl")?;
        let shader_data = std::fs::read_to_string(&shader_path)
            .with_context(|| format!("Loading a shader source from '{}'", shader_path.display()))?;
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
    async fn with_res(res: &Resources) -> Result<Self, anyhow::Error> {
        let window_receiver = res.write::<EventQueue<WindowEvent>>().subscribe::<Self>();
        let engine_receiver = res.write::<EventQueue<EngineEvent>>().subscribe::<Self>();

        let adb = res.read::<AssetDatabase>();
        let mut gfx = res.write::<Graphics>();

        let pipeline_wtm = Self::crp_with_transform_and_material(&adb, &mut gfx, "wtm")
            .context("Creating the render pipeline 'wtm'")?;
        let pipeline_wt =
            Self::crp_with_transform(&adb, &mut gfx, "wt").context("Creating the render pipeline 'wt'")?;

        let max_objects = gfx.max_objects();
        let uniform_alignment = gfx.limits().min_uniform_buffer_offset_alignment; // 256
        let buffer_size = (max_objects * uniform_alignment) as wgpu::BufferAddress; // 268'435'456
        let transform_buffer = gfx.create_buffer(
            Some("transform-buffer"),
            buffer_size,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );

        let binding_size = wgpu::BufferSize::new(std::mem::size_of::<TransformWrapper>() as _);
        let tl = gfx.transform_layout();
        let transform_bind_group = gfx
            .create_bind_group(tl)
            .with_label(Some("transform-bind-group"))
            .add_buffer(0, 0, binding_size, transform_buffer)
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

#[async_trait]
impl System for Renderer {
    async fn run(&mut self, res: &Resources, _t: Duration, _dt: Duration) {
        let frame_start = Instant::now();
        self.handle_events(res);

        if !self.renderer_enabled {
            return;
        }

        let gfx = res.read::<Graphics>();

        let encoder = gfx.create_encoder(Some("main-encoder"));

        match encoder {
            Err(SurfaceError::Lost | SurfaceError::Outdated) => self.on_surface_outdated(res),
            Err(SurfaceError::OutOfMemory) => self.on_out_of_memory(res),
            Err(SurfaceError::Timeout) => self.on_timeout(),
            Ok(mut enc) => {
                self.render(res, enc.begin(Some("main-render-pass")));
                enc.submit();
            }
        }

        res.write::<Statistics>().update_render_durations(frame_start.elapsed());
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
