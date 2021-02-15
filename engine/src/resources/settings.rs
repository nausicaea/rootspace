use ecs::{Resource, MaybeDefault};
use file_manipulation::DirPathBuf;
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub asset_tree: DirPathBuf,
    pub delta_time: Duration,
    pub max_frame_time: Duration,
    pub title: String,
    pub dimensions: (u32, u32),
    pub clear_color: [f32; 4],
    pub vsync: bool,
    pub msaa: u16,
    pub command_escape: char,
    pub command_quote: char,
    pub command_punctuation: char,
}

impl Settings {
    pub fn builder(asset_tree: DirPathBuf) -> SettingsBuilder {
        SettingsBuilder::new(asset_tree)
    }
}

impl Resource for Settings {}

impl MaybeDefault for Settings {}

impl From<SettingsBuilder> for Settings {
    fn from(value: SettingsBuilder) -> Self {
        Settings {
            asset_tree: value.asset_tree,
            delta_time: value.delta_time,
            max_frame_time: value.max_frame_time,
            title: value.title,
            dimensions: value.dimensions,
            clear_color: value.clear_color,
            vsync: value.vsync,
            msaa: value.msaa,
            command_escape: value.command_escape,
            command_quote: value.command_quote,
            command_punctuation: value.command_punctuation,
        }
    }
}

pub struct SettingsBuilder {
    asset_tree: DirPathBuf,
    delta_time: Duration,
    max_frame_time: Duration,
    title: String,
    dimensions: (u32, u32),
    clear_color: [f32; 4],
    vsync: bool,
    msaa: u16,
    command_escape: char,
    command_quote: char,
    command_punctuation: char,
}

impl SettingsBuilder {
    pub fn new(asset_tree: DirPathBuf) -> Self {
        SettingsBuilder {
            asset_tree,
            delta_time: Duration::from_millis(50),
            max_frame_time: Duration::from_millis(250),
            title: String::new(),
            dimensions: (800, 600),
            clear_color: [0.69, 0.93, 0.93, 1.0],
            vsync: true,
            msaa: 4,
            command_escape: '\\',
            command_quote: '"',
            command_punctuation: ';',
        }
    }

    pub fn with_title<S: AsRef<str>>(mut self, title: S) -> Self {
        self.title = title.as_ref().to_string();
        self
    }

    pub fn with_dimensions(mut self, dims: (u32, u32)) -> Self {
        self.dimensions = dims;
        self
    }

    pub fn with_clear_color(mut self, clear_color: [f32; 4]) -> Self {
        self.clear_color = clear_color;
        self
    }

    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    pub fn with_msaa(mut self, msaa: u16) -> Self {
        self.msaa = msaa;
        self
    }

    pub fn with_command_escape(mut self, escape: char) -> Self {
        self.command_escape = escape;
        self
    }

    pub fn with_command_quote(mut self, quote: char) -> Self {
        self.command_quote = quote;
        self
    }

    pub fn with_command_punctuation(mut self, punct: char) -> Self {
        self.command_punctuation = punct;
        self
    }

    pub fn with_delta_time(mut self, dt: Duration) -> Self {
        self.delta_time = dt;
        self
    }

    pub fn with_max_frame_time(mut self, mft: Duration) -> Self {
        self.max_frame_time = mft;
        self
    }

    pub fn build(self) -> Settings {
        Settings::from(self)
    }
}
