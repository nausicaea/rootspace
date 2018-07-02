#![allow(dead_code)]

pub mod graphics {
    pub mod headless {
        use super::{BackendTrait, FrameTrait, RenderDataTrait};
        use failure::Error;

        #[derive(Debug, Clone, Default)]
        pub struct HeadlessEventsLoop;

        #[derive(Debug, Clone, Default)]
        pub struct HeadlessRenderData;

        impl RenderDataTrait<HeadlessBackend> for HeadlessRenderData {
            fn triangle(_backend: &HeadlessBackend) -> Result<Self, Error> {
                Ok(HeadlessRenderData::default())
            }
        }

        #[derive(Debug, Clone, Default)]
        pub struct HeadlessFrame;

        impl FrameTrait<HeadlessRenderData> for HeadlessFrame {
            fn initialize(&mut self, _color: [f32; 4], _depth: f32) {
            }

            fn render<L: AsRef<[[f32; 4]; 4]>>(&mut self, _location: &L, _data: &HeadlessRenderData) -> Result<(), Error> {
                Ok(())
            }

            fn finalize(self) -> Result<(), Error> {
                Ok(())
            }
        }

        #[derive(Debug, Clone, Default)]
        pub struct HeadlessBackend;

        impl BackendTrait<HeadlessEventsLoop, HeadlessFrame> for HeadlessBackend {
            fn new(_events_loop: &HeadlessEventsLoop, _title: &str, _dimensions: [u32; 2], _vsync: bool, _msaa: u16) -> Result<Self, Error> {
                Ok(HeadlessBackend::default())
            }

            fn create_frame(&self) -> HeadlessFrame {
                HeadlessFrame::default()
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
            fn backend() {
                assert_ok!(HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0));
            }

            #[test]
            fn render_data() {
                let b = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();

                assert_ok!(HeadlessRenderData::triangle(&b));
            }

            #[test]
            fn frame() {
                let b = HeadlessBackend::new(&HeadlessEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();

                let mut f: HeadlessFrame = b.create_frame();
                f.initialize([1.0, 0.0, 0.5, 1.0], 1.0);
                assert_ok!(f.render(&MockLocation::default(), &HeadlessRenderData::default()));
                let r = f.finalize();
                assert_ok!(r);
            }
        }
    }

    pub mod glium {
        use super::{BackendTrait, FrameTrait, RenderDataTrait};
        use failure::Error;
        use glium::{Display, Frame, Surface, VertexBuffer, IndexBuffer, Program};
        use glium::index::PrimitiveType;
        use glium::glutin::{EventsLoop, WindowBuilder, ContextBuilder, GlRequest, Api, GlProfile};
        use std::fmt;

        pub struct GliumEventsLoop(EventsLoop);

        impl Default for GliumEventsLoop {
            fn default() -> Self {
                GliumEventsLoop(EventsLoop::new())
            }
        }

        impl fmt::Debug for GliumEventsLoop {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "GliumEventsLoop")
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
                let vertices = VertexBuffer::new(&backend.0, &[
                    Vertex { position: [-0.5, -0.5] },
                    Vertex { position: [0.0, 0.5] },
                    Vertex { position: [0.5, -0.5] },
                ])?;

                let indices = IndexBuffer::new(&backend.0, PrimitiveType::TrianglesList, &[0, 1, 2])?;

                let program = Program::from_source(&backend.0,
                    r#"
                    #version 330 core

                    in vec2 position;

                    void main() {
                            gl_Position = vec4(position, 0.0, 1.0);
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
                    None)?;

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
                self.0.clear_color_and_depth((color[0], color[1], color[2], color[3]), depth)
            }

            fn render<L: AsRef<[[f32; 4]; 4]>>(&mut self, location: &L, data: &GliumRenderData) -> Result<(), Error> {
                let uniforms = uniform! {
                    location: *location.as_ref(),
                };
                self.0.draw(&data.vertices, &data.indices, &data.program, &uniforms, &Default::default())?;

                Ok(())
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
            fn new(events_loop: &GliumEventsLoop, title: &str, dimensions: [u32; 2], vsync: bool, msaa: u16) -> Result<Self, Error> {
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
            #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
            fn backend() {
                assert_ok!(GliumBackend::new(&GliumEventsLoop::default(), "Title", [800, 600], false, 0));
            }

            #[test]
            #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
            #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
            fn render_data() {
                let b = GliumBackend::new(&GliumEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();

                assert_ok!(GliumRenderData::triangle(&b));
            }

            #[test]
            #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
            #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
            fn frame() {
                let b = GliumBackend::new(&GliumEventsLoop::default(), "Title", [800, 600], false, 0).unwrap();

                let data = GliumRenderData::triangle(&b).unwrap();

                let mut f: GliumFrame = b.create_frame();
                f.initialize([1.0, 0.0, 0.5, 1.0], 1.0);
                assert_ok!(f.render(&MockLocation::default(), &data));
                let r = f.finalize();
                assert_ok!(r);
            }
        }
    }

    use failure::Error;

    pub trait BackendTrait<E, F>
    where
        Self: Sized,
    {
        fn new(events_loop: &E, title: &str, dimensions: [u32; 2], vsync: bool, msaa: u16) -> Result<Self, Error>;
        fn create_frame(&self) -> F;
    }

    pub trait FrameTrait<R> {
        fn initialize(&mut self, color: [f32; 4], depth: f32);
        fn render<L: AsRef<[[f32; 4]; 4]>>(&mut self, location: &L, data: &R) -> Result<(), Error>;
        fn finalize(self) -> Result<(), Error>;
    }

    pub trait RenderDataTrait<B>
    where
        Self: Sized,
    {
        fn triangle(backend: &B) -> Result<Self, Error>;
    }
}

use self::graphics::{BackendTrait, FrameTrait};
use failure::Error;
use std::marker::PhantomData;

#[derive(Debug)]
struct Renderer<L, R, F, E, B> {
    backend: B,
    frames: usize,
    draw_calls: usize,
    _l: PhantomData<L>,
    _r: PhantomData<R>,
    _f: PhantomData<F>,
    _e: PhantomData<E>,
}

impl<L, R, F, E, B> Renderer<L, R, F, E, B>
where
    L: AsRef<[[f32; 4]; 4]>,
    F: FrameTrait<R>,
    B: BackendTrait<E, F>,
{
    pub fn new(events_loop: &E, title: &str, dimensions: [u32; 2], vsync: bool, msaa: u16) -> Result<Self, Error> {
        Ok(Renderer {
            backend: B::new(events_loop, title, dimensions, vsync, msaa)?,
            frames: 0,
            draw_calls: 0,
            _l: PhantomData::default(),
            _r: PhantomData::default(),
            _f: PhantomData::default(),
            _e: PhantomData::default(),
        })
    }

    #[cfg(any(test, feature = "diagnostics"))]
    pub fn average_draw_calls(&self) -> f32 {
        if self.frames > 0 {
            self.draw_calls as f32 / self.frames as f32
        } else {
            0.0
        }
    }

    pub fn render(&mut self, nodes: &[(L, R)]) -> Result<(), Error> {
        #[cfg(any(test, feature = "diagnostics"))]
        {
            self.frames += 1;
        }
        let mut target = self.backend.create_frame();

        target.initialize([0.0, 0.0, 0.0, 1.0], 1.0);

        for (m, d) in nodes {
            #[cfg(any(test, feature = "diagnostics"))]
            {
                self.draw_calls += 1;
            }
            target.render(m, d)?;
        }

        target.finalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::graphics::RenderDataTrait;
    use super::graphics::headless::{HeadlessBackend as HB, HeadlessFrame as HF, HeadlessRenderData as HRD, HeadlessEventsLoop as HEL};
    use super::graphics::glium::{GliumBackend as GB, GliumFrame as GF, GliumRenderData as GRD, GliumEventsLoop as GEL};
    use std::f32;

    #[derive(Debug, Clone, Default)]
    struct MockLocation([[f32; 4]; 4]);

    impl AsRef<[[f32; 4]; 4]> for MockLocation {
        fn as_ref(&self) -> &[[f32; 4]; 4] {
            &self.0
        }
    }

    #[test]
    fn new_headless() {
        assert_ok!(Renderer::<MockLocation, HRD, HF, HEL, HB>::new(&Default::default(), "Title", [800, 600], false, 0));
    }

    #[test]
    fn render_headless() {
        let mut r = Renderer::<MockLocation, HRD, HF, HEL, HB>::new(&Default::default(), "Title", [800, 600], false, 0).unwrap();

        let nodes = vec![
            (MockLocation::default(), HRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), HRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), HRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), HRD::triangle(&r.backend).unwrap()),
        ];

        assert_ok!(r.render(&nodes));
        assert_eq!(r.frames, 1);
        assert_eq!(r.draw_calls, 4);
        assert_ulps_eq!(r.average_draw_calls(), 4.0, epsilon = f32::EPSILON);
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
    #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
    fn new_glium() {
        assert_ok!(Renderer::<MockLocation, GRD, GF, GEL, GB>::new(&Default::default(), "Title", [800, 600], false, 0));
    }

    #[test]
    #[cfg_attr(feature = "wsl", should_panic(expected = "No backend is available"))]
    #[cfg_attr(target_os = "macos", should_panic(expected = "Windows can only be created on the main thread on macOS"))]
    fn render_glium() {
        let mut r = Renderer::<MockLocation, GRD, GF, GEL, GB>::new(&Default::default(), "Title", [800, 600], false, 0).unwrap();

        let nodes = vec![
            (MockLocation::default(), GRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), GRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), GRD::triangle(&r.backend).unwrap()),
            (MockLocation::default(), GRD::triangle(&r.backend).unwrap()),
        ];

        assert_ok!(r.render(&nodes));
        assert_eq!(r.frames, 1);
        assert_eq!(r.draw_calls, 4);
        assert_ulps_eq!(r.average_draw_calls(), 4.0, epsilon = f32::EPSILON);
    }
}
