use std::{
    cmp::{max, min},
    collections::HashMap,
    ops::Range,
    time::{Duration, Instant},
};

use anyhow::Context;
use async_trait::async_trait;
use itertools::Itertools;
use wgpu::{DynamicOffset, SurfaceError};
use winit::{dpi::PhysicalSize, event::WindowEvent};

use crate::engine::resources::graphics::gpu_material::GpuMaterial;
use crate::{
    ecs::{
        component::Component,
        entity::index::Index,
        event_queue::{receiver_id::ReceiverId, EventQueue},
        resources::Resources,
        storage::Storage,
        system::System,
        with_resources::WithResources,
    },
    engine::{
        components::{camera::Camera, renderable::Renderable, transform::Transform},
        events::engine_event::EngineEvent,
        resources::{
            asset_database::AssetDatabase,
            graphics::{
                encoder::RenderPass,
                ids::{BindGroupId, BufferId, PipelineId},
                instance::Instance,
                vertex::Vertex,
                Graphics, TransformWrapper,
            },
            statistics::Statistics,
        },
    },
    glamour::{mat::Mat4, num::ToMatrix},
    rose_tree::hierarchy::Hierarchy,
};

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
    #[tracing::instrument(skip_all)]
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

    #[tracing::instrument(skip_all)]
    fn prepare<'a>(&mut self, res: &'a Resources) -> DrawData<'a> {
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

        // Calculate all camera transforms and the respective buffer offset
        let uniform_alignment = gfx.limits().min_uniform_buffer_offset_alignment; // 256 bytes
        let (uniform_buffer_offsets, cam_persp) = res
            .iter_r::<Camera>()
            .enumerate()
            .map(|(i, (idx, c))| {
                (
                    i,
                    c.as_persp_matrix() * hier_transform::<Transform>(idx, &hier, &transforms),
                )
            })
            .map(|(i, trf)| {
                let transform_offset = (i as DynamicOffset) * (uniform_alignment as DynamicOffset); // first 0x0, then 0x100
                (transform_offset, TransformWrapper(trf))
            })
            .fold(
                (Vec::<DynamicOffset>::new(), Vec::<TransformWrapper>::new()),
                |mut state, elem| {
                    state.0.push(elem.0);
                    state.1.push(elem.1);
                    state
                },
            );

        // Iterate through all entities with a renderable and transform
        // Extract all fields of Renderable that are shared across instances
        // Group the rest by instance buffer ID
        // Sort by instance ID
        // Convert the transforms to instances
        let mut instance_draw_data: Vec<InstanceDrawData> = Vec::new();
        let mut instance_buffer_data: HashMap<BufferId, Vec<Instance>> = HashMap::new();
        let res_groups = res
            .iter_rr::<Renderable, Transform>()
            .group_by(|(_, ren, _)| ren.model.mesh.instance_buffer);
        for (instance_buffer, data) in &res_groups {
            let mut vertex_buffer = None;
            let mut index_buffer = None;
            let mut num_indices = None;
            let mut materials = None;
            let mut min_instance_id = u32::MAX;
            let mut max_instance_id = u32::MIN;

            let instance_data: Vec<_> = data
                .sorted_by_key(|(_, ren, _)| ren.model.mesh.instance_id)
                .map(|(idx, ren, trf)| {
                    if vertex_buffer.is_none() {
                        vertex_buffer = Some(ren.model.mesh.vertex_buffer);
                    }
                    if index_buffer.is_none() {
                        index_buffer = Some(ren.model.mesh.index_buffer);
                    }
                    if num_indices.is_none() {
                        num_indices = Some(ren.model.mesh.num_indices);
                    }
                    if materials.is_none() {
                        materials = Some(&ren.model.materials);
                    }
                    min_instance_id = min(min_instance_id, ren.model.mesh.instance_id.to_u32());
                    max_instance_id = max(max_instance_id, ren.model.mesh.instance_id.to_u32());
                    Instance {
                        model: hier_transform::<Transform>(idx, &hier, &transforms).0,
                        with_camera: if trf.ui { 0.0 } else { 1.0 },
                    }
                })
                .collect();

            instance_draw_data.push(InstanceDrawData {
                vertex_buffer: vertex_buffer.unwrap(),
                instance_buffer,
                index_buffer: index_buffer.unwrap(),
                num_indices: num_indices.unwrap(),
                materials: materials.unwrap(),
                instance_ids: min_instance_id..(max_instance_id + 1),
            });

            instance_buffer_data.insert(instance_buffer, instance_data);
        }

        // Write the camera transforms to the corresponding uniform buffer
        gfx.write_buffer(self.transform_buffer, unsafe {
            std::slice::from_raw_parts(
                cam_persp.as_ptr() as *const u8,
                cam_persp.len() * uniform_alignment as usize,
            )
        });

        // Update the instance buffers
        for (instance_buffer, instance_data) in instance_buffer_data {
            gfx.write_buffer(instance_buffer, unsafe {
                std::slice::from_raw_parts(
                    instance_data.as_ptr() as *const u8,
                    instance_data.len() * std::mem::size_of::<Instance>(),
                )
            });
        }

        DrawData {
            uniform_buffer_offsets,
            instance_draw_data,
        }
    }

    #[tracing::instrument(skip_all)]
    fn draw(&mut self, draw_data: &DrawData, mut rp: RenderPass) -> usize {
        let mut draw_calls = 0;

        for instance_data in &draw_data.instance_draw_data {
            for &transform_offset in &draw_data.uniform_buffer_offsets {
                draw_calls += 1;
                if instance_data.materials.is_empty() {
                    rp.set_pipeline(self.pipeline_wt)
                        .set_bind_group(0, self.transform_bind_group, &[transform_offset])
                        .set_vertex_buffer(0, instance_data.vertex_buffer)
                        .set_vertex_buffer(1, instance_data.instance_buffer)
                        .set_index_buffer(instance_data.index_buffer)
                        .draw_indexed(0..instance_data.num_indices, 0, instance_data.instance_ids.clone());
                } else {
                    rp.set_pipeline(self.pipeline_wtm)
                        .set_bind_group(0, self.transform_bind_group, &[transform_offset])
                        .set_bind_group(1, instance_data.materials[0].bind_group, &[])
                        .set_vertex_buffer(0, instance_data.vertex_buffer)
                        .set_vertex_buffer(1, instance_data.instance_buffer)
                        .set_index_buffer(instance_data.index_buffer)
                        .draw_indexed(0..instance_data.num_indices, 0, instance_data.instance_ids.clone());
                }
            }
        }

        draw_calls
    }

    #[tracing::instrument(skip_all)]
    fn on_window_resized(&self, res: &Resources, ps: PhysicalSize<u32>) {
        tracing::trace!("Resizing surface");
        res.write::<Graphics>().resize(ps)
    }

    #[tracing::instrument(skip_all)]
    fn on_surface_outdated(&self, res: &Resources) {
        tracing::trace!("Surface is outdated");
        res.write::<Graphics>().reconfigure()
    }

    #[tracing::instrument(skip_all)]
    fn on_out_of_memory(&self, res: &Resources) {
        tracing::error!("surface is out of memory");
        res.write::<EventQueue<EngineEvent>>().send(EngineEvent::Exit)
    }

    #[tracing::instrument(skip_all)]
    fn on_timeout(&self) {
        tracing::warn!("Surface timed out")
    }

    #[tracing::instrument(skip_all)]
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
            .add_vertex_buffer_layout::<Instance>()
            .submit();

        Ok(pipeline)
    }

    #[tracing::instrument(skip_all)]
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
            .add_vertex_buffer_layout::<Instance>()
            .submit();

        Ok(pipeline)
    }
}

impl WithResources for Renderer {
    #[tracing::instrument(skip_all)]
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
    #[tracing::instrument(skip_all)]
    async fn run(&mut self, res: &Resources, _t: Duration, _dt: Duration) {
        let frame_start = Instant::now();
        self.handle_events(res);

        if !self.renderer_enabled {
            res.write::<Statistics>().update_render_stats(0, frame_start.elapsed(), Duration::ZERO, Duration::ZERO, Duration::ZERO);
            return;
        }

        let prepare_start = Instant::now();
        let draw_data = self.prepare(res);
        let prepare_duration = prepare_start.elapsed();

        let gfx = res.read::<Graphics>();
        let encoder = gfx.create_encoder(Some("main-encoder"));
        let (draw_calls, draw_duration, submit_duration) = match encoder {
            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                self.on_surface_outdated(res);
                (0, Duration::ZERO, Duration::ZERO)
            },
            Err(SurfaceError::OutOfMemory) => {
                self.on_out_of_memory(res);
                (0, Duration::ZERO, Duration::ZERO)
            },
            Err(SurfaceError::Timeout) => {
                self.on_timeout();
                (0, Duration::ZERO, Duration::ZERO)
            },
            Ok(mut enc) => {
                let draw_start = Instant::now();
                let draw_calls = self.draw(&draw_data, enc.begin(Some("main-render-pass")));
                let draw_duration = draw_start.elapsed();
                let submit_start = Instant::now();
                enc.submit();
                let submit_duration = submit_start.elapsed();
                (draw_calls, draw_duration, submit_duration)
            }
        };

        res.write::<Statistics>().update_render_stats(
            draw_calls,
            frame_start.elapsed(),
            prepare_duration,
            draw_duration,
            submit_duration,
        );
    }
}

#[derive(Debug)]
struct DrawData<'a> {
    uniform_buffer_offsets: Vec<DynamicOffset>,
    instance_draw_data: Vec<InstanceDrawData<'a>>,
}

#[derive(Debug)]
struct InstanceDrawData<'a> {
    vertex_buffer: BufferId,
    instance_buffer: BufferId,
    index_buffer: BufferId,
    num_indices: u32,
    materials: &'a [GpuMaterial],
    instance_ids: Range<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{engine::resources::asset_database::AssetDatabaseDeps, Reg};

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
