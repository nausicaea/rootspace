use std::any::TypeId;
use std::cell::{RefCell, Ref, RefMut};
use std::collections::HashMap;
use std::fmt;
use mopa::Any;

pub trait Resource: Any + fmt::Debug {}

mopafy!(Resource);

#[derive(Default)]
pub struct Resources(HashMap<TypeId, RefCell<Box<Resource>>>);

impl Resources {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn insert<R>(&mut self, res: R) -> Option<R> where R: Resource {
        self.0.insert(TypeId::of::<R>(), RefCell::new(Box::new(res)))
            .map(|r| *r.into_inner().downcast::<R>().expect("Could not downcast the resource"))
    }

    pub fn remove<R>(&mut self) -> Option<R> where R: Resource {
        self.0.remove(&TypeId::of::<R>())
            .map(|r| *r.into_inner().downcast::<R>().expect("Could not downcast the resource"))
    }

    pub fn has<R>(&mut self) -> bool where R: Resource {
        self.0.contains_key(&TypeId::of::<R>())
    }

    pub fn borrow<R>(&self) -> Option<Ref<R>> where R: Resource {
        self.0.get(&TypeId::of::<R>())
            .map(|r| Ref::map(r.borrow(), |i| i.downcast_ref::<R>().expect("Could not downcast the resource")))
    }

    pub fn borrow_mut<R>(&self) -> Option<RefMut<R>> where R: Resource {
        self.0.get(&TypeId::of::<R>())
            .map(|r| RefMut::map(r.borrow_mut(), |i| i.downcast_mut::<R>().expect("Could not downcast the resource")))
    }

    pub fn get_mut<R>(&mut self) -> Option<&mut R> where R: Resource {
        self.0.get_mut(&TypeId::of::<R>())
            .map(|r| r.get_mut().downcast_mut::<R>().expect("Could not downcast the resource"))
    }
}
