use crate::resource::Resource;
use crate::storage::Storage;
use crate::storage::zst_storage::ZstStorage;
use crate::storage::vec_storage::VecStorage;

/// A component is a data type that is associated with a particular `Entity`.
pub trait Component: Sized {
    /// Components are stored in a `Resource` and the implementor of a component may choose the
    /// type of storage used.
    type Storage: Storage<Item = Self> + Resource + Default;
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
