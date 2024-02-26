use std::path::PathBuf;

use winit::keyboard::{ModifiersState, SmolStr};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{AxisId, ElementState, Ime, MouseButton, MouseScrollDelta, TouchPhase},
    keyboard,
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
        event: KeyEvent,
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

    /// Touchpad magnification event with two-finger pinch gesture.
    /// Positive delta values indicate magnification (zooming in) and negative delta values indicate shrinking (zooming out).
    ///
    /// Platform-specific
    /// Only available on macOS.
    TouchpadMagnify { delta: f64, phase: TouchPhase },

    /// Touchpad rotation event with two-finger rotation gesture.
    /// Positive delta values indicate rotation counterclockwise and negative delta values indicate rotation clockwise.
    ///
    /// Platform-specific
    /// Only available on macOS.
    TouchpadRotate { delta: f32, phase: TouchPhase },

    /// Emitted when a window should be redrawn.
    ///
    /// This gets triggered in two scenarios:
    /// - The OS has performed an operation that's invalidated the window's contents (such as
    ///   resizing the window).
    /// - The application has explicitly requested a redraw via [`winit::Window::request_redraw`].
    ///
    /// Winit will aggregate duplicate redraw requests into a single event, to
    /// help avoid duplicating rendering work.
    RedrawRequested,
}

impl<'a> TryFrom<winit::event::WindowEvent> for WindowEvent {
    type Error = NonSerializableEvent;

    fn try_from(value: winit::event::WindowEvent) -> Result<Self, Self::Error> {
        use winit::event::WindowEvent::*;
        match value {
            Resized(ps) => Ok(WindowEvent::Resized(ps)),
            Moved(pp) => Ok(WindowEvent::Moved(pp)),
            CloseRequested => Ok(WindowEvent::CloseRequested),
            Destroyed => Ok(WindowEvent::Destroyed),
            DroppedFile(pb) => Ok(WindowEvent::DroppedFile(pb)),
            HoveredFile(pb) => Ok(WindowEvent::HoveredFile(pb)),
            HoveredFileCancelled => Ok(WindowEvent::HoveredFileCancelled),
            Focused(b) => Ok(WindowEvent::Focused(b)),
            KeyboardInput {
                event, is_synthetic, ..
            } => Ok(WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: event.physical_key,
                    logical_key: event.logical_key,
                    text: event.text,
                    location: event.location,
                    state: event.state,
                    repeat: event.repeat,
                },
                is_synthetic,
            }),
            ModifiersChanged(ms) => Ok(WindowEvent::ModifiersChanged(ms.state())),
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
            ActivationTokenDone { .. } => Err(NonSerializableEvent),
            TouchpadMagnify { delta, phase, .. } => Ok(WindowEvent::TouchpadMagnify { delta, phase }),
            SmartMagnify { .. } => Err(NonSerializableEvent),
            TouchpadRotate { delta, phase, .. } => Ok(WindowEvent::TouchpadRotate { delta, phase }),
            RedrawRequested => Ok(WindowEvent::RedrawRequested),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub struct KeyEvent {
    /// Represents the position of a key independent of the currently active layout.
    ///
    /// It also uniquely identifies the physical key (i.e. it's mostly synonymous with a scancode).
    /// The most prevalent use case for this is games. For example the default keys for the player
    /// to move around might be the W, A, S, and D keys on a US layout. The position of these keys
    /// is more important than their label, so they should map to Z, Q, S, and D on an "AZERTY"
    /// layout. (This value is `KeyCode::KeyW` for the Z key on an AZERTY layout.)
    ///
    /// ## Caveats
    ///
    /// - Certain niche hardware will shuffle around physical key positions, e.g. a keyboard that
    /// implements DVORAK in hardware (or firmware)
    /// - Your application will likely have to handle keyboards which are missing keys that your
    /// own keyboard has.
    /// - Certain `KeyCode`s will move between a couple of different positions depending on what
    /// layout the keyboard was manufactured to support.
    ///
    ///  **Because of these caveats, it is important that you provide users with a way to configure
    ///  most (if not all) keybinds in your application.**
    ///
    /// ## `Fn` and `FnLock`
    ///
    /// `Fn` and `FnLock` key events are *exceedingly unlikely* to be emitted by Winit. These keys
    /// are usually handled at the hardware or OS level, and aren't surfaced to applications. If
    /// you somehow see this in the wild, we'd like to know :)
    pub physical_key: winit::keyboard::PhysicalKey,

    /// This value is affected by all modifiers except <kbd>Ctrl</kbd>.
    ///
    /// This has two use cases:
    /// - Allows querying whether the current input is a Dead key.
    /// - Allows handling key-bindings on platforms which don't
    /// support [`key_without_modifiers`].
    ///
    /// If you use this field (or [`key_without_modifiers`] for that matter) for keyboard
    /// shortcuts, **it is important that you provide users with a way to configure your
    /// application's shortcuts so you don't render your application unusable for users with an
    /// incompatible keyboard layout.**
    ///
    /// ## Platform-specific
    /// - **Web:** Dead keys might be reported as the real key instead
    /// of `Dead` depending on the browser/OS.
    ///
    /// [`key_without_modifiers`]: crate::platform::modifier_supplement::KeyEventExtModifierSupplement::key_without_modifiers
    pub logical_key: winit::keyboard::Key,

    /// Contains the text produced by this keypress.
    ///
    /// In most cases this is identical to the content
    /// of the `Character` variant of `logical_key`.
    /// However, on Windows when a dead key was pressed earlier
    /// but cannot be combined with the character from this
    /// keypress, the produced text will consist of two characters:
    /// the dead-key-character followed by the character resulting
    /// from this keypress.
    ///
    /// An additional difference from `logical_key` is that
    /// this field stores the text representation of any key
    /// that has such a representation. For example when
    /// `logical_key` is `Key::Named(NamedKey::Enter)`, this field is `Some("\r")`.
    ///
    /// This is `None` if the current keypress cannot
    /// be interpreted as text.
    ///
    /// See also: `text_with_all_modifiers()`
    pub text: Option<SmolStr>,

    /// Contains the location of this key on the keyboard.
    ///
    /// Certain keys on the keyboard may appear in more than once place. For example, the "Shift" key
    /// appears on the left side of the QWERTY keyboard as well as the right side. However, both keys
    /// have the same symbolic value. Another example of this phenomenon is the "1" key, which appears
    /// both above the "Q" key and as the "Keypad 1" key.
    ///
    /// This field allows the user to differentiate between keys like this that have the same symbolic
    /// value but different locations on the keyboard.
    ///
    /// See the [`KeyLocation`] type for more details.
    ///
    /// [`KeyLocation`]: crate::keyboard::KeyLocation
    pub location: keyboard::KeyLocation,

    /// Whether the key is being pressed or released.
    ///
    /// See the [`ElementState`] type for more details.
    pub state: ElementState,

    /// Whether or not this key is a key repeat event.
    ///
    /// On some systems, holding down a key for some period of time causes that key to be repeated
    /// as though it were being pressed and released repeatedly. This field is `true` if and only if
    /// this event is the result of one of those repeats.
    pub repeat: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, thiserror::Error)]
#[error("Cannot create a serializable mapping from the given winit event")]
pub struct NonSerializableEvent;
