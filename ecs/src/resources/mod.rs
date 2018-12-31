pub mod entities;
pub mod components;

use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use mopa::Any;

pub trait Resource: Any + fmt::Debug {}

mopafy!(Resource);

#[derive(Default)]
pub struct Resources(HashMap<TypeId, Box<Resource>>);

impl Resources {
    pub fn insert<R>(&mut self, res: R) -> Option<R> where R: Resource {
        self.0.insert(TypeId::of::<R>(), Box::new(res))
            .map(|r| *r.downcast::<R>().expect("Could not downcast the resource"))
    }

    pub fn remove<R>(&mut self) -> Option<R> where R: Resource {
        self.0.remove(&TypeId::of::<R>())
            .map(|r| *r.downcast::<R>().expect("Could not downcast the resource"))
    }

    pub fn has<R>(&mut self) -> bool where R: Resource {
        self.0.contains_key(&TypeId::of::<R>())
    }

    pub fn get<R>(&self) -> Option<&R> where R: Resource {
        self.0.get(&TypeId::of::<R>())
            .map(|r| r.downcast_ref::<R>().expect("Could not downcast the resource"))
    }

    pub fn get_mut<R>(&mut self) -> Option<&mut R> where R: Resource {
        self.0.get_mut(&TypeId::of::<R>())
            .map(|r| r.downcast_mut::<R>().expect("Could not downcast the resource"))
    }
}
