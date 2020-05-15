pub mod vec_storage;
pub mod zst_storage;

use crate::entities::Entity;

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

