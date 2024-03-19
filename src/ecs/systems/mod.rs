use std::{iter::FusedIterator, sync::Arc};

use tokio::sync::Mutex;

use super::{registry::SystemRegistry, resources::Resources, system::System, with_resources::WithResources};

#[derive(Default)]
pub struct Systems(Vec<Arc<Mutex<Box<dyn System>>>>);

impl Systems {
    pub fn with_capacity(cap: usize) -> Self {
        Systems(Vec::with_capacity(cap))
    }

    #[tracing::instrument]
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

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn insert<S>(&mut self, sys: S)
    where
        S: System,
    {
        self.0.push(Arc::new(Mutex::new(Box::new(sys))))
    }

    pub fn iter(&self) -> SystemsIter {
        self.into_iter()
    }
}

// impl PartialEq for Systems {
//     fn eq(&self, rhs: &Self) -> bool {
//         if self.len() != rhs.len() {
//             return false;
//         }
//
//         self.0.iter().zip(rhs).all(|(lhs, rhs)| lhs.name() == rhs.name())
//     }
// }

impl std::fmt::Debug for Systems {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "System(#{})", self.0.len())
    }
}

impl<'a> IntoIterator for &'a Systems {
    type IntoIter = SystemsIter<'a>;
    type Item = <SystemsIter<'a> as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        SystemsIter::new(self)
    }
}

pub struct SystemsIter<'a> {
    length: usize,
    cursor: usize,
    systems: &'a Systems,
}

impl<'a> SystemsIter<'a> {
    fn new(systems: &'a Systems) -> Self {
        SystemsIter {
            length: systems.0.len(),
            cursor: 0,
            systems,
        }
    }
}

impl<'a> Iterator for SystemsIter<'a> {
    type Item = Arc<Mutex<Box<dyn System>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.length {
            return None;
        }

        let arc = self.systems.0[self.cursor].clone();
        self.cursor += 1;
        Some(arc)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rest = self.length.saturating_sub(self.cursor);
        (rest, Some(rest))
    }
}

impl<'a> ExactSizeIterator for SystemsIter<'a> {}

impl<'a> FusedIterator for SystemsIter<'a> {}
