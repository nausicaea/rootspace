//! Provides the resource downcastable trait.

use downcast_rs::{impl_downcast, Downcast};
use std::fmt::Debug;

/// A resource is a data structure that is not coupled to a specific entity. Resources can be used
/// to provide "global" state to systems.
pub trait Resource: Downcast + Debug {}

impl_downcast!(Resource);

impl<T> Resource for T where T: Downcast + Debug {}

// impl Resource for () {}
//
// impl Resource for u8 {}
// impl Resource for u16 {}
// impl Resource for u32 {}
// impl Resource for u64 {}
// impl Resource for u128 {}
// impl Resource for usize {}
//
// impl Resource for i8 {}
// impl Resource for i16 {}
// impl Resource for i32 {}
// impl Resource for i64 {}
// impl Resource for i128 {}
// impl Resource for isize {}
//
// impl Resource for f32 {}
// impl Resource for f64 {}
