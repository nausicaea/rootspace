use std::convert::TryFrom;

use ecs::{MaybeDefault, Resource};
use file_manipulation::DirPathBuf;

use crate::graphics::BackendTrait;
use crate::resources::BackendResource;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendSettings {
    pub asset_tree: DirPathBuf,
    pub title: String,
    pub dimensions: (u32, u32),
    pub clear_color: [f32; 4],
    pub vsync: bool,
    pub msaa: u16,
    pub command_escape: char,
    pub command_quote: char,
    pub command_punctuation: char,
}

impl BackendSettings {
    pub fn builder(asset_tree: DirPathBuf) -> BackendSettingsBuilder {
        BackendSettingsBuilder::new(asset_tree)
    }

    pub fn build_backend<B: BackendTrait>(&self) -> anyhow::Result<BackendResource<B>> {
        TryFrom::try_from(self)
    }
}

impl Resource for BackendSettings {}

impl MaybeDefault for BackendSettings {}

impl<B> From<BackendResource<B>> for BackendSettings
where
    B: BackendTrait,
{
    fn from(value: BackendResource<B>) -> Self {
        value.settings.clone()
    }
}

impl From<BackendSettingsBuilder> for BackendSettings {
    fn from(value: BackendSettingsBuilder) -> Self {
        BackendSettings {
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

pub struct BackendSettingsBuilder {
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

impl BackendSettingsBuilder {
    pub fn new(asset_tree: DirPathBuf) -> Self {
        BackendSettingsBuilder {
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

    pub fn build(self) -> BackendSettings {
        BackendSettings::from(self)
    }
}
