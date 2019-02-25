use crate::resources::Resource;
use crate::entities::Entity;
use std::fmt;
use std::ptr;
use hibitset::{BitSet, BitSetLike, BitIter};

pub trait Component: Sized {
    type Storage: Storage<Self>;
}

pub trait Storage<T> {
    fn insert(&mut self, entity: Entity, datum: T) -> Option<T>;
    fn remove(&mut self, entity: &Entity) -> Option<T>;
    fn has(&self, entity: &Entity) -> bool;
    fn clear(&mut self);
    fn get(&self, entity: &Entity) -> Option<&T>;
    fn get_mut(&mut self, entity: &Entity) -> Option<&mut T>;
}

pub struct VecStorage<T> {
    index: BitSet,
    data: Vec<T>,
}

impl<T> VecStorage<T> {
    pub fn insert(&mut self, entity: Entity, datum: T) -> Option<T> {
        let idx = entity.idx();
        let idx_usize = idx as usize;

        // Adjust the length of the data container if necessary.
        if self.data.len() <= idx_usize {
            self.data.reserve(idx_usize + 1 - self.data.len());
            unsafe {
                self.data.set_len(idx_usize + 1);
            }
        }

        // If the index was previously occupied, return the old piece of data.
        if self.index.add(idx) {
            unsafe {
                let old_datum = ptr::read(self.data.get_unchecked(idx_usize));
                ptr::write(self.data.get_unchecked_mut(idx_usize), datum);
                Some(old_datum)
            }
        } else {
            unsafe {
                ptr::write(self.data.get_unchecked_mut(idx_usize), datum);
                None
            }
        }
    }

    pub fn remove(&mut self, entity: &Entity) -> Option<T> {
        let idx = entity.idx();

        // If the index was previously occupied, return the old piece of data.
        if self.index.remove(idx) {
            unsafe {
                let old_datum = ptr::read(self.data.get_unchecked(idx as usize));
                Some(old_datum)
            }
        } else {
            None
        }
    }

    pub fn has(&self, entity: &Entity) -> bool {
        self.index.contains(entity.idx() as u32)
    }

    pub fn clear(&mut self) {
        let data = &mut self.data;

        for idx in (&self.index).iter() {
            unsafe {
                ptr::drop_in_place(data.get_unchecked_mut(idx as usize))
            }
        }

        self.index.clear();
        unsafe {
            data.set_len(0);
        }
    }

    pub fn get(&self, entity: &Entity) -> Option<&T> {
        let idx = entity.idx();

        if self.index.contains(idx) {
            unsafe {
                Some(self.data.get_unchecked(idx as usize))
            }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, entity: &Entity) -> Option<&mut T> {
        let idx = entity.idx();

        if self.index.contains(idx) {
            unsafe {
                Some(self.data.get_unchecked_mut(idx as usize))
            }
        } else {
            None
        }
    }

    pub fn iter(&self) -> VecStorageIter<T> {
        VecStorageIter {
            idx_iter: (&self.index).iter(),
            data: &self.data,
        }
    }
}

impl<T> Drop for VecStorage<T> {
    fn drop(&mut self) {
        self.clear()
    }
}

impl<T> Default for VecStorage<T> {
    fn default() -> Self {
        VecStorage {
            index: BitSet::default(),
            data: Vec::default(),
        }
    }
}

impl<T> fmt::Debug for VecStorage<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VecStorage {{ ... }}")
    }
}

impl<T> Resource for VecStorage<T> where T: 'static {}

pub struct VecStorageIter<'a, T> {
    idx_iter: BitIter<&'a BitSet>,
    data: &'a [T],
}

impl<'a, T> Iterator for VecStorageIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.idx_iter.next() {
            unsafe {
                Some(self.data.get_unchecked(idx as usize))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DropCounter<'a> {
        count: &'a mut usize,
    }

    impl<'a> Drop for DropCounter<'a> {
        fn drop(&mut self) {
            *self.count += 1;
        }
    }

    #[test]
    fn vec_storage_default() {
        let _: VecStorage<u32> = Default::default();
    }

    #[test]
    fn vec_storage_insert() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0, 1);
        assert!(s.insert(a, 101).is_none());

        let b = Entity::new(1, 1);
        assert!(s.insert(b, 102).is_none());

        let c = Entity::new(0, 3);
        assert_eq!(s.insert(c, 103), Some(101));
    }

    #[test]
    fn vec_storage_remove() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0, 1);
        assert!(s.remove(&a).is_none());

        let b = Entity::new(1, 1);
        let _ = s.insert(b, 102);
        assert_eq!(s.remove(&b), Some(102));
    }

    #[test]
    fn vec_storage_has() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0, 1);
        assert!(!s.has(&a));
        let _ = s.insert(a, 101);
        assert!(s.has(&a));
    }

    #[test]
    fn vec_storage_clear() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0, 1);
        let _ = s.insert(a, 101);

        let b = Entity::new(1, 1);
        let _ = s.insert(b, 102);

        let c = Entity::new(2, 1);
        let _ = s.insert(c, 103);

        assert!(s.has(&a));
        assert!(s.has(&b));
        assert!(s.has(&c));

        s.clear();

        assert!(!s.has(&a));
        assert!(!s.has(&b));
        assert!(!s.has(&c));
    }

    #[test]
    fn vec_storage_get() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0, 1);
        let _ = s.insert(a, 101);
        assert_eq!(s.get(&a) , Some(&101));

        let b = Entity::new(1, 1);
        assert!(s.get(&b).is_none());
    }

    #[test]
    fn vec_storage_get_mut() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0, 1);
        let _ = s.insert(a, 101);
        assert_eq!(s.get_mut(&a) , Some(&mut 101));

        let b = Entity::new(1, 1);
        assert!(s.get_mut(&b).is_none());
    }

    #[test]
    fn vec_storage_drops() {
        let mut a_count = 0usize;
        let mut b_count = 0usize;

        {
            let mut s: VecStorage<DropCounter> = Default::default();

            {
                let a = Entity::new(0, 1);
                let _ = s.insert(a, DropCounter { count: &mut a_count });
                let _ = s.remove(&a);
            }

            {
                let b = Entity::new(1, 1);
                let _ = s.insert(b, DropCounter { count: &mut b_count });
            }
        }

        assert_eq!(a_count, 1);
        assert_eq!(b_count, 1);
    }

    #[test]
    fn vec_storage_iter() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0, 1);
        let _ = s.insert(a, 101);

        let b = Entity::new(1, 1);
        let _ = s.insert(b, 102);

        let c = Entity::new(2, 1);
        let _ = s.insert(c, 103);

        let data: Vec<u32> = s.iter()
            .cloned()
            .collect();
        assert_eq!(data, vec![101, 102, 103]);
    }
}
