//! Provides facilities for reasoning about data (e.g. components) coupled to entities.

use crate::{entities::Entity, indexing::Index, resource::Resource};
use serde::{Deserialize, Serialize, ser::{Serializer, SerializeMap, SerializeSeq}, de::{Visitor, Deserializer, MapAccess, SeqAccess}};
use std::{collections::HashSet, fmt, ptr};
use typename::TypeName;
use std::marker::PhantomData;

/// A component is a data type that is associated with a particular `Entity`.
pub trait Component: Sized {
    /// Components are stored in a `Resource` and the implementor of a component may choose the
    /// type of storage used.
    type Storage: Storage<Self> + Resource + Default;
}

macro_rules! impl_component {
    ($t:ty, $s:ident) => {
        impl Component for $t {
            type Storage = $s<Self>;
        }
    };
}

impl_component!((), ZstStorage);
impl_component!(bool, VecStorage);
impl_component!(u8, VecStorage);
impl_component!(i8, VecStorage);
impl_component!(u16, VecStorage);
impl_component!(i16, VecStorage);
impl_component!(u32, VecStorage);
impl_component!(i32, VecStorage);
impl_component!(u64, VecStorage);
impl_component!(i64, VecStorage);
impl_component!(u128, VecStorage);
impl_component!(i128, VecStorage);
impl_component!(usize, VecStorage);
impl_component!(isize, VecStorage);
impl_component!(f32, VecStorage);
impl_component!(f64, VecStorage);
impl_component!(char, VecStorage);
impl_component!(String, VecStorage);

/// A component storage resource must provide the following methods.
pub trait Storage<T> {
    /// Return the number of stored components.
    fn len(&self) -> usize;
    /// Return `true` if the storage is empty.
    fn is_empty(&self) -> bool;
    /// Insert a component of type `T` into the storage provider for the specified `Entity`.
    fn insert(&mut self, entity: Entity, datum: T) -> Option<T>;
    /// Remove the specified component type from the specified `Entity`.
    fn remove(&mut self, entity: &Entity) -> Option<T>;
    /// Return `true` if the specified entity has a component of type `T`.
    fn has(&self, entity: &Entity) -> bool;
    /// Empties the component storage.
    fn clear(&mut self);
    /// Borrows the component of type `T` for the specified `Entity`.
    fn get(&self, entity: &Entity) -> Option<&T>;
    /// Mutably borrows the component of type `T` for the specified `Entity`.
    fn get_mut(&mut self, entity: &Entity) -> Option<&mut T>;
}

/// Implements component storage for zero-sized types.
#[derive(TypeName)]
pub struct ZstStorage<T> {
    index: HashSet<Index>,
    _data: PhantomData<T>,
}

impl<T> ZstStorage<T> {
    fn insert_internal(&mut self, idx: Index) {
        self.index.insert(idx);
    }
}

impl<T> Storage<T> for ZstStorage<T> {
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

impl<T> PartialEq<ZstStorage<T>> for ZstStorage<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.index.eq(&rhs.index)
    }
}

impl<T> fmt::Debug for ZstStorage<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ZstStorage(#len: {})", self.len())
    }
}

impl<T> Serialize for ZstStorage<T> {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = ser.serialize_seq(Some(self.len()))?;
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

/// Implements component storage based on a `Vec<T>`.
#[derive(TypeName)]
pub struct VecStorage<T> {
    /// The index into the data vector.
    index: HashSet<Index>,
    /// The data vector containing the components.
    data: Vec<T>,
}

impl<T> VecStorage<T> {
    /// Return an iterator over all occupied entries.
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        let index = &self.index;
        self.data
            .iter()
            .enumerate()
            .filter(move |(idx, _)| index.contains(&idx.into()))
            .map(|(_, d)| d)
    }

    /// Return a mutable iterator over all occupied entries.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        let index = &self.index;
        self.data
            .iter_mut()
            .enumerate()
            .filter(move |(idx, _)| index.contains(&idx.into()))
            .map(|(_, d)| d)
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

        self.index.iter()
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

        assert_tokens(&v, &[
            Token::Map { len: Some(1) },
            Token::U32(2),
            Token::NewtypeStruct { name: "TestComponent" },
            Token::U64(100),
            Token::MapEnd,
        ]);
    }
}
