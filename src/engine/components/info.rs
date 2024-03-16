use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ecs::{component::Component, storage::vec_storage::VecStorage};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Info {
    pub name: String,
    pub description: String,
    #[serde(skip)]
    pub origin: Option<(String, String)>,
}

impl Info {
    pub fn new<S: AsRef<str>>(name: S, description: S) -> Self {
        Info {
            name: name.as_ref().to_string(),
            description: description.as_ref().to_string(),
            ..Default::default()
        }
    }
}

impl Component for Info {
    type Storage = VecStorage<Self>;
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some((ref grp, ref nm)) = self.origin {
            write!(f, "{}@{}:{}", self.name, grp, nm)
        } else {
            write!(f, "{}", self.name)
        }
    }
}
