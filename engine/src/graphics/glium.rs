use super::{BackendTrait, EventsLoopTrait, FrameTrait, RenderDataTrait};
use event::Event;
use failure::Error;
use glium::draw_parameters::DepthTest;
use glium::glutin::{Api, ContextBuilder, EventsLoop, GlProfile, GlRequest, WindowBuilder};
use glium::index::PrimitiveType;
use glium::{
    Blend, BlendingFunction, Depth, Display, DrawParameters, Frame, IndexBuffer,
    LinearBlendingFactor, Program, Surface, VertexBuffer,
};
use std::fmt;

pub use glium::glutin::Event as GliumEvent;

pub struct GliumEventsLoop(Box<EventsLoop>);

impl Default for GliumEventsLoop {
    fn default() -> Self {
        GliumEventsLoop(Box::new(EventsLoop::new()))
    }
}

impl fmt::Debug for GliumEventsLoop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GliumEventsLoop")
    }
}

impl EventsLoopTrait<Event, GliumEvent> for GliumEventsLoop {
    fn poll<F: FnMut(GliumEvent)>(&mut self, f: F) {
        self.0.poll_events(f)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

#[derive(Debug)]
pub struct GliumRenderData {
    vertices: VertexBuffer<Vertex>,
    indices: IndexBuffer<u16>,
    program: Program,
}

impl RenderDataTrait<GliumBackend> for GliumRenderData {
    fn triangle(backend: &GliumBackend) -> Result<Self, Error> {
        let vertices = VertexBuffer::new(
            &backend.0,
            &[
                Vertex {
                    position: [-0.5, -0.5],
                },
                Vertex {
                    position: [0.0, 0.5],
                },
                Vertex {
                    position: [0.5, -0.5],
                },
            ],
        )?;

        let indices = IndexBuffer::new(&backend.0, PrimitiveType::TrianglesList, &[0, 1, 2])?;

        let program = Program::from_source(
            &backend.0,
            r#"
                    #version 330 core

                    uniform mat4 location;

                    in vec2 position;

                    void main() {
                            gl_Position = location * vec4(position, 0.0, 1.0);
                    }
                    "#,
            r#"
                    #version 330 core

                    const vec3 specular_color = vec3(0.3, 0.15, 0.1);

                    out vec4 color;

                    void main() {
                            color = vec4(specular_color, 1.0);
                    }
                    "#,
            None,
        )?;

        Ok(GliumRenderData {
            vertices,
            indices,
            program,
        })
    }
}

pub struct GliumFrame(Frame);

impl FrameTrait<GliumRenderData> for GliumFrame {
    fn initialize(&mut self, color: [f32; 4], depth: f32) {
        self.0
            .clear_color_and_depth((color[0], color[1], color[2], color[3]), depth)
    }

    fn render<L: AsRef<[[f32; 4]; 4]>>(
        &mut self,
        location: &L,
        data: &GliumRenderData,
    ) -> Result<(), Error> {
        let uniforms = uniform! {
            location: *location.as_ref(),
        };

        let draw_params = DrawParameters {
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

        match self.0.draw(
            &data.vertices,
            &data.indices,
            &data.program,
            &uniforms,
            &draw_params,
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
        write!(f, "GliumFrame")
    }
}

#[derive(Clone)]
pub struct GliumBackend(Display);

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
            .with_dimensions(dimensions[0], dimensions[1]);

        let context = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core)
            .with_vsync(vsync)
            .with_multisampling(msaa);

        match Display::new(window, context, &events_loop.0) {
            Ok(d) => Ok(GliumBackend(d)),
            Err(e) => Err(format_err!("{}", e)),
        }
    }

    fn create_frame(&self) -> GliumFrame {
        GliumFrame(self.0.draw())
    }
}

impl fmt::Debug for GliumBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GliumBackend")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Default)]
    struct MockLocation([[f32; 4]; 4]);

    impl AsRef<[[f32; 4]; 4]> for MockLocation {
        fn as_ref(&self) -> &[[f32; 4]; 4] {
            &self.0
        }
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
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
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn render_data() {
        let b =
            GliumBackend::new(&GliumEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();

        assert_ok!(GliumRenderData::triangle(&b));
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
    #[cfg_attr(
        target_os = "macos",
        should_panic(expected = "Windows can only be created on the main thread on macOS")
    )]
    fn frame() {
        let b =
            GliumBackend::new(&GliumEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();

        let data = GliumRenderData::triangle(&b).unwrap();

        let mut f: GliumFrame = b.create_frame();
        f.initialize([1.0, 0.0, 0.5, 1.0], 1.0);
        assert_ok!(f.render(&MockLocation::default(), &data));
        let r = f.finalize();
        assert_ok!(r);
    }
}
