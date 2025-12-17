use super::{registry::SystemRegistry, resources::Resources, system::System, with_resources::WithResources};
use parking_lot::Mutex;
use rayon::iter::IntoParallelRefIterator;
use std::{iter::FusedIterator, sync::Arc};

#[derive(Default)]
pub struct Systems(Vec<Arc<Mutex<Box<dyn System>>>>);

impl Systems {
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        Systems(Vec::with_capacity(cap))
    }

    #[tracing::instrument(skip_all)]
    pub fn with_resources<SR>(res: &Resources) -> anyhow::Result<Self>
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

        let sr = SR::with_res(res)?;
        let mut sys = Systems::with_capacity(SR::LEN);
        recursor(&mut sys, sr);

        Ok(sys)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn insert<S>(&mut self, sys: S)
    where
        S: System,
    {
        self.0.push(Arc::new(Mutex::new(Box::new(sys))));
    }

    #[must_use]
    pub fn iter(&self) -> SystemsIter<'_> {
        self.into_iter()
    }

    pub fn par_iter(&'_ self) -> rayon::slice::Iter<'_, Arc<Mutex<Box<dyn System>>>> {
        self.0.par_iter()
    }
}

impl std::fmt::Debug for Systems {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "System(#{})", self.0.len())
    }
}

impl<'a> IntoIterator for &'a Systems {
    type Item = <SystemsIter<'a> as Iterator>::Item;
    type IntoIter = SystemsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        SystemsIter::new(self)
    }
}

pub struct SystemsIter<'a> {
    length: usize,
    cursor: usize,
    systems: &'a [Arc<Mutex<Box<dyn System>>>],
}

impl<'a> SystemsIter<'a> {
    fn new(systems: &'a Systems) -> Self {
        SystemsIter {
            length: systems.0.len(),
            cursor: 0,
            systems: &systems.0,
        }
    }
}

impl Iterator for SystemsIter<'_> {
    type Item = Arc<Mutex<Box<dyn System>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.length {
            return None;
        }

        let arc = self.systems[self.cursor].clone();
        self.cursor += 1;
        Some(arc)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rest = self.length.saturating_sub(self.cursor);
        (rest, Some(rest))
    }
}

impl ExactSizeIterator for SystemsIter<'_> {}

impl FusedIterator for SystemsIter<'_> {}
