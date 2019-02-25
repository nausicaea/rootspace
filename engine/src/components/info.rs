use ecs::{Component, VecStorage};

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

    pub fn description(&self) -> &str {
        self.description.as_str()
    }
}

impl Component for Info {
    type Storage = VecStorage<Self>;
}
