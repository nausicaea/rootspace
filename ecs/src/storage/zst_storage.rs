use super::Storage;
use crate::{entities::Entity, indexing::index::Index, resource::Resource};
use serde::{
    de::{Deserializer, SeqAccess, Visitor},
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};
use std::{collections::HashSet, fmt, marker::PhantomData};

/// Implements component storage for zero-sized types.
pub struct ZstStorage<T> {
    index: HashSet<Index>,
    _data: PhantomData<T>,
}

impl<T> ZstStorage<T> {
    fn insert_internal(&mut self, idx: Index) {
        self.index.insert(idx);
    }
}

impl<T> ZstStorage<T>
where
    T: Default
{
    pub fn iter(&self) -> ZstStorageIter<T> {
        self.into_iter()
    }
}

impl<T> Storage<T> for ZstStorage<T>
where
    T: Default,
{
    fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    fn len(&self) -> usize {
        self.index.len()
    }

    fn insert(&mut self, entity: Entity, _datum: T) -> Option<T> {
        self.insert_internal(entity.idx());
        None
    }

    fn remove(&mut self, entity: &Entity) -> Option<T> {
        self.index.remove(&entity.idx());
        None
    }

    fn has(&self, entity: &Entity) -> bool {
        self.index.contains(&entity.idx())
    }

    fn clear(&mut self) {
        self.index.clear()
    }

    fn get(&self, _entity: &Entity) -> Option<&T> {
        None
    }

    fn get_mut(&mut self, _entity: &Entity) -> Option<&mut T> {
        None
    }

    fn index(&self) -> &HashSet<Index> {
        &self.index
    }
}

impl<T> Resource for ZstStorage<T> where T: 'static {}

impl<T> Default for ZstStorage<T> {
    fn default() -> Self {
        ZstStorage {
            index: HashSet::default(),
            _data: PhantomData::default(),
        }
    }
}

impl<'a, T> IntoIterator for &'a ZstStorage<T> {
    type Item = Index;
    type IntoIter = ZstStorageIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        ZstStorageIter::new(self)
    }
}

impl<T> PartialEq<ZstStorage<T>> for ZstStorage<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.index.eq(&rhs.index)
    }
}

impl<T> fmt::Debug for ZstStorage<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl<'de, T> Deserialize<'de> for ZstStorage<T> {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ZstStorageVisitor<T>(PhantomData<T>);

        impl<T> Default for ZstStorageVisitor<T> {
            fn default() -> Self {
                ZstStorageVisitor(PhantomData::default())
            }
        }

        impl<'de, T> Visitor<'de> for ZstStorageVisitor<T> {
            type Value = ZstStorage<T>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a sequence of indices")
            }

            fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
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

pub struct ZstStorageIter<T> {
    indices: Vec<Index>,
    cursor: usize,
    _t: PhantomData<T>,
}

impl<T> ZstStorageIter<T> {
    fn new(source: &ZstStorage<T>) -> Self {
        ZstStorageIter {
            indices: source.index.iter().copied().collect(),
            cursor: 0,
            _t: PhantomData::default(),
        }
    }
}

impl<T> Iterator for ZstStorageIter<T> {
    type Item = Index;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.indices.len() {
            return None;
        }

        let idx = self.indices[self.cursor];
        self.cursor += 1;

        Some(idx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_len = self.indices.len()
            .checked_sub(self.cursor)
            .unwrap_or(0);

        (remaining_len, Some(remaining_len))
    }
}

impl<T> ExactSizeIterator for ZstStorageIter<T> {}

impl<T> std::iter::FusedIterator for ZstStorageIter<T> {}
