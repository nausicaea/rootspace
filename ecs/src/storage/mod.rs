pub mod iterators;
pub mod vec_storage;
pub mod zst_storage;

use crate::entity::index::Index;
use std::collections::HashSet;

/// A component storage resource must provide the following methods.
pub trait Storage {
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
