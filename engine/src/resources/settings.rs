use ecs::{Resource, SerializationProxy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub clear_color: [f32; 4],
    pub command_escape: char,
    pub command_quote: char,
    pub command_punctuation: char,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            clear_color: [0.69, 0.93, 0.93, 1.0],
            command_escape: '\\',
            command_quote: '"',
            command_punctuation: ';',
        }
    }
}

impl Resource for Settings {}

impl SerializationProxy for Settings {}
