use std::path::PathBuf;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{AxisId, ElementState, Ime, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta, TouchPhase},
};

/// A serializable copy of [`WindowEvent`](winit::event::WindowEvent). Some fields and variants are
/// missing:
///
/// * `Touch(_)`
/// * `ThemeChanged(_)`
/// * `DeviceId`
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WindowEvent {
    /// The size of the window has changed. Contains the client area's new dimensions.
    Resized(PhysicalSize<u32>),
    /// The position of the window has changed. Contains the window's new position.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web / Wayland:** Unsupported.
    Moved(PhysicalPosition<i32>),
    /// The window has been requested to close.
    CloseRequested,
    /// The window has been destroyed.
    Destroyed,
    /// A file has been dropped into the window.
    ///
    /// When the user drops multiple files at once, this event will be emitted for each file
    /// separately.
    DroppedFile(PathBuf),

    /// A file is being hovered over the window.
    ///
    /// When the user hovers multiple files at once, this event will be emitted for each file
    /// separately.
    HoveredFile(PathBuf),

    /// A file was hovered, but has exited the window.
    ///
    /// There will be a single `HoveredFileCancelled` event triggered even if multiple files were
    /// hovered.
    HoveredFileCancelled,

    /// The window received a unicode character.
    ///
    /// See also the [`Ime`](Self::Ime) event for more complex character sequences.
    ReceivedCharacter(char),

    /// The window gained or lost focus.
    ///
    /// The parameter is true if the window has gained focus, and false if it has lost focus.
    Focused(bool),

    /// An event from the keyboard has been received.
    KeyboardInput {
        input: KeyboardInput,
        /// If `true`, the event was generated synthetically by winit
        /// in one of the following circumstances:
        ///
        /// * Synthetic key press events are generated for all keys pressed
        ///   when a window gains focus. Likewise, synthetic key release events
        ///   are generated for all keys pressed when a window goes out of focus.
        ///   ***Currently, this is only functional on X11 and Windows***
        ///
        /// Otherwise, this value is always `false`.
        is_synthetic: bool,
    },

    /// The keyboard modifiers have changed.
    ///
    /// ## Platform-specific
    ///
    /// - **Web:** This API is currently unimplemented on the web. This isn't by design - it's an
    ///   issue, and it should get fixed - but it's the current state of the API.
    ModifiersChanged(ModifiersState),

    /// An event from an input method.
    ///
    /// **Note:** You have to explicitly enable this event using [`Window::set_ime_allowed`].
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web:** Unsupported.
    Ime(Ime),

    /// The cursor has moved on the window.
    CursorMoved {
        /// (x,y) coords in pixels relative to the top-left corner of the window. Because the range of this data is
        /// limited by the display area and it may have been transformed by the OS to implement effects such as cursor
        /// acceleration, it should not be used to implement non-cursor-like interactions such as 3D camera control.
        position: PhysicalPosition<f64>,
    },

    /// The cursor has entered the window.
    CursorEntered,

    /// The cursor has left the window.
    CursorLeft,

    /// A mouse wheel movement or touchpad scroll occurred.
    MouseWheel { delta: MouseScrollDelta, phase: TouchPhase },

    /// An mouse button press has been received.
    MouseInput { state: ElementState, button: MouseButton },

    /// Touchpad pressure event.
    ///
    /// At the moment, only supported on Apple forcetouch-capable macbooks.
    /// The parameters are: pressure level (value between 0 and 1 representing how hard the touchpad
    /// is being pressed) and stage (integer representing the click level).
    TouchpadPressure { pressure: f32, stage: i64 },

    /// Motion on some analog axis. May report data redundant to other, more specific events.
    AxisMotion { axis: AxisId, value: f64 },

    /// The window's scale factor has changed.
    ///
    /// The following user actions can cause DPI changes:
    ///
    /// * Changing the display's resolution.
    /// * Changing the display's scale factor (e.g. in Control Panel on Windows).
    /// * Moving the window to a display with a different scale factor.
    ///
    /// After this event callback has been processed, the window will be resized to whatever value
    /// is pointed to by the `new_inner_size` reference. By default, this will contain the size suggested
    /// by the OS, but it can be changed to any value.
    ///
    /// For more information about DPI in general, see the [`dpi`](crate::dpi) module.
    ScaleFactorChanged { scale_factor: f64 },

    /// The window has been occluded (completely hidden from view).
    ///
    /// This is different to window visibility as it depends on whether the window is closed,
    /// minimised, set invisible, or fully occluded by another window.
    ///
    /// Platform-specific behavior:
    /// - **iOS / Android / Web / Wayland / Windows:** Unsupported.
    Occluded(bool),
}

impl<'a> TryFrom<winit::event::WindowEvent<'a>> for WindowEvent {
    type Error = NonSerializableEvent;

    fn try_from(value: winit::event::WindowEvent<'a>) -> Result<Self, Self::Error> {
        use winit::event::WindowEvent::*;
        match value {
            Resized(ps) => Ok(WindowEvent::Resized(ps)),
            Moved(pp) => Ok(WindowEvent::Moved(pp)),
            CloseRequested => Ok(WindowEvent::CloseRequested),
            Destroyed => Ok(WindowEvent::Destroyed),
            DroppedFile(pb) => Ok(WindowEvent::DroppedFile(pb)),
            HoveredFile(pb) => Ok(WindowEvent::HoveredFile(pb)),
            HoveredFileCancelled => Ok(WindowEvent::HoveredFileCancelled),
            ReceivedCharacter(c) => Ok(WindowEvent::ReceivedCharacter(c)),
            Focused(b) => Ok(WindowEvent::Focused(b)),
            KeyboardInput {
                input, is_synthetic, ..
            } => Ok(WindowEvent::KeyboardInput { input, is_synthetic }),
            ModifiersChanged(ms) => Ok(WindowEvent::ModifiersChanged(ms)),
            Ime(ime) => Ok(WindowEvent::Ime(ime)),
            CursorMoved { position, .. } => Ok(WindowEvent::CursorMoved { position }),
            CursorEntered { .. } => Ok(WindowEvent::CursorEntered),
            CursorLeft { .. } => Ok(WindowEvent::CursorLeft),
            MouseWheel { delta, phase, .. } => Ok(WindowEvent::MouseWheel { delta, phase }),
            MouseInput { state, button, .. } => Ok(WindowEvent::MouseInput { state, button }),
            TouchpadPressure { pressure, stage, .. } => Ok(WindowEvent::TouchpadPressure { pressure, stage }),
            AxisMotion { axis, value, .. } => Ok(WindowEvent::AxisMotion { axis, value }),
            Touch(_) => Err(NonSerializableEvent),
            ScaleFactorChanged { scale_factor, .. } => Ok(WindowEvent::ScaleFactorChanged { scale_factor }),
            ThemeChanged(_) => Err(NonSerializableEvent),
            Occluded(b) => Ok(WindowEvent::Occluded(b)),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, thiserror::Error)]
#[error("Cannot create a serializable mapping from the given event")]
pub struct NonSerializableEvent;
