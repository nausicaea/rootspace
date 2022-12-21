use ecs::{Resource, SerializationName};
use log::debug;
use serde::{Deserialize, Serialize};
use wgpu::{DeviceDescriptor, Instance, RequestAdapterOptions, TextureUsages};
use winit::{
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowId},
};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Graphics {
    settings: Settings,
    #[serde(skip)]
    runtime: Option<Runtime>,
}

impl Graphics {
    pub async fn initialize<T>(&mut self, event_loop: &EventLoopWindowTarget<T>) {
        let window = WindowBuilder::new().build(event_loop).unwrap();

        let instance = Instance::new(self.settings.backends);
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: self.settings.power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        debug!("Supported adapter features: {:?}", adapter.features());

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: self.settings.features,
                    limits: self.settings.limits.clone(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let size = window.inner_size();
        debug!("Physical window size: {:?}", &size);

        let texture_formats = surface.get_supported_formats(&adapter);
        debug!("Supported texture formats: {:?}", &texture_formats);

        let texture_format = *texture_formats
            .iter()
            .filter(|&tf| tf == &self.settings.preferred_texture_format)
            .next()
            .unwrap_or(&texture_formats[0]);
        debug!("Choosing texture format: {:?}", &texture_format);

        let present_modes = surface.get_supported_present_modes(&adapter);
        debug!("Supported present modes: {:?}", &present_modes);

        let config = wgpu::SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: texture_format,
            width: size.width,
            height: size.height,
            present_mode: self.settings.present_mode,
            alpha_mode: self.settings.alpha_mode,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("triangle.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[], // 2.
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            // continued ...
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            // continued ...
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        self.runtime = Some(Runtime {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
        });
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(ref mut rt) = self.runtime {
            if new_size.width > 0 && new_size.height > 0 {
                rt.size = new_size;
                rt.config.width = new_size.width;
                rt.config.height = new_size.height;
                rt.surface.configure(&rt.device, &rt.config);
            }
        }
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        if let Some(ref rt) = self.runtime {
            let output = rt.surface.get_current_texture()?;
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = rt.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&rt.render_pipeline); // 2.
                render_pass.draw(0..3, 0..1); // 3.
            }

            // submit will accept anything that implements IntoIter
            let _si = rt.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        Ok(())
    }

    pub fn window_id(&self) -> Option<WindowId> {
        self.runtime.as_ref().map(|rt| rt.window.id())
    }
}

impl Resource for Graphics {}

impl SerializationName for Graphics {}

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    backends: wgpu::Backends,
    power_preference: wgpu::PowerPreference,
    features: wgpu::Features,
    limits: wgpu::Limits,
    preferred_texture_format: wgpu::TextureFormat,
    present_mode: wgpu::PresentMode,
    alpha_mode: wgpu::CompositeAlphaMode,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            backends: wgpu::Backends::all(),
            power_preference: wgpu::PowerPreference::LowPower,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            preferred_texture_format: wgpu::TextureFormat::Bgra8UnormSrgb,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        }
    }
}

#[derive(Debug)]
struct Runtime {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
}
