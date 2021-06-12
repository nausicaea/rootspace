use std::fmt;

use ecs::{Component, VecStorage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Info {
    name: String,
    description: String,
}

impl Info {
    pub fn new<S: AsRef<str>>(name: S, description: S) -> Self {
        Info {
            name: name.as_ref().to_string(),
            description: description.as_ref().to_string(),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn set_name<S: AsRef<str>>(&mut self, name: S) {
        self.name = name.as_ref().to_string();
    }

    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    pub fn set_description<S: AsRef<str>>(&mut self, description: S) {
        self.description = description.as_ref().to_string();
    }

}

impl Component for Info {
    type Storage = VecStorage<Self>;
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
