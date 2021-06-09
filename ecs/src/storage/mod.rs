pub mod iterators;
pub mod vec_storage;
pub mod zst_storage;

use crate::entity::index::Index;
use std::collections::HashSet;

#[derive(Debug)]
pub enum Entry<'a, T: 'a, S: Storage<Item = T>> {
    Occupied(&'a mut S, Index),
    Vacant(&'a mut S, Index),
}

impl<'a, T: 'a, S: Storage<Item = T>> Entry<'a, T, S> {
    pub fn index(&self) -> &Index {
        match self {
            Entry::Occupied(_, ref i) => i,
            Entry::Vacant(_, ref i) => i,
        }
    }

    pub fn or_insert(self, default: T) -> &'a mut T {
        match self {
            Entry::Vacant(s, i) => {
                s.insert(i, default);
                unsafe { s.get_unchecked_mut(i) }
            },
            Entry::Occupied(s, i) => {
                unsafe { s.get_unchecked_mut(i) }
            }
        }
    }

    pub fn and_modify<F: FnOnce(&mut T)>(self, f: F) -> Self {
        match self {
            Entry::Vacant(s, i) => Entry::Vacant(s, i),
            Entry::Occupied(s, i) => {
                f(unsafe { s.get_unchecked_mut(i) });
                Entry::Occupied(s, i)
            }
        }
    }
}

impl<'a, T: Default + 'a, S: Storage<Item = T>> Entry<'a, T, S> {
    pub fn or_default(self) -> &'a mut T {
        self.or_insert(Default::default())
    }
}

/// A component storage resource must provide the following methods.
pub trait Storage: Sized {
    type Item;

    /// Return the number of stored components.
    fn len(&self) -> usize;

    /// Return `true` if the storage is empty.
    fn is_empty(&self) -> bool;

    /// Insert a component of type `Item` into the storage provider for the specified `Entity`.
    fn insert<I: Into<Index>>(&mut self, index: I, datum: Self::Item) -> Option<Self::Item>;

    /// Remove the specified component type from the specified `Entity`.
    fn remove<I: Into<Index>>(&mut self, index: I) -> Option<Self::Item>;

    /// Return `true` if the specified entity has a component of type `Item`.
    fn has<I: Into<Index>>(&self, index: I) -> bool;

    /// Empties the component storage.
    fn clear(&mut self);

    /// Gets the given indexe's corresponding entry in the map for in-place manipulation.
    fn entry<I: Into<Index>>(&mut self, index: I) -> Entry<'_, Self::Item, Self> {
        let idx: Index = index.into();
        if self.has(idx) {
            Entry::Occupied(self, idx)
        } else {
            Entry::Vacant(self, idx)
        }
    }

    /// Borrows the component of type `Item` for the specified `Entity`.
    fn get<I: Into<Index>>(&self, index: I) -> Option<&Self::Item>;

    /// Mutably borrows the component of type `Item` for the specified `Entity`.
    fn get_mut<I: Into<Index>>(&mut self, index: I) -> Option<&mut Self::Item>;

    /// Returns the registered indices
    fn index(&self) -> &HashSet<Index>;

    /// Borrows the component of type `Item` for the specified `Entity` without checking for existence.
    unsafe fn get_unchecked<I: Into<Index>>(&self, index: I) -> &Self::Item;

    /// Mutably borrows the component of type `Item` for the specified `Entity` without checking for existence.
    unsafe fn get_unchecked_mut<I: Into<Index>>(&mut self, index: I) -> &mut Self::Item;
}