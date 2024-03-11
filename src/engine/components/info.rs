use std::fmt;

use crate::ecs::component::Component;
use crate::ecs::storage::vec_storage::VecStorage;
use serde::{Deserialize, Serialize};

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
