use std::{
    borrow::{Borrow, Cow},
    convert::TryInto,
    fmt,
    rc::Rc,
};

use anyhow::Result;
use glium::{
    backend::glutin::DisplayCreationError,
    draw_parameters::DepthTest,
    glutin::{Api, ContextBuilder, GlProfile, GlRequest, WindowBuilder},
    index::PrimitiveType,
    texture::{ClientFormat, RawImage2d, Texture2d},
    uniforms::{UniformValue, Uniforms},
    Blend, BlendingFunction, Depth, Display, DrawParameters, Frame, IndexBuffer, LinearBlendingFactor, Program,
    Surface, VertexBuffer,
};
use log::trace;
use winit::{
    dpi::LogicalPosition as WinitLogicalPosition, ElementState as WinitElementState, Event as WinitEvent, EventsLoop,
    KeyboardInput, ModifiersState as WinitModifierState, MouseButton as WinitMouseButton,
    VirtualKeyCode as WinitVirtualKeyCode, WindowEvent,
};

use super::{
    BackendTrait, EventTrait, FrameTrait, IndexBufferTrait, ShaderTrait, TextureTrait, Vertex, VertexBufferTrait,
};
use crate::{
    assets::Image,
    components::Renderable,
    event::{ElementState, EngineEvent, LogicalPosition, ModifiersState, MouseButton, VirtualKeyCode},
    geometry::rect::Rect,
    resources::GraphicsBackend,
};

#[derive(Debug)]
pub struct GliumEvent(pub WinitEvent);

impl From<WinitEvent> for GliumEvent {
    fn from(value: WinitEvent) -> Self {
        GliumEvent(value)
    }
}

impl EventTrait for GliumEvent {}

impl TryInto<EngineEvent> for GliumEvent {
    type Error = ();

    fn try_into(self) -> Result<EngineEvent, Self::Error> {
        if let GliumEvent(WinitEvent::WindowEvent { event: we, .. }) = self {
            match we {
                WindowEvent::CloseRequested => Ok(EngineEvent::PhaseOneShutdown),
                WindowEvent::Resized(l) => Ok(EngineEvent::Resized(l.into())),
                WindowEvent::HiDpiFactorChanged(f) => Ok(EngineEvent::DpiChanged(f)),
                WindowEvent::Focused(f) => Ok(EngineEvent::Focused(f)),
                WindowEvent::CursorEntered { .. } => Ok(EngineEvent::CursorEntered),
                WindowEvent::CursorLeft { .. } => Ok(EngineEvent::CursorLeft),
                WindowEvent::CursorMoved {
                    position, modifiers, ..
                } => Ok(EngineEvent::CursorMoved {
                    position: position.into(),
                    modifiers: modifiers.into(),
                }),
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            scancode,
                            state,
                            virtual_keycode,
                            modifiers,
                        },
                    ..
                } => Ok(EngineEvent::KeyboardInput {
                    scan_code: scancode,
                    state: state.into(),
                    virtual_keycode: virtual_keycode.map(|v| v.into()),
                    modifiers: modifiers.into(),
                }),
                WindowEvent::MouseInput {
                    state,
                    button,
                    modifiers,
                    ..
                } => Ok(EngineEvent::MouseInput {
                    state: state.into(),
                    button: button.into(),
                    modifiers: modifiers.into(),
                }),
                WindowEvent::TouchpadPressure { pressure, stage, .. } => {
                    Ok(EngineEvent::TouchpadPressure { pressure, stage })
                }
                e => {
                    trace!("{:?}", &e);
                    Err(())
                }
            }
        } else {
            Err(())
        }
    }
}

impl From<WinitElementState> for ElementState {
    fn from(value: WinitElementState) -> Self {
        match value {
            WinitElementState::Pressed => ElementState::Pressed,
            WinitElementState::Released => ElementState::Released,
        }
    }
}

impl From<WinitLogicalPosition> for LogicalPosition {
    fn from(value: WinitLogicalPosition) -> Self {
        LogicalPosition { x: value.x, y: value.y }
    }
}

impl From<WinitModifierState> for ModifiersState {
    fn from(value: WinitModifierState) -> Self {
        ModifiersState {
            shift: value.shift,
            ctrl: value.ctrl,
            alt: value.alt,
            logo: value.logo,
        }
    }
}

impl From<WinitMouseButton> for MouseButton {
    fn from(value: WinitMouseButton) -> Self {
        match value {
            WinitMouseButton::Left => MouseButton::Left,
            WinitMouseButton::Right => MouseButton::Right,
            WinitMouseButton::Middle => MouseButton::Middle,
            WinitMouseButton::Other(o) => MouseButton::Other(o),
        }
    }
}

impl From<WinitVirtualKeyCode> for VirtualKeyCode {
    fn from(value: WinitVirtualKeyCode) -> Self {
        use crate::event::VirtualKeyCode::*;
        match value {
            WinitVirtualKeyCode::Key1 => Key1,
            WinitVirtualKeyCode::Key2 => Key2,
            WinitVirtualKeyCode::Key3 => Key3,
            WinitVirtualKeyCode::Key4 => Key4,
            WinitVirtualKeyCode::Key5 => Key5,
            WinitVirtualKeyCode::Key6 => Key6,
            WinitVirtualKeyCode::Key7 => Key7,
            WinitVirtualKeyCode::Key8 => Key8,
            WinitVirtualKeyCode::Key9 => Key9,
            WinitVirtualKeyCode::Key0 => Key0,
            WinitVirtualKeyCode::A => A,
            WinitVirtualKeyCode::B => B,
            WinitVirtualKeyCode::C => C,
            WinitVirtualKeyCode::D => D,
            WinitVirtualKeyCode::E => E,
            WinitVirtualKeyCode::F => F,
            WinitVirtualKeyCode::G => G,
            WinitVirtualKeyCode::H => H,
            WinitVirtualKeyCode::I => I,
            WinitVirtualKeyCode::J => J,
            WinitVirtualKeyCode::K => K,
            WinitVirtualKeyCode::L => L,
            WinitVirtualKeyCode::M => M,
            WinitVirtualKeyCode::N => N,
            WinitVirtualKeyCode::O => O,
            WinitVirtualKeyCode::P => P,
            WinitVirtualKeyCode::Q => Q,
            WinitVirtualKeyCode::R => R,
            WinitVirtualKeyCode::S => S,
            WinitVirtualKeyCode::T => T,
            WinitVirtualKeyCode::U => U,
            WinitVirtualKeyCode::V => V,
            WinitVirtualKeyCode::W => W,
            WinitVirtualKeyCode::X => X,
            WinitVirtualKeyCode::Y => Y,
            WinitVirtualKeyCode::Z => Z,
            WinitVirtualKeyCode::Escape => Escape,
            WinitVirtualKeyCode::F1 => F1,
            WinitVirtualKeyCode::F2 => F2,
            WinitVirtualKeyCode::F3 => F3,
            WinitVirtualKeyCode::F4 => F4,
            WinitVirtualKeyCode::F5 => F5,
            WinitVirtualKeyCode::F6 => F6,
            WinitVirtualKeyCode::F7 => F7,
            WinitVirtualKeyCode::F8 => F8,
            WinitVirtualKeyCode::F9 => F9,
            WinitVirtualKeyCode::F10 => F10,
            WinitVirtualKeyCode::F11 => F11,
            WinitVirtualKeyCode::F12 => F12,
            WinitVirtualKeyCode::F13 => F13,
            WinitVirtualKeyCode::F14 => F14,
            WinitVirtualKeyCode::F15 => F15,
            WinitVirtualKeyCode::F16 => F16,
            WinitVirtualKeyCode::F17 => F17,
            WinitVirtualKeyCode::F18 => F18,
            WinitVirtualKeyCode::F19 => F19,
            WinitVirtualKeyCode::F20 => F20,
            WinitVirtualKeyCode::F21 => F21,
            WinitVirtualKeyCode::F22 => F22,
            WinitVirtualKeyCode::F23 => F23,
            WinitVirtualKeyCode::F24 => F24,
            WinitVirtualKeyCode::Snapshot => PrintScreen,
            WinitVirtualKeyCode::Scroll => ScrollLock,
            WinitVirtualKeyCode::Pause => Pause,
            WinitVirtualKeyCode::Insert => Insert,
            WinitVirtualKeyCode::Home => Home,
            WinitVirtualKeyCode::Delete => Delete,
            WinitVirtualKeyCode::End => End,
            WinitVirtualKeyCode::PageDown => PageDown,
            WinitVirtualKeyCode::PageUp => PageUp,
            WinitVirtualKeyCode::Left => Left,
            WinitVirtualKeyCode::Up => Up,
            WinitVirtualKeyCode::Right => Right,
            WinitVirtualKeyCode::Down => Down,
            WinitVirtualKeyCode::Back => Backspace,
            WinitVirtualKeyCode::Return => Return,
            WinitVirtualKeyCode::Space => Space,
            WinitVirtualKeyCode::Compose => Compose,
            WinitVirtualKeyCode::Caret => Caret,
            WinitVirtualKeyCode::Numlock => NumLock,
            WinitVirtualKeyCode::Numpad0 => Numpad0,
            WinitVirtualKeyCode::Numpad1 => Numpad1,
            WinitVirtualKeyCode::Numpad2 => Numpad2,
            WinitVirtualKeyCode::Numpad3 => Numpad3,
            WinitVirtualKeyCode::Numpad4 => Numpad4,
            WinitVirtualKeyCode::Numpad5 => Numpad5,
            WinitVirtualKeyCode::Numpad6 => Numpad6,
            WinitVirtualKeyCode::Numpad7 => Numpad7,
            WinitVirtualKeyCode::Numpad8 => Numpad8,
            WinitVirtualKeyCode::Numpad9 => Numpad9,
            WinitVirtualKeyCode::AbntC1 => AbntC1,
            WinitVirtualKeyCode::AbntC2 => AbntC2,
            WinitVirtualKeyCode::Add => Add,
            WinitVirtualKeyCode::Apostrophe => Apostrophe,
            WinitVirtualKeyCode::Apps => Apps,
            WinitVirtualKeyCode::At => At,
            WinitVirtualKeyCode::Ax => Ax,
            WinitVirtualKeyCode::Backslash => Backslash,
            WinitVirtualKeyCode::Calculator => Calculator,
            WinitVirtualKeyCode::Capital => Capital,
            WinitVirtualKeyCode::Colon => Colon,
            WinitVirtualKeyCode::Comma => Comma,
            WinitVirtualKeyCode::Convert => Convert,
            WinitVirtualKeyCode::Decimal => Decimal,
            WinitVirtualKeyCode::Divide => Divide,
            WinitVirtualKeyCode::Equals => Equals,
            WinitVirtualKeyCode::Grave => Grave,
            WinitVirtualKeyCode::Kana => Kana,
            WinitVirtualKeyCode::Kanji => Kanji,
            WinitVirtualKeyCode::LAlt => LeftAlt,
            WinitVirtualKeyCode::LBracket => LeftBracket,
            WinitVirtualKeyCode::LControl => LeftControl,
            WinitVirtualKeyCode::LShift => LeftShift,
            WinitVirtualKeyCode::LWin => LeftLogo,
            WinitVirtualKeyCode::Mail => Mail,
            WinitVirtualKeyCode::MediaSelect => MediaSelect,
            WinitVirtualKeyCode::MediaStop => MediaStop,
            WinitVirtualKeyCode::Minus => Minus,
            WinitVirtualKeyCode::Multiply => Multiply,
            WinitVirtualKeyCode::Mute => Mute,
            WinitVirtualKeyCode::MyComputer => MyComputer,
            WinitVirtualKeyCode::NavigateForward => NavigateForward,
            WinitVirtualKeyCode::NavigateBackward => NavigateBackward,
            WinitVirtualKeyCode::NextTrack => NextTrack,
            WinitVirtualKeyCode::NoConvert => NoConvert,
            WinitVirtualKeyCode::NumpadComma => NumpadComma,
            WinitVirtualKeyCode::NumpadEnter => NumpadEnter,
            WinitVirtualKeyCode::NumpadEquals => NumpadEquals,
            WinitVirtualKeyCode::OEM102 => OEM102,
            WinitVirtualKeyCode::Period => Period,
            WinitVirtualKeyCode::PlayPause => PlayPause,
            WinitVirtualKeyCode::Power => Power,
            WinitVirtualKeyCode::PrevTrack => PrevTrack,
            WinitVirtualKeyCode::RAlt => RightAlt,
            WinitVirtualKeyCode::RBracket => RightBracket,
            WinitVirtualKeyCode::RControl => RightControl,
            WinitVirtualKeyCode::RShift => RightShift,
            WinitVirtualKeyCode::RWin => RightLogo,
            WinitVirtualKeyCode::Semicolon => Semicolon,
            WinitVirtualKeyCode::Slash => Slash,
            WinitVirtualKeyCode::Sleep => Sleep,
            WinitVirtualKeyCode::Stop => Stop,
            WinitVirtualKeyCode::Subtract => Subtract,
            WinitVirtualKeyCode::Sysrq => SysRq,
            WinitVirtualKeyCode::Tab => Tab,
            WinitVirtualKeyCode::Underline => Underline,
            WinitVirtualKeyCode::Unlabeled => Unlabeled,
            WinitVirtualKeyCode::VolumeDown => VolumeDown,
            WinitVirtualKeyCode::VolumeUp => VolumeUp,
            WinitVirtualKeyCode::Wake => Wake,
            WinitVirtualKeyCode::WebBack => WebBack,
            WinitVirtualKeyCode::WebFavorites => WebFavorites,
            WinitVirtualKeyCode::WebForward => WebForward,
            WinitVirtualKeyCode::WebHome => WebHome,
            WinitVirtualKeyCode::WebRefresh => WebRefresh,
            WinitVirtualKeyCode::WebSearch => WebSearch,
            WinitVirtualKeyCode::WebStop => WebStop,
            WinitVirtualKeyCode::Yen => Yen,
            WinitVirtualKeyCode::Copy => Copy,
            WinitVirtualKeyCode::Paste => Paste,
            WinitVirtualKeyCode::Cut => Cut,
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
    use std::f64;

    use approx::assert_ulps_ne;
    use try_default::TryDefault;

    use super::*;

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
