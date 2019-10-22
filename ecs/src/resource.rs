//! Provides the resource downcastable trait.

use downcast_rs::{impl_downcast, Downcast};
use std::fmt::Debug;

/// A resource is a data structure that is not coupled to a specific entity. Resources can be used
/// to provide "global" state to systems.
pub trait Resource: Downcast + Debug {}

impl_downcast!(Resource);

impl Resource for () {}
