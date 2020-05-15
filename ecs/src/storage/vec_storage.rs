use super::Storage;
use crate::{entities::Entity, indexing::Index, resource::Resource};
use serde::{
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};
use std::{collections::HashSet, fmt, marker::PhantomData, ptr};

/// Implements component storage based on a `Vec<T>`.
pub struct VecStorage<T> {
    /// The index into the data vector.
    index: HashSet<Index>,
    /// The data vector containing the components.
    data: Vec<T>,
}

impl<T> VecStorage<T> {
    /// Return an iterator over all occupied entries.
    pub fn iter(&self) -> VecStorageIter<T> {
        self.into_iter()
    }

    /// Return a mutable iterator over all occupied entries.
    pub fn iter_mut(&mut self) -> VecStorageIterMut<T> {
        self.into_iter()
    }

    fn insert_internal(&mut self, idx: Index, datum: T) -> Option<T> {
        let idx_usize: usize = idx.into();

        // Adjust the length of the data container if necessary.
        if self.data.len() <= idx_usize {
            self.data.reserve(idx_usize + 1 - self.data.len());
            unsafe {
                self.data.set_len(idx_usize + 1);
            }
        }

        // If the index was previously occupied, return the old piece of data.
        if !self.index.insert(idx) {
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
}

impl<T> Storage<T> for VecStorage<T> {
    fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    fn len(&self) -> usize {
        self.index.len()
    }

    fn insert(&mut self, entity: Entity, datum: T) -> Option<T> {
        let idx = entity.idx();
        self.insert_internal(idx, datum)
    }

    fn remove(&mut self, entity: &Entity) -> Option<T> {
        let idx = entity.idx();

        // If the index was previously occupied, return the old piece of data.
        if self.index.remove(&idx) {
            let idx_usize: usize = idx.into();
            unsafe {
                let old_datum = ptr::read(self.data.get_unchecked(idx_usize));
                Some(old_datum)
            }
        } else {
            None
        }
    }

    fn has(&self, entity: &Entity) -> bool {
        self.index.contains(&entity.idx())
    }

    fn clear(&mut self) {
        let data = &mut self.data;

        for idx in self.index.iter() {
            let idx_usize: usize = idx.into();
            unsafe { ptr::drop_in_place(data.get_unchecked_mut(idx_usize)) }
        }

        self.index.clear();
        unsafe {
            data.set_len(0);
        }
    }

    fn get(&self, entity: &Entity) -> Option<&T> {
        let idx = entity.idx();

        if self.index.contains(&idx) {
            let idx_usize: usize = idx.into();
            unsafe { Some(self.data.get_unchecked(idx_usize)) }
        } else {
            None
        }
    }

    fn get_mut(&mut self, entity: &Entity) -> Option<&mut T> {
        let idx = entity.idx();

        if self.index.contains(&idx) {
            let idx_usize: usize = idx.into();
            unsafe { Some(self.data.get_unchecked_mut(idx_usize)) }
        } else {
            None
        }
    }
}

impl<T> Resource for VecStorage<T> where T: 'static {}

impl<T> Drop for VecStorage<T> {
    fn drop(&mut self) {
        self.clear()
    }
}

impl<'a, T> IntoIterator for &'a VecStorage<T> {
    type Item = (Index, &'a T);
    type IntoIter = VecStorageIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        VecStorageIter::new(self)
    }
}

impl<'a, T> IntoIterator for &'a mut VecStorage<T> {
    type Item = (Index, &'a mut T);
    type IntoIter = VecStorageIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        VecStorageIterMut::new(self)
    }
}

impl<T> Default for VecStorage<T> {
    fn default() -> Self {
        VecStorage {
            index: HashSet::default(),
            data: Vec::default(),
        }
    }
}

impl<T> PartialEq<VecStorage<T>> for VecStorage<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, rhs: &Self) -> bool {
        if !self.index.eq(&rhs.index) {
            return false;
        }

        self.index
            .iter()
            .map(|idx| Into::<usize>::into(idx))
            .all(|idx| self.data[idx].eq(&rhs.data[idx]))
    }
}

impl<T> fmt::Debug for VecStorage<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VecStorage(#len: {})", self.len())
    }
}

impl<T> Serialize for VecStorage<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = ser.serialize_map(Some(self.index.len()))?;
        for idx in &self.index {
            state.serialize_entry(idx, &self.data[Into::<usize>::into(idx)])?;
        }
        state.end()
    }
}

impl<'de, T> Deserialize<'de> for VecStorage<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VecStorageVisitor<T>(PhantomData<T>);

        impl<T> Default for VecStorageVisitor<T> {
            fn default() -> Self {
                VecStorageVisitor(PhantomData::default())
            }
        }

        impl<'de, T> Visitor<'de> for VecStorageVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = VecStorage<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a map of indices to components")
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut storage = VecStorage::default();

                while let Some((idx, v)) = access.next_entry::<Index, T>()? {
                    storage.insert_internal(idx, v);
                }

                Ok(storage)
            }
        }

        de.deserialize_map(VecStorageVisitor::<T>::default())
    }
}

pub struct VecStorageIterMut<'a, T>
where
    T: 'a,
{
    data: &'a mut [T],
    indices: Vec<Index>,
    cursor: usize,
}

impl<'a, T> VecStorageIterMut<'a, T> {
    fn new(source: &'a mut VecStorage<T>) -> Self {
        VecStorageIterMut {
            data: &mut source.data,
            indices: source.index.iter().copied().collect(),
            cursor: 0,
        }
    }
}

impl<'a, T> Iterator for VecStorageIterMut<'a, T> {
    type Item = (Index, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.indices.len() {
            return None;
        }

        let idx = self.indices[self.cursor];
        self.cursor += 1;

        unsafe {
            let elem = self.data.get_unchecked_mut(Into::<usize>::into(idx));
            Some((idx, &mut *(elem as *mut _)))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_len = self.indices
            .len()
            .checked_sub(self.cursor)
            .unwrap_or(0);

        (remaining_len, Some(remaining_len))
    }
}

impl<'a, T> ExactSizeIterator for VecStorageIterMut<'a, T> {}

impl<'a, T> std::iter::FusedIterator for VecStorageIterMut<'a, T> {}

pub struct VecStorageIter<'a, T>
where
    T: 'a,
{
    data: &'a [T],
    indices: Vec<Index>,
    cursor: usize,
}

impl<'a, T> VecStorageIter<'a, T> {
    fn new(source: &'a VecStorage<T>) -> Self {
        VecStorageIter {
            data: &source.data,
            indices: source.index.iter().copied().collect(),
            cursor: 0,
        }
    }
}

impl<'a, T> Iterator for VecStorageIter<'a, T> {
    type Item = (Index, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.indices.len() {
            return None;
        }

        let idx = self.indices[self.cursor];
        self.cursor += 1;

        unsafe {
            Some((idx, self.data.get_unchecked(Into::<usize>::into(idx))))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_len = self.indices
            .len()
            .checked_sub(self.cursor)
            .unwrap_or(0);

        (remaining_len, Some(remaining_len))
    }
}

impl<'a, T> ExactSizeIterator for VecStorageIter<'a, T> {}

impl<'a, T> std::iter::FusedIterator for VecStorageIter<'a, T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Entities;
    use serde_test::{assert_tokens, Token};

    struct DropCounter<'a> {
        count: &'a mut usize,
    }

    impl<'a> Drop for DropCounter<'a> {
        fn drop(&mut self) {
            *self.count += 1;
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestComponent(usize);

    impl Component for TestComponent {
        type Storage = VecStorage<Self>;
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
        assert_eq!(s.get(&a), Some(&101));

        let b = Entity::new(1, 1);
        assert!(s.get(&b).is_none());
    }

    #[test]
    fn vec_storage_get_mut() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0, 1);
        let _ = s.insert(a, 101);
        assert_eq!(s.get_mut(&a), Some(&mut 101));

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

        let data: Vec<u32> = s.iter().cloned().collect();
        assert_eq!(data, vec![101, 102, 103]);
    }

    #[test]
    fn vec_storage_serde() {
        let mut entities = Entities::default();
        let mut v: <TestComponent as Component>::Storage = Default::default();

        let _a = entities.create();
        let _b = entities.create();
        let c = entities.create();

        v.insert(c, TestComponent(100));

        assert_tokens(
            &v,
            &[
                Token::Map { len: Some(1) },
                Token::U32(2),
                Token::NewtypeStruct { name: "TestComponent" },
                Token::U64(100),
                Token::MapEnd,
            ],
        );
    }
}
