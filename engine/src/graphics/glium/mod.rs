use super::{
    private::Sealed, BackendTrait, EventTrait, EventsLoopTrait, FrameTrait, IndexBufferTrait, ShaderTrait,
    TextureTrait, VertexBufferTrait,
};
use crate::{
    assets::{Image, Vertex},
    components::Renderable,
    event::EngineEvent,
    geometry::rect::Rect,
    resources::Backend,
};
use failure::Error;
#[cfg(target_os = "macos")]
use glium::glutin::{KeyboardInput, ModifiersState, VirtualKeyCode};
use glium::{
    backend::glutin::DisplayCreationError,
    draw_parameters::DepthTest,
    glutin::{Api, ContextBuilder, Event as GlutinEvent, EventsLoop, GlProfile, GlRequest, WindowBuilder, WindowEvent},
    index::PrimitiveType,
    texture::{ClientFormat, RawImage2d, Texture2d},
    uniforms::{UniformValue, Uniforms},
    Blend, BlendingFunction, Depth, Display, DrawParameters, Frame, IndexBuffer, LinearBlendingFactor, Program,
    Surface, VertexBuffer,
};
use std::{
    borrow::{Borrow, Cow},
    convert::TryInto,
    fmt,
    rc::Rc,
};
#[cfg(feature = "diagnostics")]
use typename::TypeName;

#[derive(Debug)]
pub struct GliumEvent(pub GlutinEvent);

impl From<GlutinEvent> for GliumEvent {
    fn from(value: GlutinEvent) -> Self {
        GliumEvent(value)
    }
}

impl Sealed for GliumEvent {}

impl EventTrait for GliumEvent {}

impl TryInto<EngineEvent> for GliumEvent {
    type Error = ();

    fn try_into(self) -> Result<EngineEvent, Self::Error> {
        if let GliumEvent(GlutinEvent::WindowEvent { event: we, .. }) = self {
            match we {
                WindowEvent::CloseRequested => Ok(EngineEvent::Shutdown),
                WindowEvent::Resized(l) => Ok(EngineEvent::Resize(l.into())),
                WindowEvent::HiDpiFactorChanged(f) => Ok(EngineEvent::ChangeDpi(f)),
                #[cfg(target_os = "macos")]
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Q),
                            modifiers: ModifiersState { logo: true, .. },
                            ..
                        },
                    ..
                } => Ok(EngineEvent::Shutdown),
                _ => Err(()),
            }
        } else {
            Err(())
        }
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

impl Sealed for GliumEventsLoop {}

impl EventsLoopTrait<GliumBackend> for GliumEventsLoop {
    fn poll<F: FnMut(GliumEvent)>(&mut self, mut f: F) {
        self.0.poll_events(|e| f(e.into()))
    }
}

pub struct GliumUniforms<'a, 'b, 'c, T: 'a> {
    transform: &'a T,
    physical_dimensions: (u32, u32),
    diffuse_texture: &'b Texture2d,
    normal_texture: Option<&'c Texture2d>,
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
        f(
            "physical_dimensions",
            UniformValue::Vec2([self.physical_dimensions.0 as f32, self.physical_dimensions.1 as f32]),
        );
        f("diffuse_texture", UniformValue::Texture2d(self.diffuse_texture, None));
        if let Some(nt) = self.normal_texture {
            f("normal_texture", UniformValue::Texture2d(nt, None));
        }
    }
}

#[derive(Debug, Clone)]
pub struct GliumTexture(Rc<Texture2d>);

impl Sealed for GliumTexture {}

impl TextureTrait<GliumBackend> for GliumTexture {
    fn empty(backend: &GliumBackend, dimensions: (u32, u32)) -> Result<Self, Error> {
        let tex = Texture2d::empty(&backend.display, dimensions.0, dimensions.1)?;

        Ok(GliumTexture(Rc::new(tex)))
    }

    fn from_image(backend: &GliumBackend, image: Image) -> Result<Self, Error> {
        let raw: RawImage2d<u8> = image.into();
        let tex = Texture2d::new(&backend.display, raw)?;

        Ok(GliumTexture(Rc::new(tex)))
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.0.width(), self.0.height())
    }

    fn write<'a, R: Into<Rect<u32>>>(&self, rect: R, data: Cow<'a, [u8]>) {
        let rect = rect.into();
        let dims = rect.dimensions();
        let img = RawImage2d {
            data: data,
            width: dims[0],
            height: dims[1],
            format: ClientFormat::U8,
        };

        self.0.main_level().write(rect.into(), img)
    }
}

#[derive(Debug, Clone)]
pub struct GliumShader(Rc<Program>);

impl Sealed for GliumShader {}

impl ShaderTrait<GliumBackend> for GliumShader {
    fn from_source<S: AsRef<str>>(backend: &GliumBackend, vs: S, fs: S) -> Result<Self, Error> {
        let progr = Program::from_source(&backend.display, vs.as_ref(), fs.as_ref(), None)?;

        Ok(GliumShader(Rc::new(progr)))
    }
}

#[derive(Debug, Clone)]
pub struct GliumVertexBuffer(Rc<VertexBuffer<Vertex>>);

impl Sealed for GliumVertexBuffer {}

impl VertexBufferTrait<GliumBackend> for GliumVertexBuffer {
    fn from_vertices(backend: &GliumBackend, vertices: &[Vertex]) -> Result<Self, Error> {
        let vbuf = VertexBuffer::new(&backend.display, vertices)?;

        Ok(GliumVertexBuffer(Rc::new(vbuf)))
    }
}

#[derive(Debug, Clone)]
pub struct GliumIndexBuffer(Rc<IndexBuffer<u16>>);

impl Sealed for GliumIndexBuffer {}

impl IndexBufferTrait<GliumBackend> for GliumIndexBuffer {
    fn from_indices(backend: &GliumBackend, indices: &[u16]) -> Result<Self, Error> {
        let ibuf = IndexBuffer::new(&backend.display, PrimitiveType::TrianglesList, indices)?;

        Ok(GliumIndexBuffer(Rc::new(ibuf)))
    }
}

pub struct GliumFrame(Frame);

impl Sealed for GliumFrame {}

impl FrameTrait<GliumBackend> for GliumFrame {
    fn initialize(&mut self, c: [f32; 4], d: f32) {
        self.0.clear_color_and_depth((c[0], c[1], c[2], c[3]), d)
    }

    fn render<T>(&mut self, transform: &T, factory: &Backend<GliumBackend>, data: &Renderable) -> Result<(), Error>
    where
        T: AsRef<[[f32; 4]; 4]>,
    {
        let physical_dimensions = self.0.get_dimensions();

        let u = GliumUniforms {
            transform: transform,
            physical_dimensions,
            diffuse_texture: &factory.borrow_texture(data.diffuse_texture()).0,
            normal_texture: data.normal_texture().map(|id| factory.borrow_texture(id).0.borrow()),
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

        let vertices = factory.borrow_vertex_buffer(data.vertices());
        let indices = factory.borrow_index_buffer(data.indices());
        let shader = factory.borrow_shader(data.shader());

        match self.0.draw(
            Borrow::<VertexBuffer<Vertex>>::borrow(&vertices.0),
            Borrow::<IndexBuffer<u16>>::borrow(&indices.0),
            shader.0.borrow(),
            &u,
            &dp,
        ) {
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

#[cfg_attr(feature = "diagnostics", derive(TypeName))]
#[derive(Clone)]
pub struct GliumBackend {
    pub display: Display,
}

impl Sealed for GliumBackend {}

impl BackendTrait for GliumBackend {
    type Event = GliumEvent;
    type EventsLoop = GliumEventsLoop;
    type Frame = GliumFrame;
    type IndexBuffer = GliumIndexBuffer;
    type Shader = GliumShader;
    type Texture = GliumTexture;
    type VertexBuffer = GliumVertexBuffer;

    fn new(
        events_loop: &GliumEventsLoop,
        title: &str,
        dimensions: (u32, u32),
        vsync: bool,
        msaa: u16,
    ) -> Result<Self, Error> {
        let window = WindowBuilder::new()
            .with_title(title)
            .with_dimensions(dimensions.into())
            .with_resizable(false);

        let context = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core)
            .with_vsync(vsync)
            .with_multisampling(msaa);

        match Display::new(window, context, &events_loop.0) {
            Ok(display) => Ok(GliumBackend { display }),
            Err(DisplayCreationError::GlutinCreationError(e)) => Err(e.into()),
            Err(DisplayCreationError::IncompatibleOpenGl(e)) => Err(e.into()),
        }
    }

    fn create_frame(&self) -> GliumFrame {
        GliumFrame(self.display.draw())
    }

    fn dpi_factor(&self) -> f64 {
        self.display.gl_window().window().get_hidpi_factor()
    }

    fn physical_dimensions(&self) -> (u32, u32) {
        self.display.get_framebuffer_dimensions()
    }
}

impl fmt::Debug for GliumBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GliumBackend {{ display: glium::Display }}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64;
    use approx::assert_ulps_ne;

    #[derive(Debug, Clone, Default)]
    struct MockLocation([[f32; 4]; 4]);

    impl AsRef<[[f32; 4]; 4]> for MockLocation {
        fn as_ref(&self) -> &[[f32; 4]; 4] {
            &self.0
        }
    }

    #[test]
    #[cfg_attr(
        feature = "wsl",
        should_panic(
            expected = "Failed to initialize any backend! Wayland status: XdgRuntimeDirNotSet X11 status: XOpenDisplayFailed"
        )
    )]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn backend() {
        let r = GliumBackend::new(&GliumEventsLoop::default(), "Title", (800, 600), false, 0);
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    #[cfg_attr(
        feature = "wsl",
        should_panic(
            expected = "Failed to initialize any backend! Wayland status: XdgRuntimeDirNotSet X11 status: XOpenDisplayFailed"
        )
    )]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn dpi_factor() {
        let b = GliumBackend::new(&GliumEventsLoop::default(), "Title", (800, 600), false, 0).unwrap();

        assert_ulps_ne!(b.dpi_factor(), 0.0f64, epsilon = f64::EPSILON);
    }

    #[test]
    #[cfg_attr(
        feature = "wsl",
        should_panic(
            expected = "Failed to initialize any backend! Wayland status: XdgRuntimeDirNotSet X11 status: XOpenDisplayFailed"
        )
    )]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn frame() {
        let b = GliumBackend::new(&GliumEventsLoop::default(), "Title", (800, 600), false, 0).unwrap();
        let mut f: Backend<GliumBackend> = Backend::default();

        let vertices = f
            .create_vertex_buffer(
                &b,
                &[
                    Vertex::new([0.0, 0.5, 0.0], [0.0, 1.0], [0.0, 0.0, 1.0]),
                    Vertex::new([-0.5, -0.5, 0.0], [0.0, 0.0], [0.0, 0.0, 1.0]),
                    Vertex::new([0.5, -0.5, 0.0], [1.0, 0.0], [0.0, 0.0, 1.0]),
                ],
            )
            .unwrap();

        let indices = f.create_index_buffer(&b, &[0, 1, 2]).unwrap();

        let shader = f
            .create_source_shader(
                &b,
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
                    uniform sampler2D diffuse_texture;
                    // uniform sampler2D normal_texture;

                    out vec4 color;

                    void main() {
                            color = vec4(0.3, 0.12, 0.9, 1.0);
                    }
                    "#,
            )
            .unwrap();

        let diffuse_texture = f.create_empty_texture(&b, (32, 32)).unwrap();
        let normal_texture = Some(f.create_empty_texture(&b, (32, 32)).unwrap());

        let data = Renderable::new(vertices, indices, shader, diffuse_texture, normal_texture);

        let mut frame: GliumFrame = b.create_frame();
        frame.initialize([1.0, 0.0, 0.5, 1.0], 1.0);
        assert!(frame.render(&MockLocation::default(), &mut f, &data).is_ok());
        let r = frame.finalize();
        assert!(r.is_ok());
    }
}
