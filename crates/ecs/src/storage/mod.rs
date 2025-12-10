use std::collections::BTreeSet;

use entry::Entry;

use super::entity::index::Index;

pub mod entry;
pub mod iterators;
pub mod vec_storage;
pub mod zst_storage;

/// A component storage resource must provide the following methods.
pub trait Storage: Sized + std::ops::Index<Index, Output = Self::Item> {
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
    fn contains<I: Into<Index>>(&self, index: I) -> bool;

    /// Empties the component storage.
    fn clear(&mut self);

    /// Gets the given indexe's corresponding entry in the map for in-place manipulation.
    fn entry<I: Into<Index>>(&mut self, index: I) -> Entry<'_, Self::Item, Self> {
        let idx: Index = index.into();
        if self.contains(idx) {
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
    fn indices(&self) -> &BTreeSet<Index>;

    /// Borrows the component of type `Item` for the specified `Entity` without checking for existence.
    ///
    /// # Safety
    ///
    /// Calls to this method are only safe if you've previously verified that the item at the specified index actually exists.
    unsafe fn get_unchecked<I: Into<Index>>(&self, index: I) -> &Self::Item;

    /// Mutably borrows the component of type `Item` for the specified `Entity` without checking for existence.
    ///
    /// # Safety
    ///
    /// Calls to this method are only safe if you've previously verified that the item at the specified index actually exists.
    unsafe fn get_unchecked_mut<I: Into<Index>>(&mut self, index: I) -> &mut Self::Item;
}
