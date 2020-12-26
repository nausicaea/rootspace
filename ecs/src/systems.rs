use crate::system::System;

pub struct Systems(Vec<Box<dyn System>>);

impl Systems {
    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn insert<S>(&mut self, sys: S)
    where
        S: System,
    {
        self.0.push(Box::new(sys))
    }

    pub fn find<S>(&self) -> Option<&S>
    where
        S: System,
    {
        self.0
            .iter()
            .filter_map(|s| s.downcast_ref::<S>())
            .last()
    }

    pub fn find_mut<S>(&mut self) -> Option<&mut S>
    where
        S: System,
    {
        self.0
            .iter_mut()
            .filter_map(|s| s.downcast_mut::<S>())
            .last()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Box<dyn System>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn System>> {
        self.0.iter_mut()
    }
}

impl Default for Systems {
    fn default() -> Self {
        Systems(Vec::default())
    }
}
