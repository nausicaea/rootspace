use super::Storage;
use crate::{entity::index::Index, resource::Resource};
use serde::{
    de::{Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};
use std::{collections::HashSet, marker::PhantomData};

/// Implements component storage for zero-sized types.
pub struct ZstStorage<T> {
    index: HashSet<Index>,
    data: T,
}

impl<T> ZstStorage<T> {
    fn insert_internal(&mut self, idx: Index) {
        self.index.insert(idx);
    }

    pub fn iter(&self) -> ZstStorageIter<T> {
        self.into_iter()
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

    fn has<I: Into<Index>>(&self, index: I) -> bool {
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

    fn index(&self) -> &HashSet<Index> {
        &self.index
    }

    unsafe fn get_unchecked<I: Into<Index>>(&self, _index: I) -> &T {
        &self.data
    }

    unsafe fn get_unchecked_mut<I: Into<Index>>(&mut self, _index: I) -> &mut T {
        &mut self.data
    }
}

impl<T> Resource for ZstStorage<T> where T: 'static {}

impl<T> Default for ZstStorage<T>
where
    T: Default,
{
    fn default() -> Self {
        ZstStorage {
            index: HashSet::default(),
            data: T::default(),
        }
    }
}

impl<'a, T> IntoIterator for &'a ZstStorage<T> {
    type Item = &'a T;
    type IntoIter = ZstStorageIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ZstStorageIter::new(self)
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
                ZstStorageVisitor(PhantomData::default())
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
                let mut storage = ZstStorage::default();

                while let Some(idx) = access.next_element::<Index>()? {
                    storage.insert_internal(idx);
                }

                Ok(storage)
            }
        }

        de.deserialize_seq(ZstStorageVisitor::<T>::default())
    }
}

pub struct ZstStorageIter<'a, T> {
    indices_len: usize,
    cursor: usize,
    data: &'a T,
}

impl<'a, T> ZstStorageIter<'a, T> {
    fn new(source: &'a ZstStorage<T>) -> Self {
        ZstStorageIter {
            indices_len: source.index.len(),
            cursor: 0,
            data: &source.data,
        }
    }
}

impl<'a, T> Iterator for ZstStorageIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.indices_len {
            return None;
        }

        self.cursor += 1;

        Some(self.data)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_len = self.indices_len
            .checked_sub(self.cursor)
            .unwrap_or(0);

        (remaining_len, Some(remaining_len))
    }
}

impl<'a, T> ExactSizeIterator for ZstStorageIter<'a, T> {}

impl<'a, T> std::iter::FusedIterator for ZstStorageIter<'a, T> {}
