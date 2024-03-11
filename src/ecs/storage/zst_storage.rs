use std::{collections::BTreeSet, marker::PhantomData};

use serde::{
    de::{Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};

use super::super::{entity::index::Index, resource::Resource, with_dependencies::WithDependencies};
use super::{
    iterators::{IndexedRIter, RIter, WIter},
    Storage,
};

/// Implements component storage for zero-sized types.
pub struct ZstStorage<T> {
    index: BTreeSet<Index>,
    data: T,
}

impl<T> ZstStorage<T> {
    fn insert_internal(&mut self, idx: Index) {
        self.index.insert(idx);
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
}

impl<T> ZstStorage<T>
where
    T: Default,
{
    pub fn with_capacity(_capacity: usize) -> Self {
        ZstStorage {
            index: BTreeSet::default(),
            data: T::default(),
        }
    }
}

impl<T> std::ops::Index<Index> for ZstStorage<T> {
    type Output = T;

    fn index(&self, index: Index) -> &Self::Output {
        self.get(index)
            .unwrap_or_else(|| panic!("Could not find the index {}", index))
    }
}

impl<T> Storage for ZstStorage<T> {
    type Item = T;

    fn len(&self) -> usize {
        self.index.len()
    }

    fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    fn insert<I: Into<Index>>(&mut self, index: I, _datum: T) -> Option<T> {
        self.insert_internal(index.into());
        None
    }

    fn remove<I: Into<Index>>(&mut self, index: I) -> Option<T> {
        self.index.remove(&index.into());
        None
    }

    fn contains<I: Into<Index>>(&self, index: I) -> bool {
        self.index.contains(&index.into())
    }

    fn clear(&mut self) {
        self.index.clear()
    }

    fn get<I: Into<Index>>(&self, _index: I) -> Option<&T> {
        Some(&self.data)
    }

    fn get_mut<I: Into<Index>>(&mut self, _index: I) -> Option<&mut T> {
        Some(&mut self.data)
    }

    fn indices(&self) -> &BTreeSet<Index> {
        &self.index
    }

    unsafe fn get_unchecked<I: Into<Index>>(&self, _index: I) -> &T {
        &self.data
    }

    unsafe fn get_unchecked_mut<I: Into<Index>>(&mut self, _index: I) -> &mut T {
        &mut self.data
    }
}

impl<T> Resource for ZstStorage<T> where T: 'static + Send + Sync {}

impl<T> Default for ZstStorage<T>
where
    T: Default,
{
    fn default() -> Self {
        ZstStorage {
            index: BTreeSet::default(),
            data: T::default(),
        }
    }
}

impl<D, T: Default> WithDependencies<D> for ZstStorage<T> {
    async fn with_deps(_: &D) -> Result<Self, anyhow::Error> {
        Ok(ZstStorage::default())
    }
}

impl<'a, T> IntoIterator for &'a ZstStorage<T> {
    type IntoIter = RIter<'a, ZstStorage<T>>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        RIter::new(self)
    }
}

impl<'a, T> IntoIterator for &'a mut ZstStorage<T> {
    type IntoIter = WIter<'a, ZstStorage<T>>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        WIter::new(self)
    }
}

impl<T> PartialEq<ZstStorage<T>> for ZstStorage<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.index.eq(&rhs.index)
    }
}

impl<T> std::fmt::Debug for ZstStorage<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ZstStorage(#len: {})", self.index.len())
    }
}

impl<T> Serialize for ZstStorage<T> {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = ser.serialize_seq(Some(self.index.len()))?;
        for idx in &self.index {
            state.serialize_element(idx)?;
        }
        state.end()
    }
}

impl<'de, T> Deserialize<'de> for ZstStorage<T>
where
    T: Default,
{
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ZstStorageVisitor<T>(PhantomData<T>);

        impl<U> Default for ZstStorageVisitor<U> {
            fn default() -> Self {
                ZstStorageVisitor(PhantomData)
            }
        }

        impl<'ef, U> Visitor<'ef> for ZstStorageVisitor<U>
        where
            U: Default,
        {
            type Value = ZstStorage<U>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a sequence of indices")
            }

            fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'ef>,
            {
                let mut storage = ZstStorage::with_capacity(access.size_hint().unwrap_or(0));

                while let Some(idx) = access.next_element::<Index>()? {
                    storage.insert_internal(idx);
                }

                Ok(storage)
            }
        }

        de.deserialize_seq(ZstStorageVisitor::<T>::default())
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{registry::End, registry::ResourceRegistry, world::World};
    use super::*;
    use crate::Reg;

    #[test]
    fn zst_storage_reg_macro() {
        type _RR = Reg![ZstStorage<u32>];
    }

    #[test]
    fn zst_storage_resource_registry() {
        let _rr = ResourceRegistry::push(End, ZstStorage::<usize>::default());
    }

    #[tokio::test]
    async fn zst_storage_world() {
        let _w = World::with_dependencies::<Reg![ZstStorage<usize>], Reg![], Reg![], (), Reg![], _>(&())
            .await
            .unwrap();
    }
}
