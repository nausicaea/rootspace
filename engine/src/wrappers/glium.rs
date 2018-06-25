use ecs::event::EventTrait;
use event::Event;
use failure::Error as FailureError;
use glium::glutin::{
    Api, ContextBuilder, Event as WinitEvent, EventsLoop as WinitEventsLoop, GlProfile, GlRequest,
    WindowBuilder,
};
use glium::{Display as GliumDisplay, Frame as GliumFrame, Surface as GliumSurface};
use std::convert::{TryFrom, TryInto};

pub trait EventsLoopTrait<E>
where
    E: EventTrait,
{
    type OsEvent: TryInto<E>;

    fn poll<F>(&mut self, f: F)
    where
        F: FnMut(Self::OsEvent);
}

pub trait FrameTrait {
    fn clear(&mut self, color: &[f32; 4], depth: f32);
    fn render(&mut self) -> Result<(), FailureError>;
    fn finalize(self) -> Result<(), FailureError>;
}

pub trait DisplayTrait
where
    Self: Sized,
{
    type EventsLoop;
    type Frame: FrameTrait;

    fn create(
        events_loop: &Self::EventsLoop,
        title: &str,
        dimensions: &[u32; 2],
        vsync: bool,
        msaa: u16,
    ) -> Result<Self, FailureError>;
    fn create_frame(&self) -> Self::Frame;
}

#[derive(Default)]
pub struct HeadlessEventsLoop;

impl EventsLoopTrait<Event> for HeadlessEventsLoop {
    type OsEvent = ();

    fn poll<F>(&mut self, _f: F)
    where
        F: FnMut(Self::OsEvent),
    {
    }
}

impl TryFrom<()> for Event {
    type Error = ();

    fn try_from(_value: ()) -> Result<Event, Self::Error> {
        Err(())
    }
}

#[derive(Default)]
pub struct HeadlessFrame;

impl FrameTrait for HeadlessFrame {
    fn clear(&mut self, _color: &[f32; 4], _depth: f32) {}

    fn render(&mut self) -> Result<(), FailureError> {
        Ok(())
    }

    fn finalize(self) -> Result<(), FailureError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct HeadlessDisplay;

impl DisplayTrait for HeadlessDisplay {
    type EventsLoop = HeadlessEventsLoop;
    type Frame = HeadlessFrame;

    fn create(
        _events_loop: &Self::EventsLoop,
        _title: &str,
        _dimensions: &[u32; 2],
        _vsync: bool,
        _msaa: u16,
    ) -> Result<Self, FailureError> {
        Ok(Default::default())
    }

    fn create_frame(&self) -> Self::Frame {
        Default::default()
    }
}

impl TryFrom<WinitEvent> for Event {
    type Error = ();

    fn try_from(value: WinitEvent) -> Result<Event, Self::Error> {
        if let WinitEvent::WindowEvent { event: _we, .. } = value {
            unimplemented!()
        } else {
            Err(())
        }
    }
}

impl EventsLoopTrait<Event> for WinitEventsLoop {
    type OsEvent = WinitEvent;

    fn poll<F>(&mut self, f: F)
    where
        F: FnMut(Self::OsEvent),
    {
        self.poll_events(f)
    }
}

impl FrameTrait for GliumFrame {
    fn clear(&mut self, color: &[f32; 4], depth: f32) {
        self.clear_color_and_depth((color[0], color[1], color[2], color[3]), depth)
    }

    fn render(&mut self) -> Result<(), FailureError> {
        // self.draw(MultiVerticesSource, Into<IndicesSource>, &Program, &Uniforms, &DrawParameters)
        unimplemented!()
    }

    fn finalize(self) -> Result<(), FailureError> {
        if let Err(e) = self.finish() {
            Err(Into::into(e))
        } else {
            Ok(())
        }
    }
}

impl DisplayTrait for GliumDisplay {
    type EventsLoop = WinitEventsLoop;
    type Frame = GliumFrame;

    fn create(
        events_loop: &Self::EventsLoop,
        title: &str,
        dimensions: &[u32; 2],
        vsync: bool,
        msaa: u16,
    ) -> Result<Self, FailureError> {
        let window = WindowBuilder::new()
            .with_title(title)
            .with_dimensions(dimensions[0], dimensions[1]);

        let context = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core)
            .with_vsync(vsync)
            .with_multisampling(msaa);

        match GliumDisplay::new(window, context, events_loop) {
            Ok(d) => Ok(d),
            Err(e) => Err(format_err!("{}", e)),
        }
    }

    fn create_frame(&self) -> Self::Frame {
        self.draw()
    }
}
