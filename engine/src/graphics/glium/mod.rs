use super::{
    BackendTrait, EventTrait, FrameTrait, IndexBufferTrait, ShaderTrait, TextureTrait, Vertex, VertexBufferTrait,
};
use crate::{
    assets::Image,
    components::Renderable,
    event::{EngineEvent, KeyModifiers, KeyState, VirtualKeyCode},
    geometry::rect::Rect,
    resources::GraphicsBackend,
};
use anyhow::Result;
use glium::{
    backend::glutin::DisplayCreationError,
    draw_parameters::DepthTest,
    glutin::{
        Api, ContextBuilder, ElementState, Event as GlutinEvent, EventsLoop, GlProfile, GlRequest, KeyboardInput,
        ModifiersState, VirtualKeyCode as GliumVkc, WindowBuilder, WindowEvent,
    },
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

#[derive(Debug)]
pub struct GliumEvent(pub GlutinEvent);

impl From<GlutinEvent> for GliumEvent {
    fn from(value: GlutinEvent) -> Self {
        GliumEvent(value)
    }
}

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
                            virtual_keycode: Some(GliumVkc::Q),
                            modifiers: ModifiersState { logo: true, .. },
                            ..
                        },
                    ..
                } => Ok(EngineEvent::Shutdown),
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            scancode: sc,
                            state: st,
                            virtual_keycode: vkc,
                            modifiers: mods,
                        },
                    ..
                } => Ok(EngineEvent::KeyboardInput {
                    scan_code: sc,
                    state: st.into(),
                    virtual_keycode: vkc.map(|v| v.into()),
                    modifiers: mods.into(),
                }),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}

impl From<ElementState> for KeyState {
    fn from(value: ElementState) -> Self {
        match value {
            ElementState::Pressed => KeyState::Pressed,
            ElementState::Released => KeyState::Released,
        }
    }
}

impl From<ModifiersState> for KeyModifiers {
    fn from(value: ModifiersState) -> Self {
        KeyModifiers {
            shift: value.shift,
            ctrl: value.ctrl,
            alt: value.alt,
            logo: value.logo,
        }
    }
}

impl From<GliumVkc> for VirtualKeyCode {
    fn from(value: GliumVkc) -> Self {
        use crate::event::VirtualKeyCode::*;
        match value {
            GliumVkc::Key1 => Key1,
            GliumVkc::Key2 => Key2,
            GliumVkc::Key3 => Key3,
            GliumVkc::Key4 => Key4,
            GliumVkc::Key5 => Key5,
            GliumVkc::Key6 => Key6,
            GliumVkc::Key7 => Key7,
            GliumVkc::Key8 => Key8,
            GliumVkc::Key9 => Key9,
            GliumVkc::Key0 => Key0,
            GliumVkc::A => A,
            GliumVkc::B => B,
            GliumVkc::C => C,
            GliumVkc::D => D,
            GliumVkc::E => E,
            GliumVkc::F => F,
            GliumVkc::G => G,
            GliumVkc::H => H,
            GliumVkc::I => I,
            GliumVkc::J => J,
            GliumVkc::K => K,
            GliumVkc::L => L,
            GliumVkc::M => M,
            GliumVkc::N => N,
            GliumVkc::O => O,
            GliumVkc::P => P,
            GliumVkc::Q => Q,
            GliumVkc::R => R,
            GliumVkc::S => S,
            GliumVkc::T => T,
            GliumVkc::U => U,
            GliumVkc::V => V,
            GliumVkc::W => W,
            GliumVkc::X => X,
            GliumVkc::Y => Y,
            GliumVkc::Z => Z,
            GliumVkc::Escape => Escape,
            GliumVkc::F1 => F1,
            GliumVkc::F2 => F2,
            GliumVkc::F3 => F3,
            GliumVkc::F4 => F4,
            GliumVkc::F5 => F5,
            GliumVkc::F6 => F6,
            GliumVkc::F7 => F7,
            GliumVkc::F8 => F8,
            GliumVkc::F9 => F9,
            GliumVkc::F10 => F10,
            GliumVkc::F11 => F11,
            GliumVkc::F12 => F12,
            GliumVkc::F13 => F13,
            GliumVkc::F14 => F14,
            GliumVkc::F15 => F15,
            GliumVkc::F16 => F16,
            GliumVkc::F17 => F17,
            GliumVkc::F18 => F18,
            GliumVkc::F19 => F19,
            GliumVkc::F20 => F20,
            GliumVkc::F21 => F21,
            GliumVkc::F22 => F22,
            GliumVkc::F23 => F23,
            GliumVkc::F24 => F24,
            GliumVkc::Snapshot => PrintScreen,
            GliumVkc::Scroll => ScrollLock,
            GliumVkc::Pause => Pause,
            GliumVkc::Insert => Insert,
            GliumVkc::Home => Home,
            GliumVkc::Delete => Delete,
            GliumVkc::End => End,
            GliumVkc::PageDown => PageDown,
            GliumVkc::PageUp => PageUp,
            GliumVkc::Left => Left,
            GliumVkc::Up => Up,
            GliumVkc::Right => Right,
            GliumVkc::Down => Down,
            GliumVkc::Back => Backspace,
            GliumVkc::Return => Return,
            GliumVkc::Space => Space,
            GliumVkc::Compose => Compose,
            GliumVkc::Caret => Caret,
            GliumVkc::Numlock => NumLock,
            GliumVkc::Numpad0 => Numpad0,
            GliumVkc::Numpad1 => Numpad1,
            GliumVkc::Numpad2 => Numpad2,
            GliumVkc::Numpad3 => Numpad3,
            GliumVkc::Numpad4 => Numpad4,
            GliumVkc::Numpad5 => Numpad5,
            GliumVkc::Numpad6 => Numpad6,
            GliumVkc::Numpad7 => Numpad7,
            GliumVkc::Numpad8 => Numpad8,
            GliumVkc::Numpad9 => Numpad9,
            GliumVkc::AbntC1 => AbntC1,
            GliumVkc::AbntC2 => AbntC2,
            GliumVkc::Add => Add,
            GliumVkc::Apostrophe => Apostrophe,
            GliumVkc::Apps => Apps,
            GliumVkc::At => At,
            GliumVkc::Ax => Ax,
            GliumVkc::Backslash => Backslash,
            GliumVkc::Calculator => Calculator,
            GliumVkc::Capital => Capital,
            GliumVkc::Colon => Colon,
            GliumVkc::Comma => Comma,
            GliumVkc::Convert => Convert,
            GliumVkc::Decimal => Decimal,
            GliumVkc::Divide => Divide,
            GliumVkc::Equals => Equals,
            GliumVkc::Grave => Grave,
            GliumVkc::Kana => Kana,
            GliumVkc::Kanji => Kanji,
            GliumVkc::LAlt => LeftAlt,
            GliumVkc::LBracket => LeftBracket,
            GliumVkc::LControl => LeftControl,
            GliumVkc::LShift => LeftShift,
            GliumVkc::LWin => LeftLogo,
            GliumVkc::Mail => Mail,
            GliumVkc::MediaSelect => MediaSelect,
            GliumVkc::MediaStop => MediaStop,
            GliumVkc::Minus => Minus,
            GliumVkc::Multiply => Multiply,
            GliumVkc::Mute => Mute,
            GliumVkc::MyComputer => MyComputer,
            GliumVkc::NavigateForward => NavigateForward,
            GliumVkc::NavigateBackward => NavigateBackward,
            GliumVkc::NextTrack => NextTrack,
            GliumVkc::NoConvert => NoConvert,
            GliumVkc::NumpadComma => NumpadComma,
            GliumVkc::NumpadEnter => NumpadEnter,
            GliumVkc::NumpadEquals => NumpadEquals,
            GliumVkc::OEM102 => OEM102,
            GliumVkc::Period => Period,
            GliumVkc::PlayPause => PlayPause,
            GliumVkc::Power => Power,
            GliumVkc::PrevTrack => PrevTrack,
            GliumVkc::RAlt => RightAlt,
            GliumVkc::RBracket => RightBracket,
            GliumVkc::RControl => RightControl,
            GliumVkc::RShift => RightShift,
            GliumVkc::RWin => RightLogo,
            GliumVkc::Semicolon => Semicolon,
            GliumVkc::Slash => Slash,
            GliumVkc::Sleep => Sleep,
            GliumVkc::Stop => Stop,
            GliumVkc::Subtract => Subtract,
            GliumVkc::Sysrq => SysRq,
            GliumVkc::Tab => Tab,
            GliumVkc::Underline => Underline,
            GliumVkc::Unlabeled => Unlabeled,
            GliumVkc::VolumeDown => VolumeDown,
            GliumVkc::VolumeUp => VolumeUp,
            GliumVkc::Wake => Wake,
            GliumVkc::WebBack => WebBack,
            GliumVkc::WebFavorites => WebFavorites,
            GliumVkc::WebForward => WebForward,
            GliumVkc::WebHome => WebHome,
            GliumVkc::WebRefresh => WebRefresh,
            GliumVkc::WebSearch => WebSearch,
            GliumVkc::WebStop => WebStop,
            GliumVkc::Yen => Yen,
            GliumVkc::Copy => Copy,
            GliumVkc::Paste => Paste,
            GliumVkc::Cut => Cut,
        }
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

impl TextureTrait<GliumBackend> for GliumTexture {
    fn empty(backend: &GliumBackend, dimensions: (u32, u32)) -> Result<Self> {
        let tex = Texture2d::empty(&backend.display, dimensions.0, dimensions.1)?;

        Ok(GliumTexture(Rc::new(tex)))
    }

    fn from_image(backend: &GliumBackend, image: Image) -> Result<Self> {
        let raw: RawImage2d<u8> = image.into();
        let tex = Texture2d::new(&backend.display, raw)?;

        Ok(GliumTexture(Rc::new(tex)))
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.0.width(), self.0.height())
    }

    fn write<R: Into<Rect<u32>>>(&self, rect: R, data: Cow<[u8]>) {
        let rect = rect.into();
        let dims = rect.dimensions();
        let img = RawImage2d {
            data,
            width: dims[0],
            height: dims[1],
            format: ClientFormat::U8,
        };

        self.0.main_level().write(rect.into(), img)
    }
}

#[derive(Debug, Clone)]
pub struct GliumShader(Rc<Program>);

impl ShaderTrait<GliumBackend> for GliumShader {
    fn from_source<S: AsRef<str>>(backend: &GliumBackend, vs: S, fs: S) -> Result<Self> {
        let progr = Program::from_source(&backend.display, vs.as_ref(), fs.as_ref(), None)?;

        Ok(GliumShader(Rc::new(progr)))
    }
}

#[derive(Debug, Clone)]
pub struct GliumVertexBuffer(Rc<VertexBuffer<Vertex>>);

impl VertexBufferTrait<GliumBackend> for GliumVertexBuffer {
    fn from_vertices(backend: &GliumBackend, vertices: &[Vertex]) -> Result<Self> {
        let vbuf = VertexBuffer::new(&backend.display, vertices)?;

        Ok(GliumVertexBuffer(Rc::new(vbuf)))
    }
}

#[derive(Debug, Clone)]
pub struct GliumIndexBuffer(Rc<IndexBuffer<u16>>);

impl IndexBufferTrait<GliumBackend> for GliumIndexBuffer {
    fn from_indices(backend: &GliumBackend, indices: &[u16]) -> Result<Self> {
        let ibuf = IndexBuffer::new(&backend.display, PrimitiveType::TrianglesList, indices)?;

        Ok(GliumIndexBuffer(Rc::new(ibuf)))
    }
}

pub struct GliumFrame(Frame);

impl FrameTrait<GliumBackend> for GliumFrame {
    fn initialize(&mut self, c: [f32; 4], d: f32) {
        self.0.clear_color_and_depth((c[0], c[1], c[2], c[3]), d)
    }

    fn render<T>(&mut self, transform: &T, factory: &GraphicsBackend<GliumBackend>, data: &Renderable) -> Result<()>
    where
        T: AsRef<[[f32; 4]; 4]>,
    {
        let physical_dimensions = self.0.get_dimensions();

        let u = GliumUniforms {
            transform,
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

    fn finalize(self) -> Result<()> {
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
    pub events_loop: Rc<EventsLoop>,
}

impl BackendTrait for GliumBackend {
    type Event = GliumEvent;
    type Frame = GliumFrame;
    type IndexBuffer = GliumIndexBuffer;
    type Shader = GliumShader;
    type Texture = GliumTexture;
    type VertexBuffer = GliumVertexBuffer;

    fn new<S: AsRef<str>>(title: S, dimensions: (u32, u32), vsync: bool, msaa: u16) -> Result<Self> {
        let window = WindowBuilder::new()
            .with_title(title.as_ref())
            .with_dimensions(dimensions.into())
            .with_resizable(false);

        let context = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core)
            .with_vsync(vsync)
            .with_multisampling(msaa);

        let events_loop = EventsLoop::new();

        match Display::new(window, context, &events_loop) {
            Ok(display) => Ok(GliumBackend {
                display,
                events_loop: Rc::new(events_loop),
            }),
            Err(DisplayCreationError::GlutinCreationError(e)) => Err(e.into()),
            Err(DisplayCreationError::IncompatibleOpenGl(e)) => Err(e.into()),
        }
    }

    fn poll_events<F: FnMut(GliumEvent)>(&mut self, mut f: F) {
        if let Some(el) = Rc::get_mut(&mut self.events_loop) {
            el.poll_events(|e| f(e.into()))
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
    use approx::assert_ulps_ne;
    use std::f64;
    use try_default::TryDefault;

    #[derive(Debug, Clone, Default)]
    struct MockLocation([[f32; 4]; 4]);

    impl AsRef<[[f32; 4]; 4]> for MockLocation {
        fn as_ref(&self) -> &[[f32; 4]; 4] {
            &self.0
        }
    }

    #[test]
    #[cfg_attr(target_os = "macos", ignore = "Crashes on macos")]
    fn backend() {
        let r = GliumBackend::new("Title", (800, 600), false, 0);
        assert!(r.is_ok(), "{}", r.unwrap_err());
    }

    #[test]
    #[cfg_attr(target_os = "macos", ignore = "Crashes on macos")]
    fn dpi_factor() {
        let b = GliumBackend::new("Title", (800, 600), false, 0).unwrap();

        assert_ulps_ne!(b.dpi_factor(), 0.0f64, epsilon = f64::EPSILON);
    }

    #[test]
    #[cfg_attr(target_os = "macos", ignore = "Crashes on macos")]
    fn frame() {
        let mut f = GraphicsBackend::<GliumBackend>::try_default().unwrap();

        let vertices = f
            .create_vertex_buffer(&[
                Vertex::new([0.0, 0.5, 0.0], [0.0, 1.0], [0.0, 0.0, 1.0]),
                Vertex::new([-0.5, -0.5, 0.0], [0.0, 0.0], [0.0, 0.0, 1.0]),
                Vertex::new([0.5, -0.5, 0.0], [1.0, 0.0], [0.0, 0.0, 1.0]),
            ])
            .unwrap();

        let indices = f.create_index_buffer(&[0, 1, 2]).unwrap();

        let shader = f
            .create_source_shader(
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

        let diffuse_texture = f.create_empty_texture((32, 32)).unwrap();
        let normal_texture = Some(f.create_empty_texture((32, 32)).unwrap());

        let data = Renderable::new(vertices, indices, shader, diffuse_texture, normal_texture);

        let mut frame: GliumFrame = f.create_frame();
        frame.initialize([1.0, 0.0, 0.5, 1.0], 1.0);
        assert!(frame.render(&MockLocation::default(), &mut f, &data).is_ok());
        let r = frame.finalize();
        assert!(r.is_ok());
    }
}
