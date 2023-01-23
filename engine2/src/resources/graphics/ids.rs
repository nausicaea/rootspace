macro_rules! impl_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(usize);

        impl Into<usize> for $name {
            fn into(self) -> usize {
                self.0
            }
        }

        impl From<usize> for $name {
            fn from(value: usize) -> Self {
                $name(value)
            }
        }
    };
}

impl_id!(BindGroupLayoutId);
impl_id!(BindGroupId);
impl_id!(BufferId);
impl_id!(TextureId);
impl_id!(TextureViewId);
impl_id!(SamplerId);
impl_id!(PipelineId);
impl_id!(ShaderModuleId);
