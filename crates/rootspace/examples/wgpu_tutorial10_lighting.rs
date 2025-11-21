use griffon::winit::{event_loop::{EventLoop, EventLoopWindowTarget}, event::{Event, WindowEvent, KeyEvent}, keyboard::{PhysicalKey, KeyCode}, dpi::PhysicalSize};
use griffon::wgpu::{SurfaceError, ShaderStages, BindingType, BufferUsages, BufferBindingType};
use griffon::resources::{Graphics, GraphicsDeps};
use griffon::base::camera_uniform::CameraUniform;
use griffon::Settings;
use rootspace::components::camera::Camera;
use tracing::error;
use std::sync::Arc;
use tokio::runtime::Builder as RuntimeBuilder;
use ecs::with_dependencies::WithDependencies;
use griffon::base::light_uniform::LightUniform;
use griffon::components::light::Light;

fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tokio-console")]
    console_subscriber::init();
    #[cfg(not(feature = "tokio-console"))]
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new()?;
    let rt = Arc::new(RuntimeBuilder::new_multi_thread().enable_all().build()?);
    let state = rt.block_on(State::new(&event_loop))?;
    event_loop.run(state.run())?;
    Ok(())
}

#[derive(Debug)]
struct State {
    graphics: Graphics,
    camera: Camera,
    camera_controller: CameraController,
}

impl State {
    async fn new(elwt: &EventLoopWindowTarget<()>) -> anyhow::Result<Self> {
        #[derive(Debug)]
        struct Dependencies<'a>(&'a EventLoopWindowTarget<()>, &'a Settings);

        impl GraphicsDeps for Dependencies<'_> {
            type CustomEvent = ();

            fn event_loop(&self) -> &EventLoopWindowTarget<()> {
                self.0
            }

            fn settings(&self) -> &Settings {
                self.1
            }
        }

        let mut graphics = Graphics::with_deps(&Dependencies(elwt, &Settings::default())).await?;

        let inner_size = graphics.window_inner_size();
        let camera = Camera::new(
            inner_size.width,
            inner_size.height,
            45.0 / 360.0 * std::f32::consts::PI,
            (0.1, 100.0),
        );

        let camera_uniform = CameraUniform {
            projection: camera.as_persp_matrix().0,
        };
        let camera_buffer = graphics.create_buffer_init(
            Some("Camera Buffer"),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            &[camera_uniform],
        );

        let texture_bind_group_layout = graphics.create_bind_group_layout()
            .with_label("texture_bind_group_layout")
            .add_bind_group_layout_entry(
                0,
                ShaderStages::FRAGMENT,
                BindingType::Texture {
                    multisampled: false,
                    view_dimension: griffon::wgpu::TextureViewDimension::D2,
                    sample_type: griffon::wgpu::TextureSampleType::Float { filterable: true },
                }
            )
            .add_bind_group_layout_entry(
                0,
                ShaderStages::FRAGMENT,
                BindingType::Sampler(griffon::wgpu::SamplerBindingType::Filtering)
            )
            .submit();

        let camera_bind_group_layout = graphics.create_bind_group_layout()
            .with_label("camera_bind_group_layout")
            .add_bind_group_layout_entry(
                0,
                ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                }
            ).submit();

        let camera_bind_group = graphics.create_bind_group(camera_bind_group_layout)
            .with_label("camera_bind_group")
            .add_entire_buffer(0, camera_buffer)
            .submit();

        // let light_uniform = LightUniform {
        //     model_view: todo!(),
        //     color: [1.0, 1.0, 1.0, 1.0],
        // };
        //
        // let light_buffer = graphics.create_buffer_init(
        //     Some("Light VB"),
        //     BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        //     &[light_uniform],
        // );
        //
        // let light_bind_group_layout = todo!();
        //
        // let light_bind_group = todo!();

        Ok(Self {
            camera_controller: CameraController::new(0.2),
            camera,
            graphics,
        })
    }

    fn run(mut self) -> impl FnMut(Event<()>, &EventLoopWindowTarget<()>) {
        move |event, elwt| {
            self.event_callback(event, elwt);
        }
    }

    fn event_callback(&mut self, event: Event<()>, elwt: &EventLoopWindowTarget<()>) {
        match &event {
            Event::WindowEvent {
                event: window_event,
                ..
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
                },
                WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                    ..
                } => self.handle_key(elwt, *code, key_state.is_pressed()),
                _ => (),
            },
            _ => todo!(),
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
        todo!()
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        todo!()
    }
}

#[derive(Debug)]
struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    fn handle_key(&mut self, key: KeyCode, is_pressed: bool) -> bool {
        match key {
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft => {
                self.is_down_pressed = is_pressed;
                true
            }
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    fn update_camera(&self, camera: &mut Camera) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the up/ down is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so
            // that it doesn't change. The eye therefore still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
