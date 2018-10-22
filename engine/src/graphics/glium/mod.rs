use resources::Vertex;
use super::{BackendTrait, EventsLoopTrait, FrameTrait, TextureTrait};
use event::Event;
use failure::Error;
use glium::{
    draw_parameters::DepthTest,
    glutin::{Api, ContextBuilder, Event as GlutinEvent, EventsLoop, GlProfile, GlRequest, WindowBuilder},
    texture::{Texture2d, ClientFormat, RawImage2d},
    uniforms::{UniformValue, Uniforms},
    Blend, BlendingFunction, Depth, Display, DrawParameters, Frame, IndexBuffer, LinearBlendingFactor, Program,
    Surface, VertexBuffer, Rect
};
use std::{borrow::{Borrow, Cow}, fmt};

#[derive(Debug)]
pub struct GliumEvent(pub GlutinEvent);

impl From<GlutinEvent> for GliumEvent {
    fn from(value: GlutinEvent) -> Self {
        GliumEvent(value)
    }
}

pub struct GliumEventsLoop(Box<EventsLoop>);

impl Default for GliumEventsLoop {
    fn default() -> Self {
        GliumEventsLoop(Box::new(EventsLoop::new()))
    }
}

impl fmt::Debug for GliumEventsLoop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GliumEventsLoop(...)")
    }
}

impl EventsLoopTrait<Event, GliumEvent> for GliumEventsLoop {
    fn poll<F: FnMut(GliumEvent)>(&mut self, mut f: F) {
        self.0.poll_events(|e| f(e.into()))
    }
}

pub struct GliumUniforms<'a, 'b, 'c, T: 'a> {
    transform: &'a T,
    dimensions: [f32; 2],
    diffuse_texture: &'b Texture2d,
    normal_texture: &'c Texture2d,
}

impl<'a, 'b, 'c, T> fmt::Debug for GliumUniforms<'a, 'b, 'c, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GliumUniforms {{ ... }}")
    }
}

impl<'a, 'b, 'c, T> Uniforms for GliumUniforms<'a, 'b, 'c, T>
where
    T: AsRef<[[f32; 4]; 4]> + 'a,
{
    fn visit_values<'f, F: FnMut(&str, UniformValue<'f>)>(&'f self, mut f: F) {
        f("transform", UniformValue::Mat4(*self.transform.as_ref()));
        f("dimensions", UniformValue::Vec2(self.dimensions));
        f("diffuse_texture", UniformValue::Texture2d(self.diffuse_texture, None));
        f("normal_texture", UniformValue::Texture2d(self.normal_texture, None));
    }
}

#[derive(Debug)]
pub struct GliumTexture(Texture2d);

impl TextureTrait for GliumTexture {
    type Backend = GliumBackend;

    fn empty(backend: &GliumBackend, width: u32, height: u32) -> Result<Self, Error> {
        let tex = Texture2d::empty(&backend.display, width, height)?;

        Ok(GliumTexture(tex))
    }

    fn width(&self) -> u32 {
        self.0.width()
    }

    fn height(&self) -> u32 {
        self.0.height()
    }

    fn write<'a>(&self, x: u32, y: u32, width: u32, height: u32, data: Cow<'a, [u8]>) {
        self.0.main_level().write(
            Rect {
                left: x,
                bottom: y,
                width: width,
                height: height,
            },
            RawImage2d {
                data: data,
                width: width,
                height: height,
                format: ClientFormat::U8,
            },
        )
    }
}

#[derive(Debug)]
pub struct GliumRenderData {
    vertices: VertexBuffer<Vertex>,
    indices: IndexBuffer<u16>,
    program: Program,
    diffuse_texture: Texture2d,
    normal_texture: Texture2d,
}

pub struct GliumFrame(Frame);

impl FrameTrait<GliumRenderData> for GliumFrame {
    fn initialize(&mut self, c: [f32; 4], d: f32) {
        self.0.clear_color_and_depth((c[0], c[1], c[2], c[3]), d)
    }

    fn render<T, R>(&mut self, transform: &T, renderable: &R) -> Result<(), Error>
    where
        T: AsRef<[[f32; 4]; 4]>,
        R: Borrow<GliumRenderData>,
    {
        let data = renderable.borrow();

        let dimensions = self.0.get_dimensions();

        let u = GliumUniforms {
            transform: transform,
            dimensions: [dimensions.0 as f32, dimensions.1 as f32],
            diffuse_texture: &data.diffuse_texture,
            normal_texture: &data.normal_texture,
        };

        let dp = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Depth::default()
            },
            blend: Blend {
                color: BlendingFunction::Addition {
                    source: LinearBlendingFactor::One,
                    destination: LinearBlendingFactor::OneMinusSourceAlpha,
                },
                alpha: BlendingFunction::Addition {
                    source: LinearBlendingFactor::One,
                    destination: LinearBlendingFactor::OneMinusSourceAlpha,
                },
                constant_value: (0.0, 0.0, 0.0, 0.0),
            },
            ..DrawParameters::default()
        };

        match self.0.draw(&data.vertices, &data.indices, &data.program, &u, &dp) {
            Ok(()) => Ok(()),
            Err(e) => Err(Into::into(e)),
        }
    }

    fn finalize(self) -> Result<(), Error> {
        self.0.finish()?;

        Ok(())
    }
}

impl fmt::Debug for GliumFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GliumFrame(...)")
    }
}

#[derive(Clone)]
pub struct GliumBackend {
    pub display: Display,
}

impl BackendTrait<GliumEventsLoop, GliumFrame> for GliumBackend {
    fn new(
        events_loop: &GliumEventsLoop,
        title: &str,
        dimensions: [u32; 2],
        vsync: bool,
        msaa: u16,
    ) -> Result<Self, Error> {
        let window = WindowBuilder::new()
            .with_title(title)
            .with_dimensions(From::from((dimensions[0], dimensions[1])));

        let context = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core)
            .with_vsync(vsync)
            .with_multisampling(msaa);

        match Display::new(window, context, &events_loop.0) {
            Ok(display) => Ok(GliumBackend { display }),
            Err(e) => Err(format_err!("{}", e)),
        }
    }

    fn create_frame(&self) -> GliumFrame {
        GliumFrame(self.display.draw())
    }

    fn dpi_factor(&self) -> f64 {
        self.display.gl_window().get_hidpi_factor()
    }
}

impl fmt::Debug for GliumBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GliumBackend {{ display: glium::Display }}")
    }
}

#[cfg(test)]
pub fn triangle(backend: &GliumBackend) -> Result<GliumRenderData, Error> {
    let vertices = VertexBuffer::new(
        &backend.display,
        &[
            Vertex::new([0.0, 0.5, 0.0], [0.0, 1.0], [0.0, 0.0, 1.0]),
            Vertex::new([-0.5, -0.5, 0.0], [0.0, 0.0], [0.0, 0.0, 1.0]),
            Vertex::new([0.5, -0.5, 0.0], [1.0, 0.0], [0.0, 0.0, 1.0]),
        ],
    )?;

    let indices = IndexBuffer::new(&backend.display, ::glium::index::PrimitiveType::TrianglesList, &[0, 1, 2])?;

    let program = Program::from_source(
        &backend.display,
        r#"
                #version 330 core

                uniform mat4 transform;

                layout (location = 0) in vec3 position;
                layout (location = 1) in vec2 tex_coord;
                layout (location = 2) in vec3 normals;

                void main() {
                        gl_Position = transform * vec4(position, 1.0);
                }
                "#,
        r#"
                #version 330 core

                uniform vec2 dimensions;
                uniform Sampler2D diffuse_texture;
                uniform Sampler2D normal_texture;

                out vec4 color;

                void main() {
                        color = vec4(0.3, 0.12, 0.9, 1.0);
                }
                "#,
        None,
    )?;

    let diffuse_texture = Texture2d::empty(&backend.display, 32, 32)?;
    let normal_texture = Texture2d::empty(&backend.display, 32, 32)?;

    Ok(GliumRenderData {
        vertices,
        indices,
        program,
        diffuse_texture,
        normal_texture,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64;

    #[derive(Debug, Clone, Default)]
    struct MockLocation([[f32; 4]; 4]);

    impl AsRef<[[f32; 4]; 4]> for MockLocation {
        fn as_ref(&self) -> &[[f32; 4]; 4] {
            &self.0
        }
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "Failed to initialize any backend!\n    Wayland status: NoCompositorListening\n    X11 status: XOpenDisplayFailed\n"))]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn backend() {
        assert_ok!(GliumBackend::new(
            &GliumEventsLoop::default(),
            "Title",
            [800, 600],
            false,
            0
        ));
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "Failed to initialize any backend!\n    Wayland status: NoCompositorListening\n    X11 status: XOpenDisplayFailed\n"))]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn dpi_factor() {
        let b = GliumBackend::new(&GliumEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();

        assert_ulps_ne!(b.dpi_factor(), 0.0f64, epsilon = f64::EPSILON);
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "Failed to initialize any backend!\n    Wayland status: NoCompositorListening\n    X11 status: XOpenDisplayFailed\n"))]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn frame() {
        let b = GliumBackend::new(&GliumEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();

        let data = triangle(&b).unwrap();

        let mut f: GliumFrame = b.create_frame();
        f.initialize([1.0, 0.0, 0.5, 1.0], 1.0);
        assert_ok!(f.render(&MockLocation::default(), &data));
        let r = f.finalize();
        assert_ok!(r);
    }
}
