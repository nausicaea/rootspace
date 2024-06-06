use std::{
    cmp::{max, min},
    collections::HashMap,
    mem::size_of,
    ops::Range,
    slice::from_raw_parts,
    time::{Duration, Instant},
};

use num_traits::Inv;

use anyhow::Context;
use async_trait::async_trait;
use itertools::Itertools;
use wgpu::{BufferAddress, BufferSize, BufferUsages, SurfaceError};
use winit::{dpi::PhysicalSize, event::WindowEvent};

use crate::{
    components::light::Light,
    resources::graphics::{camera_uniform::CameraUniform, gpu_material::GpuMaterial, light_uniform::LightUniform},
};
use crate::{
    components::{camera::Camera, renderable::Renderable, transform::Transform},
    events::engine_event::EngineEvent,
    resources::{
        asset_database::AssetDatabase,
        graphics::{
            encoder::RenderPass,
            ids::{BindGroupId, BufferId, PipelineId},
            instance::Instance,
            vertex::Vertex,
            Graphics,
        },
        statistics::Statistics,
    },
};
use ecs::{
    component::Component,
    entity::index::Index,
    event_queue::{receiver_id::ReceiverId, EventQueue},
    resources::Resources,
    storage::Storage,
    system::System,
    with_resources::WithResources,
};
use glamour::{affine::builder::AffineBuilder, mat::Mat4};
use rose_tree::hierarchy::Hierarchy;

#[derive(Debug)]
pub struct Renderer {
    window_receiver: ReceiverId<WindowEvent>,
    engine_receiver: ReceiverId<EngineEvent>,
    renderer_enabled: bool,
    camera_buffer: BufferId,
    camera_bind_group: BindGroupId,
    light_buffer: BufferId,
    light_bind_group: BindGroupId,
    pipeline_ldb: PipelineId,
    pipeline_wc: PipelineId,
    pipeline_wcm: PipelineId,
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
        let gfx = res.read::<Graphics>();
        let _hier = res.read::<Hierarchy<Index>>();
        let _transforms = res.read_components::<Transform>();

        // 1. Allow only a single camera
        // 2. Obtain the camera projection matrix and write it to the corresponding uniform buffer
        // 3. Obtain the camera model matrix, calculate the inverse resulting in the view matrix, then
        //    use in the next step
        // 4. For each instance, multiply the view and the model matrix and write to the instance
        //    buffer
        // 5. Doe the same as step 4 for each light

        // Calculate all camera transforms and the respective buffer offset
        let (camera_uniform, camera_view) = res
            .iter_rr::<Camera, Transform>()
            .map(|(_idx, cam, trf)| {
                use num_traits::Inv;

                let camera_transform = trf.affine; //hier_transform(idx, &hier, &transforms);
                let camera_view = camera_transform.inv();

                (
                    CameraUniform {
                        projection: cam.as_persp_matrix().0,
                    },
                    camera_view,
                )
            })
            .next()
            .expect("exactly one camera must be present");

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
                .map(|(_idx, ren, trf)| {
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

                    let instance_transform = trf.affine; //hier_transform(idx, &hier, &transforms);
                    let model_view = camera_view * instance_transform;

                    Instance {
                        model_view: model_view.0,
                        normal: model_view.inv().t().0,
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

        let (light_draw_data, light_buffer_data) = res
            .iter_r::<Light>()
            .map(|(_, lght)| {
                let ldd = LightDrawData {
                    vertex_buffer: lght.model.mesh.vertex_buffer,
                    index_buffer: lght.model.mesh.index_buffer,
                    num_indices: lght.model.mesh.num_indices,
                };

                let light_transform = AffineBuilder::default()
                    .with_scale(0.25)
                    .with_translation(lght.position)
                    .build();
                let model_view = camera_view * light_transform;

                let lu = LightUniform {
                    model_view: model_view.0,
                    color: lght.color.into(),
                };

                (ldd, lu)
            })
            .next()
            .unwrap_or_else(|| todo!("currently, only one light is supported"));

        // Write the camera uniform data to the corresponding uniform buffer
        gfx.write_buffer(self.camera_buffer, &[camera_uniform]);

        // Update the instance buffers
        for (instance_buffer, instance_data) in instance_buffer_data {
            gfx.write_buffer(instance_buffer, unsafe {
                from_raw_parts(
                    instance_data.as_ptr() as *const u8,
                    instance_data.len() * size_of::<Instance>(),
                )
            });
        }

        // Write the light buffer data to the corresponding uniform buffer
        gfx.write_buffer(self.light_buffer, &[light_buffer_data]);

        DrawData {
            lights: vec![light_draw_data],
            instances: instance_draw_data,
        }
    }

    #[tracing::instrument(skip_all)]
    fn draw(&mut self, draw_data: &DrawData, mut rp: RenderPass) -> usize {
        let mut draw_calls = 0;

        for light in &draw_data.lights {
            draw_calls += 1;
            rp.set_pipeline(self.pipeline_ldb)
                .set_bind_group(0, self.camera_bind_group, &[])
                .set_bind_group(1, self.light_bind_group, &[])
                .set_vertex_buffer(0, light.vertex_buffer)
                .set_index_buffer(light.index_buffer)
                .draw_indexed(0..light.num_indices, 0, 0..1);
        }

        for instance_data in &draw_data.instances {
            draw_calls += 1;
            if instance_data.materials.is_empty() {
                rp.set_pipeline(self.pipeline_wc)
                    .set_bind_group(0, self.camera_bind_group, &[])
                    .set_bind_group(1, self.light_bind_group, &[])
                    .set_vertex_buffer(0, instance_data.vertex_buffer)
                    .set_vertex_buffer(1, instance_data.instance_buffer)
                    .set_index_buffer(instance_data.index_buffer)
                    .draw_indexed(0..instance_data.num_indices, 0, instance_data.instance_ids.clone());
            } else {
                rp.set_pipeline(self.pipeline_wcm)
                    .set_bind_group(0, self.camera_bind_group, &[])
                    .set_bind_group(1, self.light_bind_group, &[])
                    .set_bind_group(2, instance_data.materials[0].bind_group, &[])
                    .set_vertex_buffer(0, instance_data.vertex_buffer)
                    .set_vertex_buffer(1, instance_data.instance_buffer)
                    .set_index_buffer(instance_data.index_buffer)
                    .draw_indexed(0..instance_data.num_indices, 0, instance_data.instance_ids.clone());
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
    fn crp_light_debug(adb: &AssetDatabase, gfx: &mut Graphics) -> Result<PipelineId, anyhow::Error> {
        let shader_path = adb.find_asset("shaders", "light_debug.wgsl")?;
        let shader_data = std::fs::read_to_string(&shader_path)
            .with_context(|| format!("Loading a shader source from '{}'", shader_path.display()))?;
        let shader_module = gfx.create_shader_module(Some("light-debug:shader"), &shader_data);

        let cbl = gfx.camera_buffer_layout();
        let lbl = gfx.light_buffer_layout();

        let pipeline = gfx
            .create_render_pipeline()
            .with_label("light-debug:pipeline")
            .add_bind_group_layout(cbl)
            .add_bind_group_layout(lbl)
            .with_vertex_shader_module(shader_module, "vertex_main")
            .with_fragment_shader_module(shader_module, "fragment_main")
            .add_vertex_buffer_layout::<Vertex>()
            .submit();

        Ok(pipeline)
    }

    #[tracing::instrument(skip_all)]
    fn crp_with_camera(adb: &AssetDatabase, gfx: &mut Graphics) -> Result<PipelineId, anyhow::Error> {
        let shader_path = adb.find_asset("shaders", "with_camera.wgsl")?;
        let shader_data = std::fs::read_to_string(&shader_path)
            .with_context(|| format!("Loading a shader source from '{}'", shader_path.display()))?;
        let shader_module = gfx.create_shader_module(Some("with-camera:shader"), &shader_data);

        let cbl = gfx.camera_buffer_layout();
        let lbl = gfx.light_buffer_layout();

        let pipeline = gfx
            .create_render_pipeline()
            .with_label("with-camera:pipeline")
            .add_bind_group_layout(cbl)
            .add_bind_group_layout(lbl)
            .with_vertex_shader_module(shader_module, "vertex_main")
            .with_fragment_shader_module(shader_module, "fragment_main")
            .add_vertex_buffer_layout::<Vertex>()
            .add_vertex_buffer_layout::<Instance>()
            .submit();

        Ok(pipeline)
    }

    #[tracing::instrument(skip_all)]
    fn crp_with_camera_and_material(adb: &AssetDatabase, gfx: &mut Graphics) -> Result<PipelineId, anyhow::Error> {
        let shader_path = adb.find_asset("shaders", "with_camera_and_material.wgsl")?;
        let shader_data = std::fs::read_to_string(&shader_path)
            .with_context(|| format!("Loading a shader source from '{}'", shader_path.display()))?;
        let shader_module = gfx.create_shader_module(Some("with-camera-material:shader"), &shader_data);

        let cbl = gfx.camera_buffer_layout();
        let lbl = gfx.light_buffer_layout();
        let mbl = gfx.material_buffer_layout();

        let pipeline = gfx
            .create_render_pipeline()
            .with_label("with-camera-material:pipeline")
            .add_bind_group_layout(cbl)
            .add_bind_group_layout(lbl)
            .add_bind_group_layout(mbl)
            .with_vertex_shader_module(shader_module, "vertex_main")
            .with_fragment_shader_module(shader_module, "fragment_main")
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

        let pipeline_ldb =
            Self::crp_light_debug(&adb, &mut gfx).context("Creating the light debugging render pipeline")?;
        let pipeline_wcm = Self::crp_with_camera_and_material(&adb, &mut gfx)
            .context("Creating the render pipeline 'with-camera-material'")?;
        let pipeline_wc =
            Self::crp_with_camera(&adb, &mut gfx).context("Creating the render pipeline 'with-camera'")?;

        let uniform_alignment = gfx.limits().min_uniform_buffer_offset_alignment; // 256

        let camera_buffer = gfx.create_buffer(
            Some("camera-buffer"),
            uniform_alignment as BufferAddress,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let cbl = gfx.camera_buffer_layout();
        let camera_bind_group = gfx
            .create_bind_group(cbl)
            .with_label(Some("camera-bind-group"))
            .add_buffer(0, 0, BufferSize::new(size_of::<CameraUniform>() as _), camera_buffer)
            .submit();

        let light_buffer = gfx.create_buffer(
            Some("light-buffer"),
            uniform_alignment as BufferAddress,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );

        let ll = gfx.light_buffer_layout();
        let light_bind_group = gfx
            .create_bind_group(ll)
            .with_label(Some("light-bind-group"))
            .add_buffer(0, 0, BufferSize::new(size_of::<LightUniform>() as _), light_buffer)
            .submit();

        Ok(Renderer {
            window_receiver,
            engine_receiver,
            renderer_enabled: true,
            camera_buffer,
            camera_bind_group,
            light_buffer,
            light_bind_group,
            pipeline_ldb,
            pipeline_wcm,
            pipeline_wc,
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
            res.write::<Statistics>().update_render_stats(
                0,
                frame_start.elapsed(),
                Duration::ZERO,
                Duration::ZERO,
                Duration::ZERO,
            );
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
            }
            Err(SurfaceError::OutOfMemory) => {
                self.on_out_of_memory(res);
                (0, Duration::ZERO, Duration::ZERO)
            }
            Err(SurfaceError::Timeout) => {
                self.on_timeout();
                (0, Duration::ZERO, Duration::ZERO)
            }
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
    lights: Vec<LightDrawData>,
    instances: Vec<InstanceDrawData<'a>>,
}

#[derive(Debug)]
struct LightDrawData {
    vertex_buffer: BufferId,
    index_buffer: BufferId,
    num_indices: u32,
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

#[tracing::instrument(skip_all)]
fn hier_transform(idx: Index, hier: &Hierarchy<Index>, transforms: &<Transform as Component>::Storage) -> Mat4<f32> {
    hier.ancestors(idx)
        .filter_map(|a| transforms.get(a).map(|at| Into::<Mat4<f32>>::into(at.affine)))
        .product::<Mat4<f32>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::asset_database::AssetDatabaseDeps;
    use ecs::Reg;

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
