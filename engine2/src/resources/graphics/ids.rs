/// Semi-private trait to allow conversion from usize only for internal functions
pub trait FromUsize {
    fn from_usize<P: self::private::IsPrivate>(value: usize) -> Self;
}

/// This **private-module-in-public-trait** trick was described wonderfully in
/// [Jack Wrenn's blog](https://jack.wrenn.fyi/blog/private-trait-methods/).
pub(super) mod private {
    pub enum Private {}

    pub trait IsPrivate {}

    impl IsPrivate for Private {}
}

macro_rules! impl_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(usize);

        impl Into<usize> for $name {
            fn into(self) -> usize {
                self.0
            }
        }

        impl FromUsize for $name {
            fn from_usize<L: $crate::resources::graphics::ids::private::IsPrivate>(value: usize) -> Self {
                $name(value)
            }
        }
    };
}

impl_id!(BindGroupLayoutId);
impl_id!(BindGroupId);
impl_id!(BufferId);
impl_id!(TextureId);
impl_id!(SamplerId);
impl_id!(PipelineId);
