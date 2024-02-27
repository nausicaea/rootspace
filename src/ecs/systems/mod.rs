use std::slice::{Iter, IterMut};

use super::{registry::SystemRegistry, resources::Resources, system::System, with_resources::WithResources};

#[derive(Default)]
pub struct Systems(Vec<Box<dyn System>>);

impl Systems {
    pub fn with_capacity(cap: usize) -> Self {
        Systems(Vec::with_capacity(cap))
    }

    pub async fn with_resources<SR>(res: &Resources) -> Result<Self, anyhow::Error>
    where
        SR: SystemRegistry + WithResources,
    {
        fn recursor<S: SystemRegistry>(sys: &mut Systems, reg: S) {
            if S::LEN == 0 {
                return;
            }

            let (head, tail) = reg.unzip();
            sys.insert(head);
            recursor(sys, tail);
        }

        let sr = SR::with_res(res).await?;
        let mut sys = Systems::with_capacity(SR::LEN);
        recursor(&mut sys, sr);

        Ok(sys)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains<S>(&self) -> bool
    where
        S: System,
    {
        self.0.iter().any(|s| s.is::<S>())
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn insert<S>(&mut self, sys: S)
    where
        S: System,
    {
        self.0.push(Box::new(sys))
    }

    pub fn get<S>(&self) -> &S
    where
        S: System,
    {
        self.find::<S>()
            .unwrap_or_else(|| panic!("Could not find the system {}", std::any::type_name::<S>()))
    }

    pub fn get_mut<S>(&mut self) -> &mut S
    where
        S: System,
    {
        self.find_mut::<S>()
            .unwrap_or_else(|| panic!("Could not find the system {}", std::any::type_name::<S>()))
    }

    pub fn find_with_position<S>(&self) -> Option<(usize, &S)>
    where
        S: System,
    {
        self.0
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.downcast_ref::<S>().map(|sdc| (i, sdc)))
            .next()
    }

    pub fn find<S>(&self) -> Option<&S>
    where
        S: System,
    {
        self.0.iter().filter_map(|s| s.downcast_ref::<S>()).next()
    }

    pub fn find_mut<S>(&mut self) -> Option<&mut S>
    where
        S: System,
    {
        self.0.iter_mut().filter_map(|s| s.downcast_mut::<S>()).last()
    }

    pub fn iter(&self) -> Iter<'_, Box<dyn System>> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Box<dyn System>> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a Systems {
    type IntoIter = Iter<'a, Box<dyn System>>;
    type Item = &'a Box<dyn System>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).iter()
    }
}

impl<'a> IntoIterator for &'a mut Systems {
    type IntoIter = IterMut<'a, Box<dyn System>>;
    type Item = &'a mut Box<dyn System>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).iter_mut()
    }
}

impl PartialEq for Systems {
    fn eq(&self, rhs: &Self) -> bool {
        if self.len() != rhs.len() {
            return false;
        }

        self.0.iter().zip(rhs).all(|(lhs, rhs)| lhs.name() == rhs.name())
    }
}

impl std::fmt::Debug for Systems {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "System(#{})", self.0.len())
    }
}
