use crate::{
    event::EngineEventTrait,
    graphics::{glium::GliumEvent, headless::HeadlessEvent},
};
use ecs::EventTrait;
use glium::glutin::{Event as GlutinEvent, WindowEvent};
#[cfg(target_os = "macos")]
use glium::glutin::{KeyboardInput, ModifiersState, VirtualKeyCode};
use std::convert::TryFrom;

bitflags! {
    pub struct MockEvtFlag: u64 {
        const STARTUP = 0x01;
        const SHUTDOWN = 0x02;
        const HARD_SHUTDOWN = 0x04;
        const COMMAND = 0x08;
        const RESIZE = 0x10;
        const CHANGE_DPI = 0x20;
    }
}

impl Default for MockEvtFlag {
    fn default() -> Self {
        MockEvtFlag::all()
    }
}

#[derive(Debug, Clone)]
pub struct MockEvt {
    flag: MockEvtFlag,
    data: MockEvtData,
}

impl MockEvt {
    pub fn new(flag: MockEvtFlag, data: MockEvtData) -> Self {
        MockEvt {
            flag,
            data,
        }
    }
}

impl EngineEventTrait for MockEvt {
    fn startup() -> MockEvtFlag {
        MockEvtFlag::STARTUP
    }

    fn shutdown() -> MockEvtFlag {
        MockEvtFlag::SHUTDOWN
    }

    fn hard_shutdown() -> MockEvtFlag {
        MockEvtFlag::HARD_SHUTDOWN
    }

    fn command() -> MockEvtFlag {
        MockEvtFlag::COMMAND
    }

    fn resize() -> MockEvtFlag {
        MockEvtFlag::RESIZE
    }

    fn change_dpi() -> MockEvtFlag {
        MockEvtFlag::CHANGE_DPI
    }

    fn new_startup() -> Self {
        MockEvt::new(
            MockEvtFlag::STARTUP,
            MockEvtData::Empty
        )
    }

    fn new_shutdown() -> Self {
        MockEvt::new(
            MockEvtFlag::SHUTDOWN,
            MockEvtData::Empty,
        )
    }

    fn new_hard_shutdown() -> Self {
        MockEvt::new(
            MockEvtFlag::HARD_SHUTDOWN,
            MockEvtData::Empty,
        )
    }

    fn new_command(args: Vec<String>) -> Self {
        MockEvt::new(
            MockEvtFlag::COMMAND,
            MockEvtData::Command(args),
        )
    }

    fn new_resize(dims: (u32, u32)) -> Self {
        MockEvt::new(
            MockEvtFlag::RESIZE,
            MockEvtData::Resize(dims),
        )
    }

    fn new_change_dpi(factor: f64) -> Self {
        MockEvt::new(
            MockEvtFlag::CHANGE_DPI,
            MockEvtData::ChangeDpi(factor),
        )
    }

    fn flag(&self) -> MockEvtFlag {
        self.flag
    }

    fn command_data(&self) -> Option<&[String]> {
        match self.data {
            MockEvtData::Command(ref args) => Some(args),
            _ => None,
        }
    }

    fn resize_data(&self) -> Option<(u32, u32)> {
        match self.data {
            MockEvtData::Resize(dims) => Some(dims),
            _ => None,
        }
    }

    fn change_dpi_data(&self) -> Option<f64> {
        match self.data {
            MockEvtData::ChangeDpi(factor) => Some(factor),
            _ => None,
        }
    }
}

impl TryFrom<HeadlessEvent> for MockEvt {
    type Error = ();

    fn try_from(_value: HeadlessEvent) -> Result<Self, Self::Error> {
        Err(())
    }
}

impl TryFrom<GliumEvent> for MockEvt {
    type Error = ();

    fn try_from(value: GliumEvent) -> Result<Self, Self::Error> {
        if let GliumEvent(GlutinEvent::WindowEvent { event: we, .. }) = value {
            match we {
                WindowEvent::CloseRequested => Ok(MockEvt::new_shutdown()),
                WindowEvent::Resized(l) => Ok(MockEvt::new_resize(l.into())),
                WindowEvent::HiDpiFactorChanged(f) => Ok(MockEvt::new_change_dpi(f)),
                #[cfg(target_os = "macos")]
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Q),
                            modifiers: ModifiersState { logo: true, .. },
                            ..
                        },
                    ..
                } => Ok(MockEvt::new_shutdown()),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}

impl EventTrait for MockEvt {
    type EventFlag = MockEvtFlag;

    fn matches_filter(&self, flag: Self::EventFlag) -> bool {
        flag.contains(self.flag)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MockEvtData {
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
        assert_eq!(MockEvtFlag::default(), MockEvtFlag::all());
    }

    #[test]
    fn flags() {
        assert_eq!(MockEvt::startup(), MockEvtFlag::STARTUP);
        assert_eq!(MockEvt::shutdown(), MockEvtFlag::SHUTDOWN);
        assert_eq!(MockEvt::hard_shutdown(), MockEvtFlag::HARD_SHUTDOWN);
        assert_eq!(MockEvt::command(), MockEvtFlag::COMMAND);
        assert_eq!(MockEvt::resize(), MockEvtFlag::RESIZE);
        assert_eq!(MockEvt::change_dpi(), MockEvtFlag::CHANGE_DPI);
    }

    #[test]
    fn accessors() {
        let e: MockEvt = MockEvt::new_startup();
        let _: MockEvtFlag = e.flag();
    }

    #[test]
    fn ready_event() {
        let e: MockEvt = MockEvt::new_startup();
        assert_eq!(e.flag(), MockEvtFlag::STARTUP);
    }

    #[test]
    fn shutdown_event() {
        let e: MockEvt = MockEvt::new_shutdown();
        assert_eq!(e.flag(), MockEvtFlag::SHUTDOWN);
    }

    #[test]
    fn hard_shutdown_event() {
        let e: MockEvt = MockEvt::new_hard_shutdown();
        assert_eq!(e.flag(), MockEvtFlag::HARD_SHUTDOWN);
    }

    #[test]
    fn command_event() {
        let e: MockEvt = MockEvt::new_command(vec![String::from("echo")]);
        assert_eq!(e.flag(), MockEvtFlag::COMMAND);
        assert_eq!(e.command_data(), Some(vec![String::from("echo")].as_slice()));
    }

    #[test]
    fn resize_event() {
        let e: MockEvt = MockEvt::new_resize((1, 2));
        assert_eq!(e.flag(), MockEvtFlag::RESIZE);
        assert_eq!(e.resize_data(), Some((1, 2)));
    }

    #[test]
    fn change_dpi_event() {
        let e: MockEvt = MockEvt::new_change_dpi(2.0);
        assert_eq!(e.flag(), MockEvtFlag::CHANGE_DPI);
        assert_eq!(e.change_dpi_data(), Some(2.0));
    }
}
