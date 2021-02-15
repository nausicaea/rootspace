use ecs::{Resource, MaybeDefault};
use file_manipulation::DirPathBuf;
use serde::{Serialize, Deserialize};

pub trait SettingsTrait: Resource {
    fn asset_tree(&self) -> &DirPathBuf;
    fn title(&self) -> &str;
    fn dimensions(&self) -> (u32, u32);
    fn clear_color(&self) -> [f32; 4];
    fn vsync(&self) -> bool;
    fn msaa(&self) -> u16;
    fn command_escape(&self) -> char;
    fn command_quote(&self) -> char;
    fn command_punctuation(&self) -> char;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    asset_tree: DirPathBuf,
    title: String,
    dimensions: (u32, u32),
    clear_color: [f32; 4],
    vsync: bool,
    msaa: u16,
    command_escape: char,
    command_quote: char,
    command_punctuation: char,
}

impl Settings {
    pub fn builder(asset_tree: DirPathBuf) -> SettingsBuilder {
        SettingsBuilder::new(asset_tree)
    }
}

impl SettingsTrait for Settings {
    fn asset_tree(&self) -> &DirPathBuf {
        &self.asset_tree
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    fn clear_color(&self) -> [f32; 4] {
        self.clear_color
    }

    fn vsync(&self) -> bool {
        self.vsync
    }

    fn msaa(&self) -> u16 {
        self.msaa
    }

    fn command_escape(&self) -> char {
        self.command_escape
    }

    fn command_quote(&self) -> char {
        self.command_quote
    }

    fn command_punctuation(&self) -> char {
        self.command_punctuation
    }
}

impl Resource for Settings {}

impl MaybeDefault for Settings {}

impl From<SettingsBuilder> for Settings {
    fn from(value: SettingsBuilder) -> Self {
        Settings {
            asset_tree: value.asset_tree,
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

    pub fn build(self) -> Settings {
        Settings::from(self)
    }
}
