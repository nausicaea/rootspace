use std::fmt;

use ecs::{Component, VecStorage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Info {
    name: String,
    description: String,
}

impl Info {
    pub fn new<S: AsRef<str>>(name: S, description: S) -> Self {
        Info::builder().with_name(name).with_description(description).build()
    }

    pub fn builder() -> InfoBuilder {
        InfoBuilder::default()
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

impl Default for Info {
    fn default() -> Self {
        Info::builder().build()
    }
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.description.is_empty() {
            write!(f, "{} ({})", self.name, self.description)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

#[derive(Debug, Default)]
pub struct InfoBuilder {
    name: Option<String>,
    description: Option<String>,
}

impl InfoBuilder {
    pub fn with_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = Some(description.as_ref().to_string());
        self
    }

    pub fn build(self) -> Info {
        Info {
            name: self.name.unwrap_or("".to_string()),
            description: self.description.unwrap_or("".to_string()),
        }
    }
}
