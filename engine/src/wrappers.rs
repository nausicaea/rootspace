use ecs::event::EventTrait;
use event::Event;
use failure::Error as FailureError;
use glium::glutin::{Event as WinitEvent, EventsLoop as WinitEventsLoop, WindowBuilder, ContextBuilder, Api, GlProfile, GlRequest};
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

impl EventsLoopTrait<Event> for () {
    type OsEvent = Event;

    fn poll<F>(&mut self, _f: F)
    where
        F: FnMut(Self::OsEvent),
    {
    }
}

pub trait FrameTrait {
    fn clear(&mut self, color: &[f32; 4], depth: f32);
    fn finalize(self) -> Result<(), FailureError>;
}

impl FrameTrait for GliumFrame {
    fn clear(&mut self, color: &[f32; 4], depth: f32) {
        self.clear_color_and_depth((color[0], color[1], color[2], color[3]), depth)
    }

    fn finalize(self) -> Result<(), FailureError> {
        if let Err(e) = self.finish() {
            Err(Into::into(e))
        } else {
            Ok(())
        }
    }
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
