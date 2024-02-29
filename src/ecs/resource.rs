//! Provides the resource downcastable trait.

use downcast_rs::{impl_downcast, Downcast, DowncastSync};

/// A resource is a data structure that is not coupled to a specific entity. Resources can be used
/// to provide "global" state to systems.
pub trait Resource: DowncastSync {}

impl_downcast!(sync Resource);

impl Resource for () {}
