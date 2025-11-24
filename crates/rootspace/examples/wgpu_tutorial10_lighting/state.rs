use crate::camera::{Camera, CameraController, CameraUniform};
use crate::instance::{Instance, InstanceRaw};
use crate::light::LightUniform;
use crate::model;
use crate::model::{Model, ModelVertex};
use cgmath::{InnerSpace, Rotation3};
use ecs::with_dependencies::WithDependencies;
use griffon::base::ids::{BindGroupId, BufferId, PipelineId};
use griffon::wgpu::util::DeviceExt;
use griffon::wgpu::{BindingType, BufferUsages, ShaderStages, SurfaceError};
use griffon::winit::dpi::PhysicalSize;
use griffon::winit::event::{Event, KeyEvent, WindowEvent};
use griffon::winit::event_loop::EventLoopWindowTarget;
use griffon::winit::keyboard::{KeyCode, PhysicalKey};
use griffon::{Graphics, GraphicsDeps, Settings, wgpu};
use num_traits::Zero;
use tracing::{debug, error};

#[derive(Debug)]
pub struct State {
    graphics: Graphics,
    render_pipeline: PipelineId,
    obj_model: Model,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: BufferId,
    camera_bind_group: BindGroupId,
    instances: Vec<Instance>,
    #[allow(dead_code)]
    instance_buffer: BufferId,
    light_uniform: LightUniform,
    light_buffer: BufferId,
    light_bind_group: BindGroupId,
    light_render_pipeline: PipelineId,
}

impl State {
    pub async fn new(elwt: &EventLoopWindowTarget<()>) -> anyhow::Result<Self> {
        let deps = Dependencies(elwt, &Settings::default());
        let mut gfx = Graphics::with_deps(&deps).await?;

        let size = gfx.window_inner_size();

        let texture_bind_group_layout = gfx
            .create_bind_group_layout()
            .with_label("texture_bind_group_layout")
            .add_bind_group_layout_entry(
                0,
                ShaderStages::FRAGMENT,
                BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
            )
            .add_bind_group_layout_entry(
                1,
                ShaderStages::FRAGMENT,
                BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            )
            .submit();

        let camera = Camera {
            eye: (0.0, 5.0, -10.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: size.width as f32 / size.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_controller = CameraController::new(0.2);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = gfx.create_buffer_init(
            Some("Camera Buffer"),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            &[camera_uniform],
        );

        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                    let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                    let position = cgmath::Vector3 { x, y: 0.0, z };

                    let rotation = if position.is_zero() {
                        cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance { position, rotation }
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer =
            gfx.create_buffer_init(Some("Instance Buffer"), wgpu::BufferUsages::VERTEX, &instance_data);

        let camera_bind_group_layout = gfx
            .create_bind_group_layout()
            .with_label("camera_bind_group_layout")
            .add_bind_group_layout_entry(
                0,
                ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            )
            .submit();

        let camera_bind_group = gfx
            .create_bind_group(camera_bind_group_layout)
            .with_label(Some("camera_bind_group"))
            .add_entire_buffer(0, camera_buffer)
            .submit();

        let obj_model = model::load_model("cube.obj", &mut gfx, texture_bind_group_layout)
            .await
            .unwrap();

        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        let light_buffer = gfx.create_buffer_init(
            Some("Light VB"),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            &[light_uniform],
        );

        let light_bind_group_layout = gfx
            .create_bind_group_layout()
            .add_bind_group_layout_entry(
                0,
                ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            )
            .submit();

        let light_bind_group = gfx
            .create_bind_group(light_bind_group_layout)
            .add_entire_buffer(0, light_buffer)
            .submit();

        let shader = gfx.create_shader_module(Some("Normal Shader"), include_str!("shader.wgsl"));

        let render_pipeline = gfx
            .create_render_pipeline()
            .with_label("Render Pipeline Layout")
            .add_bind_group_layout(texture_bind_group_layout)
            .add_bind_group_layout(camera_bind_group_layout)
            .add_bind_group_layout(light_bind_group_layout)
            .with_vertex_shader_module(shader, "vs_main")
            .with_fragment_shader_module(shader, "fs_main")
            .add_vertex_buffer_layout::<ModelVertex>()
            .add_vertex_buffer_layout::<InstanceRaw>()
            .submit();

        let light_shader = gfx.create_shader_module(Some("Light Shader"), include_str!("light.wgsl"));

        let light_render_pipeline = gfx
            .create_render_pipeline()
            .with_label("Light Pipeline Layout")
            .add_bind_group_layout(camera_bind_group_layout)
            .add_bind_group_layout(light_bind_group_layout)
            .with_vertex_shader_module(light_shader, "vs_main")
            .with_fragment_shader_module(light_shader, "fs_main")
            .add_vertex_buffer_layout::<ModelVertex>()
            .submit();

        Ok(Self {
            graphics: gfx,
            render_pipeline,
            obj_model,
            camera,
            camera_controller,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            instances,
            instance_buffer,
            light_uniform,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
        })
    }

    pub fn run(mut self) -> impl FnMut(Event<()>, &EventLoopWindowTarget<()>) {
        move |event, elwt| {
            self.event_callback(event, elwt);
        }
    }

    fn event_callback(&mut self, event: Event<()>, elwt: &EventLoopWindowTarget<()>) {
        match &event {
            Event::WindowEvent {
                event: window_event, ..
            } => match window_event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(size) => self.resize(*size),
                WindowEvent::RedrawRequested => {
                    self.update();
                    match self.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                            let size = self.graphics.window_inner_size();
                            self.resize(size);
                        }
                        Err(e) => {
                            error!("Unable to render {}", e);
                        }
                    }
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(code),
                            state: key_state,
                            ..
                        },
                    ..
                } => self.handle_key(elwt, *code, key_state.is_pressed()),
                _ => (),
            },
            e => debug!("{e:?}"),
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.graphics.resize(size);
    }

    fn handle_key(&mut self, elwt: &EventLoopWindowTarget<()>, code: KeyCode, is_pressed: bool) {
        if code == KeyCode::Escape && is_pressed {
            elwt.exit();
        } else {
            self.camera_controller.handle_key(code, is_pressed);
        }
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.graphics.write_buffer(self.camera_buffer, &[self.camera_uniform]);

        // Update the light
        let old_position: cgmath::Vector3<_> = self.light_uniform.position.into();
        self.light_uniform.position =
            (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0)) * old_position).into();
        self.graphics.write_buffer(self.light_buffer, &[self.light_uniform]);
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        self.graphics.request_redraw();

        let mut encoder = self.graphics.create_encoder(Some("Render Encoder"))?;
        {
            let mut render_pass = encoder.begin(Some("Render Pass"));

            render_pass.set_vertex_buffer(1, self.instance_buffer);
            render_pass.set_pipeline(self.light_render_pipeline);
            for mesh in &self.obj_model.meshes {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer);
                render_pass.set_index_buffer(mesh.index_buffer);
                render_pass.set_bind_group(0, self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, self.light_bind_group, &[]);
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..1);
            }
            render_pass.set_pipeline(self.render_pipeline);
            for mesh in &self.obj_model.meshes {
                let material = &self.obj_model.materials[mesh.material];
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer);
                render_pass.set_index_buffer(mesh.index_buffer);
                render_pass.set_bind_group(0, material.bind_group, &[]);
                render_pass.set_bind_group(1, self.camera_bind_group, &[]);
                render_pass.set_bind_group(2, self.light_bind_group, &[]);
                render_pass.draw_indexed(0..mesh.num_elements, 0, 0..self.instances.len() as u32);
            }
        }
        encoder.submit();

        Ok(())
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);
pub const NUM_INSTANCES_PER_ROW: u32 = 10;

#[derive(Debug)]
pub struct Dependencies<'a>(&'a EventLoopWindowTarget<()>, &'a Settings);

impl GraphicsDeps for Dependencies<'_> {
    type CustomEvent = ();

    fn event_loop(&self) -> &EventLoopWindowTarget<()> {
        self.0
    }

    fn settings(&self) -> &Settings {
        self.1
    }
}
