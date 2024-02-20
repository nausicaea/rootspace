use std::{collections::BTreeSet, marker::PhantomData, ptr};

use serde::{
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};

use super::super::{
    entity::index::Index, resource::Resource, storage::entry::Entry, with_dependencies::WithDependencies,
};
use super::{
    iterators::{IndexedRIter, RIter, WIter},
    Storage,
};

/// Implements component storage based on a `Vec<T>`.
pub struct VecStorage<T> {
    /// The index into the data vector.
    index: BTreeSet<Index>,
    /// The data vector containing the components.
    data: Vec<T>,
}

impl<T> VecStorage<T> {
    pub fn with_capacity(_capacity: usize) -> Self {
        VecStorage {
            index: BTreeSet::default(),
            data: Vec::default(),
        }
    }

    pub fn iter(&self) -> RIter<Self> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> WIter<Self> {
        self.into_iter()
    }

    pub fn indexed_iter(&self) -> IndexedRIter<Self> {
        IndexedRIter::new(self)
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

impl<T> std::ops::Index<Index> for VecStorage<T> {
    type Output = T;

    fn index(&self, index: Index) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("Could not find the index {}", index))
    }
}

impl<T> Storage for VecStorage<T> {
    type Item = T;

    fn len(&self) -> usize {
        self.index.len()
    }

    fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    fn insert<I: Into<Index>>(&mut self, index: I, datum: T) -> Option<T> {
        self.insert_internal(index.into(), datum)
    }

    fn remove<I: Into<Index>>(&mut self, index: I) -> Option<T> {
        let idx: Index = index.into();

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

    fn contains<I: Into<Index>>(&self, index: I) -> bool {
        self.index.contains(&index.into())
    }

    fn clear(&mut self) {
        let data = &mut self.data;

        for idx in &self.index {
            let idx_usize: usize = idx.into();
            unsafe { ptr::drop_in_place(data.get_unchecked_mut(idx_usize)) }
        }

        self.index.clear();
        unsafe {
            data.set_len(0);
        }
    }

    fn entry<I: Into<Index>>(&mut self, index: I) -> Entry<'_, T, Self> {
        let idx: Index = index.into();
        if self.index.contains(&idx) {
            Entry::Occupied(self, idx)
        } else {
            Entry::Vacant(self, idx)
        }
    }

    fn get<I: Into<Index>>(&self, index: I) -> Option<&T> {
        let idx: Index = index.into();

        if self.index.contains(&idx) {
            let idx_usize: usize = idx.into();
            unsafe { Some(self.data.get_unchecked(idx_usize)) }
        } else {
            None
        }
    }

    fn get_mut<I: Into<Index>>(&mut self, index: I) -> Option<&mut T> {
        let idx: Index = index.into();

        if self.index.contains(&idx) {
            let idx_usize: usize = idx.into();
            unsafe { Some(self.data.get_unchecked_mut(idx_usize)) }
        } else {
            None
        }
    }

    fn indices(&self) -> &BTreeSet<Index> {
        &self.index
    }

    unsafe fn get_unchecked<I: Into<Index>>(&self, index: I) -> &T {
        let idx: Index = index.into();
        let idx_usize: usize = idx.into();
        self.data.get_unchecked(idx_usize)
    }

    unsafe fn get_unchecked_mut<I: Into<Index>>(&mut self, index: I) -> &mut T {
        let idx: Index = index.into();
        let idx_usize: usize = idx.into();
        self.data.get_unchecked_mut(idx_usize)
    }
}

impl<T> Resource for VecStorage<T> where T: 'static {}

impl<T> Drop for VecStorage<T> {
    fn drop(&mut self) {
        self.clear()
    }
}

impl<'a, T> IntoIterator for &'a VecStorage<T> {
    type IntoIter = RIter<'a, VecStorage<T>>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        RIter::new(self)
    }
}

impl<'a, T> IntoIterator for &'a mut VecStorage<T> {
    type IntoIter = WIter<'a, VecStorage<T>>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        WIter::new(self)
    }
}

impl<T> Default for VecStorage<T> {
    fn default() -> Self {
        VecStorage {
            index: BTreeSet::default(),
            data: Vec::default(),
        }
    }
}

impl<D, T> WithDependencies<D> for VecStorage<T> {
    fn with_deps(_: &D) -> Result<Self, anyhow::Error> {
        Ok(VecStorage::default())
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
            .map(Into::<usize>::into)
            .all(|idx| self.data[idx].eq(&rhs.data[idx]))
    }
}

impl<T> std::fmt::Debug for VecStorage<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a map of indices to components")
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut storage = VecStorage::with_capacity(access.size_hint().unwrap_or(0));

                while let Some((idx, v)) = access.next_entry::<Index, T>()? {
                    storage.insert_internal(idx, v);
                }

                Ok(storage)
            }
        }

        de.deserialize_map(VecStorageVisitor::<T>::default())
    }
}

#[cfg(test)]
mod tests {
    use std::{iter::Sum, ops::Add};

    use serde_test::{assert_tokens, Token};

    use super::super::super::{
        component::Component, entities::Entities, entity::Entity, registry::End, registry::ResourceRegistry,
        world::World,
    };
    use super::*;
    use crate::Reg;

    struct DropCounter<'a> {
        count: &'a mut usize,
    }

    impl<'a> Drop for DropCounter<'a> {
        fn drop(&mut self) {
            *self.count += 1;
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Tc(usize);

    impl Component for Tc {
        type Storage = VecStorage<Self>;
    }

    impl<'a> Add for &'a Tc {
        type Output = Tc;

        fn add(self, rhs: Self) -> Self::Output {
            Tc(self.0 + rhs.0)
        }
    }

    impl Add for Tc {
        type Output = Tc;

        fn add(self, rhs: Self) -> Self::Output {
            Tc(self.0 + rhs.0)
        }
    }

    impl<'a> Sum<&'a Tc> for Tc {
        fn sum<I: Iterator<Item = &'a Tc>>(iter: I) -> Self {
            iter.fold(Tc(0), |state, value| &state + value)
        }
    }

    impl Sum for Tc {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
            iter.fold(Tc(0), |state, value| state + value)
        }
    }

    #[test]
    fn vec_storage_reg_macro() {
        type _RR = Reg![VecStorage<u32>];
    }

    #[test]
    fn vec_storage_resource_registry() {
        let _rr = ResourceRegistry::push(End, VecStorage::<usize>::default());
    }

    #[test]
    fn vec_storage_world() {
        let _w = World::with_dependencies::<Reg![VecStorage<usize>], Reg![], Reg![], Reg![], _>(&()).unwrap();
    }

    #[test]
    fn vec_storage_default() {
        let _: VecStorage<u32> = Default::default();
    }

    #[test]
    fn vec_storage_insert() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0u32, 1u32);
        assert!(s.insert(a, 101).is_none());

        let b = Entity::new(1u32, 1u32);
        assert!(s.insert(b, 102).is_none());

        let c = Entity::new(0u32, 3u32);
        assert_eq!(s.insert(c, 103), Some(101));
    }

    #[test]
    fn vec_storage_remove() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0u32, 1u32);
        assert!(s.remove(&a).is_none());

        let b = Entity::new(1u32, 1u32);
        let _ = s.insert(b, 102);
        assert_eq!(s.remove(&b), Some(102));
    }

    #[test]
    fn vec_storage_has() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0u32, 1u32);
        assert!(!s.contains(&a));
        let _ = s.insert(a, 101);
        assert!(s.contains(&a));
    }

    #[test]
    fn vec_storage_clear() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0u32, 1u32);
        let _ = s.insert(a, 101);

        let b = Entity::new(1u32, 1u32);
        let _ = s.insert(b, 102);

        let c = Entity::new(2u32, 1u32);
        let _ = s.insert(c, 103);

        assert!(s.contains(&a));
        assert!(s.contains(&b));
        assert!(s.contains(&c));

        s.clear();

        assert!(!s.contains(&a));
        assert!(!s.contains(&b));
        assert!(!s.contains(&c));
    }

    #[test]
    fn vec_storage_get() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0u32, 1u32);
        let _ = s.insert(a, 101);
        assert_eq!(s.get(&a), Some(&101));

        let b = Entity::new(1u32, 1u32);
        assert!(s.get(&b).is_none());
    }

    #[test]
    fn vec_storage_get_mut() {
        let mut s: VecStorage<u32> = Default::default();

        let a = Entity::new(0u32, 1u32);
        let _ = s.insert(a, 101);
        assert_eq!(s.get_mut(&a), Some(&mut 101));

        let b = Entity::new(1u32, 1u32);
        assert!(s.get_mut(&b).is_none());
    }

    #[test]
    fn vec_storage_aggregate() {
        let mut s: <Tc as Component>::Storage = Default::default();
        s.insert(0u32, Tc(2));
        s.insert(1u32, Tc(3));
        s.insert(2u32, Tc(5));
        s.insert(3u32, Tc(7));

        assert_eq!(
            s.indexed_iter()
                .filter(|(idx, _)| [0u32, 1, 3].iter().any(|i| idx == i))
                .map(|(_, tc)| tc)
                .sum::<Tc>(),
            Tc(12)
        );
        assert_eq!([0u32, 1, 3].iter().filter_map(|i| s.get(*i)).sum::<Tc>(), Tc(12));
    }

    #[test]
    fn vec_storage_drops() {
        let mut a_count = 0usize;
        let mut b_count = 0usize;

        {
            let mut s: VecStorage<DropCounter> = Default::default();

            {
                let a = Entity::new(0u32, 1u32);
                let _ = s.insert(a, DropCounter { count: &mut a_count });
                let _ = s.remove(&a);
            }

            {
                let b = Entity::new(1u32, 1u32);
                let _ = s.insert(b, DropCounter { count: &mut b_count });
            }
        }

        assert_eq!(a_count, 1);
        assert_eq!(b_count, 1);
    }

    #[test]
    fn vec_storage_serde() {
        let mut entities = Entities::default();
        let mut v: <Tc as Component>::Storage = Default::default();

        let _a = entities.create();
        let _b = entities.create();
        let c = entities.create();

        v.insert(c, Tc(100));

        assert_tokens(
            &v,
            &[
                Token::Map { len: Some(1) },
                Token::U32(2),
                Token::NewtypeStruct { name: "Tc" },
                Token::U64(100),
                Token::MapEnd,
            ],
        );
    }
}
