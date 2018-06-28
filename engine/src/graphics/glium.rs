use super::{DisplayTrait, EventsLoopTrait, FrameTrait, Uniforms};
use event::Event;
use failure::Error as FailureError;
use glium::glutin::{
    Api, ContextBuilder, Event as WinitEvent, EventsLoop as WinitEventsLoop, GlProfile, GlRequest,
    WindowBuilder,
};
use glium::uniforms::{UniformValue, Uniforms as GliumUniforms};
use glium::{Display as GliumDisplay, Frame as GliumFrame, Surface as GliumSurface};
use std::convert::TryFrom;

impl GliumUniforms for Uniforms {
    fn visit_values<'a, F>(&'a self, mut f: F)
    where
        F: FnMut(&str, UniformValue<'a>),
    {
        f("pvm_matrix", UniformValue::Mat4(self.pvm_matrix.into()));
        // f("name", uniform_value);
        // I need: pvm_matrix, normal_matrix, optionally diff_tex, optionally norm_tex
    }
}

impl TryFrom<WinitEvent> for Event {
    type Error = ();

    fn try_from(value: WinitEvent) -> Result<Event, Self::Error> {
        if let WinitEvent::WindowEvent { event: _we, .. } = value {
            Err(())
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
    fn clear_frame(&mut self, color: [f32; 4], depth: f32) {
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

impl DisplayTrait for GliumDisplay {
    type EventsLoop = WinitEventsLoop;
    type Frame = GliumFrame;

    fn create(
        events_loop: &Self::EventsLoop,
        title: &str,
        dimensions: [u32; 2],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(not(windows), should_panic(expected = "No backend is available"))]
    fn display() {
        let e = WinitEventsLoop::new();
        let r = GliumDisplay::create(&e, "Title", [640, 480], false, 0);

        assert!(r.is_ok());

        let mut f = r.unwrap().draw();
        f.clear_frame([0.0, 1.0, 0.0, 1.0], 1.0);
        let r = f.finalize();

        assert_ok!(r);
    }
}
