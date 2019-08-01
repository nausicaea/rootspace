use ecs::EventTrait;
use engine::{
    event::EngineEventTrait,
    graphics::{glium::GliumEvent, headless::HeadlessEvent},
};
use glium::glutin::{Event as GlutinEvent, WindowEvent};
#[cfg(target_os = "macos")]
use glium::glutin::{KeyboardInput, ModifiersState, VirtualKeyCode};
use std::convert::TryFrom;

bitflags! {
    pub struct EventFlag: u64 {
        const STARTUP = 0x01;
        const SHUTDOWN = 0x02;
        const HARD_SHUTDOWN = 0x04;
        const COMMAND = 0x08;
        const RESIZE = 0x10;
        const CHANGE_DPI = 0x20;
    }
}

impl Default for EventFlag {
    fn default() -> Self {
        EventFlag::all()
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    flag: EventFlag,
    data: EventData,
}

impl EngineEventTrait for Event {
    fn startup() -> EventFlag {
        EventFlag::STARTUP
    }

    fn shutdown() -> EventFlag {
        EventFlag::SHUTDOWN
    }

    fn hard_shutdown() -> EventFlag {
        EventFlag::HARD_SHUTDOWN
    }

    fn command() -> EventFlag {
        EventFlag::COMMAND
    }

    fn resize() -> EventFlag {
        EventFlag::RESIZE
    }

    fn change_dpi() -> EventFlag {
        EventFlag::CHANGE_DPI
    }

    fn new_startup() -> Self {
        Event {
            flag: EventFlag::STARTUP,
            data: EventData::Empty,
        }
    }

    fn new_shutdown() -> Self {
        Event {
            flag: EventFlag::SHUTDOWN,
            data: EventData::Empty,
        }
    }

    fn new_hard_shutdown() -> Self {
        Event {
            flag: EventFlag::HARD_SHUTDOWN,
            data: EventData::Empty,
        }
    }

    fn new_command(args: Vec<String>) -> Self {
        Event {
            flag: EventFlag::COMMAND,
            data: EventData::Command(args),
        }
    }

    fn new_resize(dims: (u32, u32)) -> Self {
        Event {
            flag: EventFlag::RESIZE,
            data: EventData::Resize(dims),
        }
    }

    fn new_change_dpi(factor: f64) -> Self {
        Event {
            flag: EventFlag::CHANGE_DPI,
            data: EventData::ChangeDpi(factor),
        }
    }

    fn flag(&self) -> EventFlag {
        self.flag
    }

    fn command_data(&self) -> Option<&[String]> {
        match self.data {
            EventData::Command(ref args) => Some(args),
            _ => None,
        }
    }

    fn resize_data(&self) -> Option<(u32, u32)> {
        match self.data {
            EventData::Resize(dims) => Some(dims),
            _ => None,
        }
    }

    fn change_dpi_data(&self) -> Option<f64> {
        match self.data {
            EventData::ChangeDpi(factor) => Some(factor),
            _ => None,
        }
    }
}

impl EventTrait for Event {
    type EventFlag = EventFlag;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool {
        flag.contains(self.flag)
    }
}

impl TryFrom<HeadlessEvent> for Event {
    type Error = ();

    fn try_from(_value: HeadlessEvent) -> Result<Self, Self::Error> {
        Err(())
    }
}

impl TryFrom<GliumEvent> for Event {
    type Error = ();

    fn try_from(value: GliumEvent) -> Result<Self, Self::Error> {
        if let GliumEvent(GlutinEvent::WindowEvent { event: we, .. }) = value {
            match we {
                WindowEvent::CloseRequested => Ok(Event::new_shutdown()),
                WindowEvent::Resized(l) => Ok(Event::new_resize(l.into())),
                WindowEvent::HiDpiFactorChanged(f) => Ok(Event::new_change_dpi(f)),
                #[cfg(target_os = "macos")]
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Q),
                            modifiers: ModifiersState { logo: true, .. },
                            ..
                        },
                    ..
                } => Ok(Event::new_shutdown()),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventData {
    Empty,
    Command(Vec<String>),
    Resize((u32, u32)),
    ChangeDpi(f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_event_flag() {
        assert_eq!(EventFlag::default(), EventFlag::all());
    }

    #[test]
    fn flags() {
        assert_eq!(Event::startup(), EventFlag::STARTUP);
        assert_eq!(Event::shutdown(), EventFlag::SHUTDOWN);
        assert_eq!(Event::hard_shutdown(), EventFlag::HARD_SHUTDOWN);
        assert_eq!(Event::command(), EventFlag::COMMAND);
        assert_eq!(Event::resize(), EventFlag::RESIZE);
        assert_eq!(Event::change_dpi(), EventFlag::CHANGE_DPI);
    }

    #[test]
    fn accessors() {
        let e = Event::new_startup();
        let _: EventFlag = e.flag();
    }

    #[test]
    fn ready_event() {
        let e = Event::new_startup();
        assert_eq!(e.flag(), EventFlag::STARTUP);
    }

    #[test]
    fn shutdown_event() {
        let e = Event::new_shutdown();
        assert_eq!(e.flag(), EventFlag::SHUTDOWN);
    }

    #[test]
    fn hard_shutdown_event() {
        let e = Event::new_hard_shutdown();
        assert_eq!(e.flag(), EventFlag::HARD_SHUTDOWN);
    }

    #[test]
    fn command_event() {
        let e = Event::new_command(vec![String::from("echo")]);
        assert_eq!(e.flag(), EventFlag::COMMAND);
        assert_eq!(e.command_data(), Some(vec![String::from("echo")].as_slice()));
    }

    #[test]
    fn resize_event() {
        let e = Event::new_resize((1, 2));
        assert_eq!(e.flag(), EventFlag::RESIZE);
        assert_eq!(e.resize_data(), Some((1, 2)));
    }

    #[test]
    fn change_dpi_event() {
        let e = Event::new_change_dpi(2.0);
        assert_eq!(e.flag(), EventFlag::CHANGE_DPI);
        assert_eq!(e.change_dpi_data(), Some(2.0));
    }
}
